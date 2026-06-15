use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;

use gitehr::commands::document::{add_document, verify_documents, MANIFEST_FILENAME};
use gitehr::commands::journal::parsed_entries;

fn setup_with_git() -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir()?;
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir(".gitehr")?;
    fs::create_dir("journal")?;
    fs::create_dir("documents")?;
    fs::create_dir("imaging")?;
    std::process::Command::new("git").args(["init"]).output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .output()?;
    Ok(temp_dir)
}

#[test]
#[serial]
fn test_add_file_document() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::write("Scan0001.pdf", b"%PDF-1.4 fake content")?;
    let stored = add_document(Path::new("Scan0001.pdf"), None, false, None)?;

    assert!(stored.starts_with("documents/"));
    assert!(stored.contains("-scan0001-"));
    assert!(stored.ends_with(".pdf"));
    assert_eq!(fs::read(&stored)?, b"%PDF-1.4 fake content");

    // The journal entry created alongside must reference the Document.
    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    let docs = entries[0].metadata.documents.as_ref().expect("documents front matter");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].path, stored);
    assert_eq!(docs[0].sha256.len(), 64);
    assert_eq!(docs[0].original_filename.as_deref(), Some("Scan0001.pdf"));
    // The hash8 in the filename comes from the recorded sha256.
    assert!(stored.contains(&docs[0].sha256[..8]));

    Ok(())
}

#[test]
#[serial]
fn test_add_file_with_title_and_imaging() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::write("IMG_4521.jpg", b"\xFF\xD8 fake jpeg")?;
    let stored = add_document(
        Path::new("IMG_4521.jpg"),
        Some("Left knee photograph"),
        true,
        Some("Clinical photo taken in clinic"),
    )?;

    assert!(stored.starts_with("imaging/"));
    assert!(stored.contains("-left-knee-photograph-"));

    let entries = parsed_entries()?;
    assert_eq!(entries[0].content, "Clinical photo taken in clinic");

    Ok(())
}

#[test]
#[serial]
fn test_add_directory_document_builds_manifest() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::create_dir_all("study/series1")?;
    fs::write("study/series1/img001.dcm", b"dicom one")?;
    fs::write("study/series1/img002.dcm", b"dicom two")?;
    fs::write("study/study-info.txt", b"CT head")?;

    let stored = add_document(Path::new("study"), Some("CT head"), true, None)?;

    assert!(stored.starts_with("imaging/"));
    let manifest_path = Path::new(&stored).join(MANIFEST_FILENAME);
    assert!(manifest_path.exists(), "manifest should be written");

    let manifest: serde_json::Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    let files = manifest["files"].as_object().unwrap();
    assert_eq!(files.len(), 3);
    assert!(files.contains_key("series1/img001.dcm"));

    // The recorded sha256 anchors the manifest bytes themselves.
    let entries = parsed_entries()?;
    let docs = entries[0].metadata.documents.as_ref().unwrap();
    let manifest_bytes = fs::read(&manifest_path)?;
    let manifest_hash = gitehr::utils::sha256_hex(&manifest_bytes);
    assert_eq!(docs[0].sha256, manifest_hash);

    assert!(verify_documents(None)?);
    Ok(())
}

#[test]
#[serial]
fn test_verify_detects_tampering() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::write("letter.txt", b"original referral letter")?;
    let stored = add_document(Path::new("letter.txt"), None, false, None)?;

    assert!(verify_documents(None)?);

    fs::write(&stored, b"tampered content")?;
    assert!(!verify_documents(None)?, "tampering must fail verification");

    Ok(())
}

#[test]
#[serial]
fn test_verify_detects_file_added_to_directory_document() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::create_dir("study")?;
    fs::write("study/img.dcm", b"dicom")?;
    let stored = add_document(Path::new("study"), None, true, None)?;

    assert!(verify_documents(None)?);

    // Documents are write-once: a file smuggled in afterwards must fail.
    fs::write(Path::new(&stored).join("extra.dcm"), b"late addition")?;
    assert!(!verify_documents(None)?);

    Ok(())
}

#[test]
#[serial]
fn test_missing_document_is_not_a_failure() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::write("note.txt", b"to be removed")?;
    let stored = add_document(Path::new("note.txt"), None, false, None)?;

    // Deletion only touches the working tree (ADR-0002) and is not an
    // integrity failure.
    fs::remove_file(&stored)?;
    assert!(verify_documents(None)?);

    Ok(())
}

#[test]
#[serial]
fn test_duplicate_add_is_rejected() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    fs::write("dup.txt", b"same bytes")?;
    add_document(Path::new("dup.txt"), None, false, None)?;
    let result = add_document(Path::new("dup.txt"), None, false, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("write-once"));

    Ok(())
}

#[test]
#[serial]
fn test_add_fails_outside_repository() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::env::set_current_dir(&temp_dir)?;
    fs::write("orphan.txt", b"no repo here")?;

    let result = add_document(Path::new("orphan.txt"), None, false, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Not in a gitehr repository"));

    Ok(())
}
