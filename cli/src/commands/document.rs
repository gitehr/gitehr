use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::journal::{self, DocumentRef};

/// Documents are plain files in human-readable folders (ADR-0001), immutable
/// and write-once (ADR-0002). A Document is a single file, or a directory
/// anchored by a manifest hashing every file within it (ADR-0003). Journal
/// entries link Documents via `documents:` front matter; there is no stored
/// index - reverse lookup is always derived by scanning the journal.
const DOCUMENT_ROOTS: [&str; 2] = ["documents", "imaging"];

pub const MANIFEST_FILENAME: &str = "manifest.json";

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    /// Relative path (forward slashes) to SHA-256, sorted for determinism.
    files: BTreeMap<String, String>,
}

fn ensure_gitehr_repository() -> Result<()> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository. Run 'gitehr init' first.");
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hash_file(path: &Path) -> Result<String> {
    let bytes =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

/// Build a filename slug: lowercase, alphanumeric runs joined by single hyphens.
fn slugify(name: &str) -> String {
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
fn build_manifest(dir: &Path) -> Result<(Vec<u8>, String)> {
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

fn copy_tree(source: &Path, dest: &Path) -> Result<()> {
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

/// Add a file or directory to the record as a Document and link it from a new
/// journal entry. Returns the stored path within the record.
pub fn add_document(
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
    super::git::git_add(&dest_str)?;

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

fn check_destination(dest: &Path) -> Result<()> {
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
fn collect_refs() -> Result<Vec<(String, DocumentRef)>> {
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
fn files_on_disk() -> Result<Vec<String>> {
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

pub fn list_documents() -> Result<()> {
    ensure_gitehr_repository()?;

    let refs = collect_refs()?;
    let mut by_path: BTreeMap<String, (DocumentRef, Vec<String>)> = BTreeMap::new();
    for (entry_file, doc) in refs {
        by_path
            .entry(doc.path.clone())
            .or_insert_with(|| (doc, Vec::new()))
            .1
            .push(entry_file);
    }

    if by_path.is_empty() {
        println!("No Documents referenced by journal entries.");
    } else {
        println!("Documents ({} total):", by_path.len());
        for (path, (doc, entries)) in &by_path {
            println!();
            println!("  {}", path);
            println!("    SHA-256: {}", doc.sha256);
            if let Some(original) = &doc.original_filename {
                println!("    Original filename: {}", original);
            }
            println!("    Referenced by {} journal entr{}", entries.len(), if entries.len() == 1 { "y" } else { "ies" });
        }
    }

    let unreferenced: Vec<String> = files_on_disk()?
        .into_iter()
        .filter(|p| !by_path.contains_key(p))
        .collect();
    if !unreferenced.is_empty() {
        println!();
        println!("Unreferenced (present on disk, not linked from any journal entry):");
        for path in unreferenced {
            println!("  {}", path);
        }
    }

    Ok(())
}

pub fn document_info(path: &str) -> Result<()> {
    ensure_gitehr_repository()?;

    let path = path.trim_end_matches('/');
    let refs: Vec<(String, DocumentRef)> = collect_refs()?
        .into_iter()
        .filter(|(_, doc)| doc.path == path)
        .collect();

    if refs.is_empty() {
        if Path::new(path).exists() {
            println!("{} exists on disk but is not referenced by any journal entry.", path);
            return Ok(());
        }
        bail!("No Document found at {}", path);
    }

    let doc = &refs[0].1;
    println!("Document: {}", path);
    println!("SHA-256: {}", doc.sha256);
    if let Some(original) = &doc.original_filename {
        println!("Original filename: {}", original);
    }
    if !Path::new(path).exists() {
        println!("Status: removed from working tree (retained in Git history)");
    }
    println!("Referenced by:");
    for (entry_file, _) in &refs {
        println!("  journal/{}", entry_file);
    }
    Ok(())
}

/// Verify every Document reference in the journal (or just `filter`).
/// A reference whose target was removed from the working tree is reported but
/// is not a failure - deletion only ever touches the working tree (ADR-0002).
/// Returns true if no integrity failures were found.
pub fn verify_documents(filter: Option<&str>) -> Result<bool> {
    ensure_gitehr_repository()?;

    let mut refs = collect_refs()?;
    if let Some(filter) = filter {
        let filter = filter.trim_end_matches('/');
        refs.retain(|(_, doc)| doc.path == filter);
        if refs.is_empty() {
            bail!("No journal entry references a Document at {}", filter);
        }
    }

    // The same (path, sha256) pair may be referenced by many entries but only
    // needs checking once.
    let mut checked = std::collections::BTreeSet::new();
    let mut ok = 0usize;
    let mut missing = 0usize;
    let mut failures = Vec::new();

    for (_, doc) in &refs {
        if !checked.insert((doc.path.clone(), doc.sha256.clone())) {
            continue;
        }
        let path = Path::new(&doc.path);
        if !path.exists() {
            println!("MISSING  {} (removed from working tree; retained in Git history)", doc.path);
            missing += 1;
        } else if path.is_dir() {
            match verify_directory_document(path, &doc.sha256) {
                Ok(errors) if errors.is_empty() => {
                    println!("OK       {}", doc.path);
                    ok += 1;
                }
                Ok(errors) => {
                    for e in errors {
                        println!("FAILED   {}: {}", doc.path, e);
                        failures.push(doc.path.clone());
                    }
                }
                Err(e) => {
                    println!("FAILED   {}: {}", doc.path, e);
                    failures.push(doc.path.clone());
                }
            }
        } else {
            let actual = hash_file(path)?;
            if actual == doc.sha256 {
                println!("OK       {}", doc.path);
                ok += 1;
            } else {
                println!("FAILED   {}: hash mismatch (expected {}, found {})", doc.path, doc.sha256, actual);
                failures.push(doc.path.clone());
            }
        }
    }

    println!();
    println!(
        "Checked {} Document reference{}: {} ok, {} missing from working tree, {} failed",
        checked.len(),
        if checked.len() == 1 { "" } else { "s" },
        ok,
        missing,
        failures.len()
    );

    Ok(failures.is_empty())
}

/// Check a directory Document: the manifest must hash to the recorded sha256,
/// every manifest entry must match its file, and no unlisted files may be
/// present (Documents are write-once).
fn verify_directory_document(dir: &Path, expected_sha256: &str) -> Result<Vec<String>> {
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
            errors.push(format!("{} present but not in manifest (Documents are write-once)", rel));
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
        assert_eq!(slugify("discharge_summary (final)"), "discharge-summary-final");
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
