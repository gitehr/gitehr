use anyhow::Result;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

use gitehr::commands::status::RepoStatus;

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    temp_dir
}

#[test]
#[serial]
fn test_gather_non_gitehr_repo() -> Result<()> {
    let _temp_dir = setup();

    let status = RepoStatus::gather()?;

    assert!(!status.is_gitehr_repo, "Should not be a GitEHR repo");
    assert!(status.gitehr_version.is_none());
    assert_eq!(status.journal_entry_count, 0);
    assert_eq!(status.state_files.len(), 0);
    assert!(!status.has_uncommitted_changes);
    assert!(!status.is_encrypted);

    Ok(())
}

#[test]
#[serial]
fn test_gather_gitehr_repo_with_version() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.5")?;

    let status = RepoStatus::gather()?;

    assert!(status.is_gitehr_repo, "Should be a GitEHR repo");
    assert_eq!(
        status.gitehr_version,
        Some("0.1.5".to_string()),
        "Version should match"
    );

    Ok(())
}

#[test]
#[serial]
fn test_gather_counts_journal_entries() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::create_dir_all("journal")?;

    fs::write("journal/entry1.md", "content1")?;
    fs::write("journal/entry2.md", "content2")?;
    fs::write("journal/entry3.md", "content3")?;

    let status = RepoStatus::gather()?;

    assert_eq!(
        status.journal_entry_count, 3,
        "Should count all journal entries"
    );

    Ok(())
}

#[test]
#[serial]
fn test_gather_lists_state_files() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::create_dir_all("state")?;

    fs::write("state/config.txt", "configuration")?;
    fs::write("state/notes.txt", "clinical notes")?;

    let status = RepoStatus::gather()?;

    assert_eq!(status.state_files.len(), 2, "Should list all state files");
    assert!(status.state_files.contains(&"config.txt".to_string()));
    assert!(status.state_files.contains(&"notes.txt".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_gather_detects_encryption() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::write(".gitehr/ENCRYPTED", "encrypted_at=2024-01-01T00:00:00Z")?;

    let status = RepoStatus::gather()?;

    assert!(status.is_encrypted, "Should detect encrypted repository");

    Ok(())
}

#[test]
#[serial]
fn test_gather_detects_unencrypted() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    let status = RepoStatus::gather()?;

    assert!(!status.is_encrypted, "Should not be encrypted");

    Ok(())
}

#[test]
#[serial]
fn test_gather_with_empty_directories() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::create_dir_all("journal")?;
    fs::create_dir_all("state")?;

    let status = RepoStatus::gather()?;

    assert!(status.is_gitehr_repo);
    assert_eq!(status.journal_entry_count, 0);
    assert_eq!(status.state_files.len(), 0);

    Ok(())
}

#[test]
#[serial]
fn test_gather_excludes_readme_from_state() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::create_dir_all("state")?;

    fs::write("state/README.md", "This is documentation")?;
    fs::write("state/config.txt", "configuration")?;

    let status = RepoStatus::gather()?;

    assert_eq!(
        status.state_files.len(),
        1,
        "README.md should not be counted"
    );
    assert!(status.state_files.contains(&"config.txt".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_gather_full_status() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.7")?;
    fs::create_dir_all("journal")?;
    fs::create_dir_all("state")?;

    fs::write("journal/entry1.md", "Journal 1")?;
    fs::write("journal/entry2.md", "Journal 2")?;
    fs::write("state/patient_info.txt", "Patient data")?;

    let status = RepoStatus::gather()?;

    assert!(status.is_gitehr_repo);
    assert_eq!(status.gitehr_version, Some("0.1.7".to_string()));
    assert_eq!(status.journal_entry_count, 2);
    assert_eq!(status.state_files.len(), 1);
    assert!(!status.is_encrypted);

    Ok(())
}

#[test]
#[serial]
fn test_gather_ignores_non_md_files_in_journal() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;
    fs::create_dir_all("journal")?;

    fs::write("journal/entry1.md", "Valid entry")?;
    fs::write("journal/entry2.txt", "Not a markdown file")?;
    fs::write("journal/README.md", "Documentation")?;

    let status = RepoStatus::gather()?;

    assert_eq!(status.journal_entry_count, 2, "Should count all .md files");

    Ok(())
}

#[test]
#[serial]
fn test_gather_no_version_file() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;

    let status = RepoStatus::gather()?;

    assert!(status.is_gitehr_repo);
    assert!(
        status.gitehr_version.is_none(),
        "Version should be None if file missing"
    );

    Ok(())
}
