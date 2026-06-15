use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    MANIFEST_FILENAME, build_manifest, check_destination, copy_tree, ensure_gitehr_repository,
    hash_file, slugify,
};
use crate::commands::journal::{self, DocumentRef};

/// Add a file or directory to the record as a Document and link it from a new
/// journal entry. Returns the stored path within the record.
pub fn run(
    source: &Path,
    title: Option<&str>,
    imaging: bool,
    message: Option<&str>,
) -> Result<String> {
    ensure_gitehr_repository()?;

    if !source.exists() {
        bail!("No such file or directory: {}", source.display());
    }

    let original_filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let slug = slugify(title.unwrap_or_else(|| {
        source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
    }));
    let date = Utc::now().format("%Y-%m-%d");
    let root = if imaging { "imaging" } else { "documents" };
    fs::create_dir_all(root)?;

    let (dest, sha256) = if source.is_dir() {
        if source.join(MANIFEST_FILENAME).exists() {
            bail!(
                "Directory already contains a {} - this name is reserved for the Document manifest",
                MANIFEST_FILENAME
            );
        }
        let (manifest_bytes, manifest_hash) = build_manifest(source)?;
        let dest = PathBuf::from(root).join(format!("{}-{}-{}", date, slug, &manifest_hash[..8]));
        check_destination(&dest)?;
        copy_tree(source, &dest)?;
        fs::write(dest.join(MANIFEST_FILENAME), &manifest_bytes)
            .context("Failed to write manifest")?;
        (dest, manifest_hash)
    } else {
        let hash = hash_file(source)?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_ascii_lowercase()))
            .unwrap_or_default();
        let dest = PathBuf::from(root).join(format!("{}-{}-{}{}", date, slug, &hash[..8], ext));
        check_destination(&dest)?;
        fs::copy(source, &dest)
            .with_context(|| format!("Failed to copy {} into the record", source.display()))?;
        (dest, hash)
    };

    let dest_str = dest.to_string_lossy().to_string();
    crate::commands::git::git_add(&dest_str)?;

    let body = message
        .map(|m| m.to_string())
        .unwrap_or_else(|| format!("Added Document: {}", original_filename));
    let latest = journal::get_latest_journal_entry()?;
    let parent_hash = latest.map(|(_, hash)| hash);
    journal::create_journal_entry_with_documents(
        &body,
        parent_hash,
        vec![DocumentRef {
            path: dest_str.clone(),
            sha256: sha256.clone(),
            original_filename: Some(original_filename),
        }],
    )?;

    println!("Added Document: {}", dest_str);
    println!("SHA-256: {}", sha256);
    Ok(dest_str)
}
