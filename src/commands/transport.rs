// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

pub fn create_transport_archive(output_path: Option<&str>, encrypt: bool) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
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

    let dirs_to_include = ["journal", "state", "imaging", "documents", ".gitehr"];

    for dir in &dirs_to_include {
        let path = PathBuf::from(dir);
        if path.exists() {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
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

pub fn extract_transport_archive(archive_path: &str, output_dir: Option<&str>) -> Result<()> {
    let archive_file = File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);

    let output = output_dir.unwrap_or(".");

    println!("Extracting transport archive to: {}", output);

    archive.unpack(output)?;

    println!("Archive extracted successfully.");

    let gitehr_dir = PathBuf::from(output).join(".gitehr");
    if gitehr_dir.exists() {
        println!("GitEHR repository detected in extracted files.");
    }

    Ok(())
}
