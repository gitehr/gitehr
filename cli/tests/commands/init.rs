use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::init;

fn setup() -> tempfile::TempDir {
    tempdir().unwrap()
}

#[test]
#[serial]
fn test_initialise_creates_gitehr_directory() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    init::run()?;

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

    init::run()?;

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

    init::run()?;

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

    init::run()?;

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

    init::run()?;

    let state_path = Path::new("state");
    assert!(state_path.exists(), "state directory should exist");
    assert!(state_path.is_dir(), "state should be a directory");

    Ok(())
}

#[test]
#[serial]
fn test_initialise_fails_if_already_initialized() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;

    init::run()?;

    let result = init::run();
    assert!(result.is_err(), "Initializing twice should fail");

    Ok(())
}
