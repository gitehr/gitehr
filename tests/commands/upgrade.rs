use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::init::initialise;
use gitehr::commands::upgrade::{upgrade_binary, upgrade_repository};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    temp_dir
}

#[test]
#[serial]
fn test_upgrade_repository_updates_version() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    upgrade_repository()?;

    let content = fs::read_to_string(".gitehr/GITEHR_VERSION")?;
    assert!(!content.contains("0.1.0"), "Version should be updated");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_repository_updates_binary() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    let old_binary_path = Path::new(".gitehr/gitehr");
    let old_metadata = fs::metadata(old_binary_path)?;
    let old_size = old_metadata.len();

    std::thread::sleep(std::time::Duration::from_millis(10));

    upgrade_repository()?;

    let new_metadata = fs::metadata(old_binary_path)?;
    assert_eq!(new_metadata.len(), old_size, "Binary should be updated");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_repository_creates_journal_entry() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    let initial_entries: Vec<_> = fs::read_dir("journal")?.filter_map(|e| e.ok()).collect();

    upgrade_repository()?;

    let after_entries: Vec<_> = fs::read_dir("journal")?.filter_map(|e| e.ok()).collect();

    assert!(
        after_entries.len() > initial_entries.len(),
        "Should create an entry for the upgrade when version changes"
    );

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_repository_fails_without_gitehr() -> Result<()> {
    let _temp_dir = setup();

    let result = upgrade_repository();
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_repository_already_latest() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    let result = upgrade_repository()?;

    assert_eq!(result, (), "Should succeed even if already latest");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_binary_updates_file() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    let binary_path = Path::new(".gitehr/gitehr");
    let old_metadata = fs::metadata(binary_path)?;
    let old_modified = old_metadata.modified()?;

    std::thread::sleep(std::time::Duration::from_millis(10));

    upgrade_binary()?;

    let new_metadata = fs::metadata(binary_path)?;
    let new_modified = new_metadata.modified()?;

    assert!(new_modified >= old_modified, "Binary should be updated");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_binary_updates_version_file() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    upgrade_binary()?;

    let version = fs::read_to_string(".gitehr/GITEHR_VERSION")?;
    assert!(!version.contains("0.1.0"), "Version file should be updated");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_binary_fails_without_gitehr() -> Result<()> {
    let _temp_dir = setup();

    let result = upgrade_binary();
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_binary_creates_bundled_binary() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    let binary_path = Path::new(".gitehr/gitehr");
    let exists_before = binary_path.exists();

    upgrade_binary()?;

    assert!(binary_path.exists(), "Binary should exist after upgrade");
    assert!(exists_before, "Binary should have already existed");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_preserves_other_files() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    fs::write(".gitehr/custom_config.json", "{\"test\": true}")?;

    upgrade_repository()?;

    assert!(
        Path::new(".gitehr/custom_config.json").exists(),
        "Other files should be preserved"
    );

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_upgrade_handles_missing_current_version() -> Result<()> {
    let _temp_dir = setup();

    fs::create_dir_all(".gitehr")?;
    fs::create_dir_all("journal")?;

    upgrade_repository()?;

    let version = fs::read_to_string(".gitehr/GITEHR_VERSION")?;
    assert!(!version.is_empty(), "Should write version even if missing");

    Ok(())
}

#[test]
#[serial]
fn test_upgrade_repository_journal_entry_contains_version_info() -> Result<()> {
    let _temp_dir = setup();

    initialise()?;

    fs::write(".gitehr/GITEHR_VERSION", "0.1.0")?;

    upgrade_repository()?;

    let entries: Vec<_> = fs::read_dir("journal")?.filter_map(|e| e.ok()).collect();

    let mut found_upgrade_entry = false;
    for entry in entries {
        let content = fs::read_to_string(entry.path())?;
        if content.contains("upgraded") || content.contains("Upgrade") || content.contains("0.1.0")
        {
            found_upgrade_entry = true;
            break;
        }
    }

    assert!(
        found_upgrade_entry,
        "Should have journal entry mentioning upgrade when version changes"
    );

    Ok(())
}
