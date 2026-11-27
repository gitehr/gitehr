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
use crate::commands::git;
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

    // Plan every Document before writing anything, so a bad source (an empty
    // directory, a name collision, the same content listed twice) fails the
    // whole add up front instead of leaving earlier Documents behind.
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let mut planned: Vec<PlannedDocument> = Vec::with_capacity(sources.len());
    for source in sources {
        let plan = plan_document(source, &date)?;
        if let Some(earlier) = planned.iter().find(|p| p.dest == plan.dest) {
            bail!(
                "{} and {} would both be stored as {} - the same content cannot be added twice (Documents are write-once)",
                earlier.source.display(),
                plan.source.display(),
                plan.dest.display()
            );
        }
        planned.push(plan);
    }

    let stored_documents: Vec<StoredDocument> = planned
        .iter()
        .map(|plan| StoredDocument {
            path: plan.dest.to_string_lossy().to_string(),
            sha256: plan.sha256.clone(),
            original_filename: plan.original_filename.clone(),
        })
        .collect();
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
    let stored_paths: Vec<&str> = stored_documents
        .iter()
        .map(|document| document.path.as_str())
        .collect();

    // Write and stage the Documents, then commit them with their journal entry
    // as one isolated commit. The journal helper removes its own entry if the
    // commit fails; on any failure, also remove the Documents this call wrote.
    let mut written: Vec<&str> = Vec::new();
    let result = (|| -> Result<()> {
        for (plan, path) in planned.iter().zip(&stored_paths) {
            // Re-check immediately before writing: never write over, or roll
            // back, a path this call did not create.
            check_destination(&plan.dest)?;
            written.push(path);
            write_document(plan)?;
            git::git_add(path)?;
        }
        journal::create_journal_entry_with_documents(&body, document_refs, &stored_paths)
    })();
    if let Err(error) = result {
        return Err(git::roll_back_new_paths(&written, error));
    }

    for document in &stored_documents {
        println!("Added Document: {}", document.path);
        println!("SHA-256: {}", document.sha256);
    }

    Ok(stored_documents
        .into_iter()
        .map(|document| document.path)
        .collect())
}

/// A Document whose hash and destination are known but which has not yet been
/// written into the record.
struct PlannedDocument {
    source: PathBuf,
    dest: PathBuf,
    sha256: String,
    original_filename: String,
    /// The manifest to write for a directory Document; `None` for a file.
    manifest: Option<Vec<u8>>,
}

fn plan_document(source: &DocumentSource, date: &str) -> Result<PlannedDocument> {
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
    let root = if source.imaging {
        "imaging"
    } else {
        "documents"
    };

    let (dest, sha256, manifest) = if source.path.is_dir() {
        if source.path.join(MANIFEST_FILENAME).exists() {
            bail!(
                "Directory already contains a {} - this name is reserved for the Document manifest",
                MANIFEST_FILENAME
            );
        }
        let (manifest_bytes, manifest_hash) = build_manifest(&source.path)?;
        let dest = PathBuf::from(root).join(format!("{}-{}-{}", date, slug, &manifest_hash[..8]));
        (dest, manifest_hash, Some(manifest_bytes))
    } else {
        let hash = hash_file(&source.path)?;
        let ext = source
            .path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_ascii_lowercase()))
            .unwrap_or_default();
        let dest = PathBuf::from(root).join(format!("{}-{}-{}{}", date, slug, &hash[..8], ext));
        (dest, hash, None)
    };
    check_destination(&dest)?;

    Ok(PlannedDocument {
        source: source.path.clone(),
        dest,
        sha256,
        original_filename,
        manifest,
    })
}

fn write_document(plan: &PlannedDocument) -> Result<()> {
    if let Some(parent) = plan.dest.parent() {
        fs::create_dir_all(parent)?;
    }
    match &plan.manifest {
        Some(manifest_bytes) => {
            copy_tree(&plan.source, &plan.dest)?;
            fs::write(plan.dest.join(MANIFEST_FILENAME), manifest_bytes)
                .context("Failed to write manifest")?;
        }
        None => {
            fs::copy(&plan.source, &plan.dest).with_context(|| {
                format!("Failed to copy {} into the record", plan.source.display())
            })?;
        }
    }
    Ok(())
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
