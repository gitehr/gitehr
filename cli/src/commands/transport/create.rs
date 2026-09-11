// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Repository directories copied into a transport archive.
///
/// Unlike `scaffold`, which copies the whole `folder-structure` template
/// generically, this list is explicit, so a new template directory is only
/// archived once it is added here. `every_template_directory_is_archived`
/// enforces that, because a directory missed here is silently absent from
/// every archive the user believes is their portable record.
const INCLUDED_DIRS: [&str; 7] = [
    "journal",
    "state",
    "imaging",
    "documents",
    "openehr",
    "fhir",
    ".gitehr",
];

pub fn run(output_path: Option<&str>, encrypt: bool) -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    // Refusing is deliberate (roadmap R81): silently producing a plaintext
    // archive after the user asked for encryption is false assurance.
    if encrypt {
        anyhow::bail!(
            "Transport encryption is not yet implemented (roadmap R68). \
Re-run without --encrypt to create an unencrypted archive."
        );
    }

    let default_output = format!(
        "gitehr-transport-{}.tar.gz",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    let output = output_path.unwrap_or(&default_output);

    println!("Creating transport archive: {}", output);

    let tar_file = File::create(output)?;
    let encoder = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    for dir in &INCLUDED_DIRS {
        let path = PathBuf::from(dir);
        if path.exists() {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                // Use the WalkDir-cached file type (symlink_metadata), not
                // entry_path.is_file() (fs::metadata), which follows symlinks.
                // A symlink planted in the repo (e.g. by a hostile contributor
                // or a received/cloned repo) must not be dereferenced here, or
                // `transport create` becomes an arbitrary-file-read primitive.
                if entry.file_type().is_file() {
                    let mut file = File::open(entry_path)?;
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)?;

                    let mut header = tar::Header::new_gnu();
                    header.set_size(contents.len() as u64);
                    header.set_mode(0o644);
                    header.set_cksum();

                    archive.append_data(
                        &mut header,
                        entry_path.to_string_lossy().to_string(),
                        contents.as_slice(),
                    )?;
                }
            }
        }
    }

    archive.finish()?;

    if encrypt {
        println!("NOTE: Transport encryption is not yet implemented.");
        println!("The archive was created without additional encryption layer.");
    }

    println!("Transport archive created successfully: {}", output);
    println!();
    println!("This archive contains the complete GitEHR repository data.");
    println!("It can be extracted with: tar -xzf {}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::INCLUDED_DIRS;
    use std::path::PathBuf;

    /// Adding a directory to `folder-structure` puts it in every new
    /// repository automatically, but not into transport archives: this
    /// module's list is explicit. Name any template directory that has been
    /// scaffolded but never archived, rather than leaving the omission to be
    /// noticed when someone restores an incomplete record.
    ///
    /// Only this direction is checked. An entry here with no template
    /// directory is legitimate - a directory a repository grows at runtime
    /// still belongs in the archive.
    #[test]
    fn every_template_directory_is_archived() {
        let template = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli/ has a parent")
            .join("folder-structure");

        let missing: Vec<String> = std::fs::read_dir(&template)
            .expect("folder-structure template exists")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| !INCLUDED_DIRS.contains(&name.as_str()))
            .collect();

        assert!(
            missing.is_empty(),
            "template directories missing from transport archives: {missing:?}. \
             Add each to INCLUDED_DIRS, or document why it must not be archived."
        );
    }
}
