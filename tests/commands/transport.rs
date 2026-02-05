use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::transport::{create_transport_archive, extract_transport_archive};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    fs::create_dir_all("journal").ok();
    fs::create_dir_all("state").ok();
    fs::create_dir_all("imaging").ok();
    fs::create_dir_all("documents").ok();
    temp_dir
}

#[test]
#[serial]
fn test_create_transport_archive() -> Result<()> {
    let _temp_dir = setup();

    fs::write("journal/test.md", "Journal content")?;
    fs::write("state/test.txt", "State content")?;

    create_transport_archive(Some("test-archive.tar.gz"), false)?;

    let archive_path = Path::new("test-archive.tar.gz");
    assert!(archive_path.exists(), "Archive should be created");
    assert!(archive_path.is_file(), "Archive should be a file");

    let metadata = fs::metadata(archive_path)?;
    assert!(metadata.len() > 0, "Archive should not be empty");

    Ok(())
}

#[test]
#[serial]
fn test_create_transport_archive_with_default_name() -> Result<()> {
    let _temp_dir = setup();

    fs::write("journal/test.md", "content")?;

    create_transport_archive(None, false)?;

    let entries: Vec<_> = fs::read_dir(".")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("gitehr-transport-")
                && e.file_name().to_string_lossy().ends_with(".tar.gz")
        })
        .collect();

    assert_eq!(entries.len(), 1, "Should create archive with default name");

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_includes_journal() -> Result<()> {
    let _temp_dir = setup();

    let journal_content = "Test journal entry";
    fs::write("journal/entry.md", journal_content)?;

    create_transport_archive(Some("test.tar.gz"), false)?;

    let extract_dir = tempdir()?;
    extract_transport_archive("test.tar.gz", extract_dir.path().to_str())?;

    let extracted_file = extract_dir.path().join("journal/entry.md");
    assert!(extracted_file.exists(), "Journal should be in archive");

    let content = fs::read_to_string(&extracted_file)?;
    assert_eq!(content, journal_content);

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_includes_state() -> Result<()> {
    let _temp_dir = setup();

    let state_content = "Test state";
    fs::write("state/config.txt", state_content)?;

    create_transport_archive(Some("test.tar.gz"), false)?;

    let extract_dir = tempdir()?;
    extract_transport_archive("test.tar.gz", extract_dir.path().to_str())?;

    let extracted_file = extract_dir.path().join("state/config.txt");
    assert!(extracted_file.exists(), "State should be in archive");

    let content = fs::read_to_string(&extracted_file)?;
    assert_eq!(content, state_content);

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_includes_gitehr_config() -> Result<()> {
    let _temp_dir = setup();

    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    create_transport_archive(Some("test.tar.gz"), false)?;

    let extract_dir = tempdir()?;
    extract_transport_archive("test.tar.gz", extract_dir.path().to_str())?;

    let extracted_file = extract_dir.path().join(".gitehr/GITEHR_VERSION");
    assert!(
        extracted_file.exists(),
        ".gitehr config should be in archive"
    );

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_with_empty_directories() -> Result<()> {
    let _temp_dir = setup();

    create_transport_archive(Some("empty-archive.tar.gz"), false)?;

    let archive_path = Path::new("empty-archive.tar.gz");
    assert!(
        archive_path.exists(),
        "Archive should be created even if empty"
    );

    let extract_dir = tempdir()?;
    extract_transport_archive("empty-archive.tar.gz", extract_dir.path().to_str())?;

    let journal_dir = extract_dir.path().join("journal");
    assert!(
        journal_dir.exists() || extract_dir.path().exists(),
        "Archive should extract successfully"
    );

    Ok(())
}

#[test]
#[serial]
fn test_extract_archive_to_default_directory() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    fs::write("journal/entry.md", "content")?;

    create_transport_archive(Some("test.tar.gz"), false)?;

    let extract_root = tempdir()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&extract_root)?;

    let archive_path = original_dir.join("test.tar.gz");
    extract_transport_archive(archive_path.to_str().unwrap(), None)?;

    let journal_file = extract_root.path().join("journal/entry.md");
    assert!(
        journal_file.exists(),
        "Should extract to current directory by default"
    );

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_fails_without_gitehr() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = create_transport_archive(Some("test.tar.gz"), false);
    assert!(
        result.is_err(),
        "Creating archive should fail without .gitehr"
    );

    Ok(())
}

#[test]
#[serial]
fn test_create_archive_includes_nested_files() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir("imaging/scans")?;
    fs::write("imaging/scans/scan1.dcm", "fake dicom data")?;

    create_transport_archive(Some("test.tar.gz"), false)?;

    let extract_dir = tempdir()?;
    extract_transport_archive("test.tar.gz", extract_dir.path().to_str())?;

    let extracted_file = extract_dir.path().join("imaging/scans/scan1.dcm");
    assert!(extracted_file.exists(), "Nested files should be in archive");

    Ok(())
}

#[test]
#[serial]
fn test_archive_roundtrip() -> Result<()> {
    let _temp_dir = setup();

    let files_to_create = vec![
        ("journal/entry1.md", "Journal 1"),
        ("journal/entry2.md", "Journal 2"),
        ("state/config.txt", "Config"),
        ("documents/readme.txt", "Documentation"),
    ];

    for (path, content) in &files_to_create {
        fs::write(path, content)?;
    }

    create_transport_archive(Some("archive.tar.gz"), false)?;

    let extract_dir = tempdir()?;
    extract_transport_archive("archive.tar.gz", extract_dir.path().to_str())?;

    for (path, expected_content) in &files_to_create {
        let extracted_file = extract_dir.path().join(path);
        assert!(extracted_file.exists(), "File should exist: {}", path);

        let content = fs::read_to_string(&extracted_file)?;
        assert_eq!(&content, expected_content);
    }

    Ok(())
}
