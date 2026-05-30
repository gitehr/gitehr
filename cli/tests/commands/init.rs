use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::init::initialise;

fn setup() -> tempfile::TempDir {
    tempdir().unwrap()
}

#[test]
#[serial]
fn test_initialise_creates_gitehr_directory() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let gitehr_path = Path::new(".gitehr");
    assert!(gitehr_path.exists(), ".gitehr directory should exist");
    assert!(gitehr_path.is_dir(), ".gitehr should be a directory");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_creates_version_file() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let version_file = Path::new(".gitehr/GITEHR_VERSION");
    assert!(version_file.exists(), "GITEHR_VERSION file should exist");

    let content = fs::read_to_string(version_file)?;
    assert!(!content.trim().is_empty(), "Version should not be empty");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_bundles_binary() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let binary_path = Path::new(".gitehr/gitehr");
    assert!(binary_path.exists(), ".gitehr/gitehr binary should exist");
    assert!(binary_path.is_file(), ".gitehr/gitehr should be a file");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_creates_journal_directory() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let journal_path = Path::new("journal");
    assert!(journal_path.exists(), "journal directory should exist");
    assert!(journal_path.is_dir(), "journal should be a directory");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_creates_state_directory() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let state_path = Path::new("state");
    assert!(state_path.exists(), "state directory should exist");
    assert!(state_path.is_dir(), "state should be a directory");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_creates_genesis_entry() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let journal_entries: Vec<_> = fs::read_dir("journal")?.filter_map(|e| e.ok()).collect();

    assert!(
        journal_entries.len() >= 1,
        "Should have created at least one entry"
    );

    let mut found_genesis = false;
    for entry_file in journal_entries {
        let content = fs::read_to_string(entry_file.path())?;
        if content.contains("Genesis entry") {
            found_genesis = true;
            break;
        }
    }

    assert!(
        found_genesis,
        "Should have genesis entry with 'Genesis entry' text"
    );

    Ok(())
}

#[test]
#[serial]
fn test_initialise_fails_if_already_initialized() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    initialise()?;

    let result = initialise();
    assert!(result.is_err(), "Initializing twice should fail");

    Ok(())
}
