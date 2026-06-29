// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    MANIFEST_FILENAME, build_manifest, check_destination, copy_tree, ensure_gitehr_repository,
    hash_file, slugify,
};
use crate::commands::journal::{self, DocumentRef};

#[derive(Debug, Clone)]
pub struct DocumentSource {
    pub path: PathBuf,
    pub imaging: bool,
    pub title: Option<String>,
}

#[derive(Debug)]
struct StoredDocument {
    path: String,
    sha256: String,
    original_filename: String,
}

/// Add a file or directory to the record as a Document and link it from a new
/// journal entry. Returns the stored path within the record.
#[allow(dead_code)]
pub fn run(
    source: &Path,
    title: Option<&str>,
    imaging: bool,
    message: Option<&str>,
) -> Result<String> {
    let stored_paths = run_many_with_sources(
        &[DocumentSource {
            path: source.to_path_buf(),
            imaging,
            title: title.map(str::to_string),
        }],
        message,
    )?;

    Ok(stored_paths
        .into_iter()
        .next()
        .expect("single-source document add must return one stored path"))
}

/// Add one or more files or directories to the record as Documents and link
/// them from a single new journal entry. Returns the stored paths within the
/// record.
pub fn run_many(
    sources: &[PathBuf],
    title: Option<&str>,
    imaging: bool,
    message: Option<&str>,
) -> Result<Vec<String>> {
    if sources.len() > 1 && title.is_some() {
        bail!("--title can only be used when adding one Document");
    }

    let document_sources: Vec<DocumentSource> = sources
        .iter()
        .map(|source| DocumentSource {
            path: source.clone(),
            imaging,
            title: title.map(str::to_string),
        })
        .collect();

    run_many_with_sources(&document_sources, message)
}

/// Add one or more Documents with per-source storage options and link them from
/// a single new journal entry.
pub fn run_many_with_sources(
    sources: &[DocumentSource],
    message: Option<&str>,
) -> Result<Vec<String>> {
    ensure_gitehr_repository()?;

    if sources.is_empty() {
        bail!("At least one Document path is required");
    }

    for source in sources {
        if !source.path.exists() {
            bail!("No such file or directory: {}", source.path.display());
        }
    }

    let mut stored_documents = Vec::with_capacity(sources.len());
    for source in sources {
        let stored = store_document(source)?;
        crate::commands::git::git_add(&stored.path)?;
        stored_documents.push(stored);
    }

    let body = message
        .map(|m| m.to_string())
        .unwrap_or_else(|| default_journal_body(&stored_documents));
    let document_refs = stored_documents
        .iter()
        .map(|document| DocumentRef {
            path: document.path.clone(),
            sha256: document.sha256.clone(),
            original_filename: Some(document.original_filename.clone()),
        })
        .collect();

    journal::create_journal_entry_with_documents(&body, document_refs)?;

    for document in &stored_documents {
        println!("Added Document: {}", document.path);
        println!("SHA-256: {}", document.sha256);
    }

    Ok(stored_documents
        .into_iter()
        .map(|document| document.path)
        .collect())
}

fn store_document(source: &DocumentSource) -> Result<StoredDocument> {
    let original_filename = source
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let slug = slugify(source.title.as_deref().unwrap_or_else(|| {
        source
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
    }));
    let date = Utc::now().format("%Y-%m-%d");
    let root = if source.imaging {
        "imaging"
    } else {
        "documents"
    };
    fs::create_dir_all(root)?;

    let (dest, sha256) = if source.path.is_dir() {
        if source.path.join(MANIFEST_FILENAME).exists() {
            bail!(
                "Directory already contains a {} - this name is reserved for the Document manifest",
                MANIFEST_FILENAME
            );
        }
        let (manifest_bytes, manifest_hash) = build_manifest(&source.path)?;
        let dest = PathBuf::from(root).join(format!("{}-{}-{}", date, slug, &manifest_hash[..8]));
        check_destination(&dest)?;
        copy_tree(&source.path, &dest)?;
        fs::write(dest.join(MANIFEST_FILENAME), &manifest_bytes)
            .context("Failed to write manifest")?;
        (dest, manifest_hash)
    } else {
        let hash = hash_file(&source.path)?;
        let ext = source
            .path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_ascii_lowercase()))
            .unwrap_or_default();
        let dest = PathBuf::from(root).join(format!("{}-{}-{}{}", date, slug, &hash[..8], ext));
        check_destination(&dest)?;
        fs::copy(&source.path, &dest)
            .with_context(|| format!("Failed to copy {} into the record", source.path.display()))?;
        (dest, hash)
    };

    let dest_str = dest.to_string_lossy().to_string();

    Ok(StoredDocument {
        path: dest_str,
        sha256,
        original_filename,
    })
}

fn default_journal_body(documents: &[StoredDocument]) -> String {
    match documents {
        [document] => format!("Added Document: {}", document.original_filename),
        _ => {
            let filenames = documents
                .iter()
                .map(|document| document.original_filename.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("Added Documents: {}", filenames)
        }
    }
}
