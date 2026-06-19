use crate::utils::sha256_hex;
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::journal::{self, DocumentRef};

pub mod add;
pub mod info;
pub mod list;
pub mod verify;

#[derive(Subcommand)]
pub enum DocumentCommands {
    #[command(about = "Add a Document to the record and link it from a new journal entry")]
    Add {
        #[arg(help = "Path to the file (or directory, e.g. a DICOM study) to add")]
        path: PathBuf,
        #[arg(long, help = "Store under imaging/ instead of documents/")]
        imaging: bool,
        #[arg(
            short,
            long,
            help = "Title used to build the stored filename slug (defaults to the original filename)"
        )]
        title: Option<String>,
        #[arg(short, long, help = "Journal entry text describing the Document")]
        message: Option<String>,
    },
    #[command(about = "List Documents referenced by journal entries")]
    List,
    #[command(about = "Show which journal entries reference a Document")]
    Info {
        #[arg(help = "Path of the Document within the record (e.g. documents/2026-06-12-...pdf)")]
        path: String,
    },
    #[command(about = "Verify Document integrity against the hashes recorded in journal entries")]
    Verify {
        #[arg(help = "Verify a single Document path (default: all)")]
        path: Option<String>,
    },
}

pub fn run(command: DocumentCommands) -> Result<()> {
    match command {
        DocumentCommands::Add {
            path,
            imaging,
            title,
            message,
        } => {
            add::run(
                path.as_path(),
                title.as_deref(),
                imaging,
                message.as_deref(),
            )?;
            Ok(())
        }
        DocumentCommands::List => list::run(),
        DocumentCommands::Info { path } => info::run(&path),
        DocumentCommands::Verify { path } => {
            let ok = verify::run(path.as_deref())?;
            if !ok {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

// ── Shared constants ──────────────────────────────────────────────────────────

/// Documents are plain files in human-readable folders (ADR-0001), immutable
/// and write-once (ADR-0002). A Document is a single file, or a directory
/// anchored by a manifest hashing every file within it (ADR-0003). Journal
/// entries link Documents via `documents:` front matter; there is no stored
/// index - reverse lookup is always derived by scanning the journal.
pub const DOCUMENT_ROOTS: [&str; 2] = ["documents", "imaging"];

pub const MANIFEST_FILENAME: &str = "manifest.json";

// ── Shared data structures ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Manifest {
    /// Relative path (forward slashes) to SHA-256, sorted for determinism.
    pub files: BTreeMap<String, String>,
}

// ── Shared helper functions ───────────────────────────────────────────────────

pub fn ensure_gitehr_repository() -> Result<()> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository. Run 'gitehr init' first.");
    }
    Ok(())
}

pub fn hash_file(path: &Path) -> Result<String> {
    let bytes =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

/// Build a filename slug: lowercase, alphanumeric runs joined by single hyphens.
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_hyphen = true;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if !last_was_hyphen {
            slug.push('-');
            last_was_hyphen = true;
        }
        if slug.len() >= 60 {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "document".to_string()
    } else {
        slug
    }
}

/// Serialize a manifest for the given directory. Returns the exact bytes to
/// write and their SHA-256, which is the Document's anchoring hash.
pub fn build_manifest(dir: &Path) -> Result<(Vec<u8>, String)> {
    let mut files = BTreeMap::new();
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)?
            .to_string_lossy()
            .replace('\\', "/");
        files.insert(rel, hash_file(entry.path())?);
    }
    if files.is_empty() {
        bail!("Directory contains no files: {}", dir.display());
    }
    let manifest = Manifest { files };
    let mut bytes = serde_json::to_vec_pretty(&manifest).context("Failed to serialize manifest")?;
    bytes.push(b'\n');
    let hash = sha256_hex(&bytes);
    Ok((bytes, hash))
}

pub fn copy_tree(source: &Path, dest: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(source)?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), &target)
                .with_context(|| format!("Failed to copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

pub fn check_destination(dest: &Path) -> Result<()> {
    if dest.exists() {
        bail!(
            "Document already exists in the record: {} (Documents are write-once; identical content added today produces the same name)",
            dest.display()
        );
    }
    Ok(())
}

/// All Document references found in journal front matter, with the filename
/// of the entry that holds each reference. This is the derived reverse lookup.
pub fn collect_refs() -> Result<Vec<(String, DocumentRef)>> {
    let mut refs = Vec::new();
    for entry in journal::parsed_entries()? {
        if let Some(documents) = &entry.metadata.documents {
            for doc in documents {
                refs.push((entry.filename.clone(), doc.clone()));
            }
        }
    }
    Ok(refs)
}

/// Files and directories present under the Document roots, as record paths.
/// A directory Document counts as one item; per-folder README.md is layout
/// scaffolding, not a Document.
pub fn files_on_disk() -> Result<Vec<String>> {
    let mut found = Vec::new();
    for root in DOCUMENT_ROOTS {
        let root_path = Path::new(root);
        if !root_path.exists() {
            continue;
        }
        for entry in fs::read_dir(root_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "README.md" {
                continue;
            }
            found.push(format!("{}/{}", root, name));
        }
    }
    found.sort();
    Ok(found)
}

/// Check a directory Document: the manifest must hash to the recorded sha256,
/// every manifest entry must match its file, and no unlisted files may be
/// present (Documents are write-once).
pub fn verify_directory_document(dir: &Path, expected_sha256: &str) -> Result<Vec<String>> {
    let manifest_path = dir.join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Ok(vec![format!("missing {}", MANIFEST_FILENAME)]);
    }

    let manifest_bytes = fs::read(&manifest_path).context("Failed to read manifest")?;
    let mut errors = Vec::new();

    if sha256_hex(&manifest_bytes) != expected_sha256 {
        errors.push("manifest hash does not match the hash recorded in the journal".to_string());
    }

    let manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).context("Failed to parse manifest")?;

    for (rel, expected) in &manifest.files {
        let file_path = dir.join(rel);
        if !file_path.exists() {
            errors.push(format!("{} listed in manifest but missing", rel));
        } else if &hash_file(&file_path)? != expected {
            errors.push(format!("{} hash mismatch", rel));
        }
    }

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)?
            .to_string_lossy()
            .replace('\\', "/");
        if rel == MANIFEST_FILENAME {
            continue;
        }
        if !manifest.files.contains_key(&rel) {
            errors.push(format!(
                "{} present but not in manifest (Documents are write-once)",
                rel
            ));
        }
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("CT Head Report"), "ct-head-report");
        assert_eq!(slugify("Scan0001.PDF"), "scan0001-pdf");
        assert_eq!(
            slugify("discharge_summary (final)"),
            "discharge-summary-final"
        );
    }

    #[test]
    fn test_slugify_degenerate() {
        assert_eq!(slugify("???"), "document");
        assert_eq!(slugify(""), "document");
        assert_eq!(slugify("--a--"), "a");
    }

    #[test]
    fn test_slugify_truncates() {
        let long = "x".repeat(200);
        assert!(slugify(&long).len() <= 60);
    }
}
