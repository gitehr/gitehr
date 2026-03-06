use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::remote::{add_remote, remove_remote};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    temp_dir
}

#[test]
#[serial]
fn test_add_remote() -> Result<()> {
    let _temp_dir = setup();

    add_remote("origin", "https://example.com/repo.git")?;

    let config_path = Path::new(".gitehr/remotes.json");
    assert!(config_path.exists(), "remotes.json should be created");

    let content = fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(
        config["remotes"]["origin"].is_object(),
        "origin remote should exist"
    );
    assert_eq!(
        config["remotes"]["origin"]["url"],
        "https://example.com/repo.git"
    );

    Ok(())
}

#[test]
#[serial]
fn test_add_multiple_remotes() -> Result<()> {
    let _temp_dir = setup();

    add_remote("origin", "https://example.com/repo.git")?;
    add_remote("backup", "https://backup.com/repo.git")?;

    let content = fs::read_to_string(".gitehr/remotes.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(config["remotes"]["origin"].is_object());
    assert!(config["remotes"]["backup"].is_object());
    assert_eq!(
        config["remotes"]["origin"]["url"],
        "https://example.com/repo.git"
    );
    assert_eq!(
        config["remotes"]["backup"]["url"],
        "https://backup.com/repo.git"
    );

    Ok(())
}

#[test]
#[serial]
fn test_add_duplicate_remote_fails() -> Result<()> {
    let _temp_dir = setup();

    add_remote("origin", "https://example.com/repo.git")?;

    let result = add_remote("origin", "https://different.com/repo.git");
    assert!(result.is_err(), "Adding duplicate remote should fail");

    Ok(())
}

#[test]
#[serial]
fn test_remove_remote() -> Result<()> {
    let _temp_dir = setup();

    add_remote("origin", "https://example.com/repo.git")?;
    remove_remote("origin")?;

    let content = fs::read_to_string(".gitehr/remotes.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(
        config["remotes"]["origin"].is_null(),
        "origin should be removed"
    );

    Ok(())
}

#[test]
#[serial]
fn test_remove_nonexistent_remote_fails() -> Result<()> {
    let _temp_dir = setup();

    let result = remove_remote("nonexistent");
    assert!(result.is_err(), "Removing nonexistent remote should fail");

    Ok(())
}

#[test]
#[serial]
fn test_remote_config_persists() -> Result<()> {
    let _temp_dir = setup();

    add_remote("remote1", "https://example.com/repo1.git")?;
    add_remote("remote2", "https://example.com/repo2.git")?;

    remove_remote("remote1")?;

    let content = fs::read_to_string(".gitehr/remotes.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(config["remotes"]["remote1"].is_null());
    assert!(config["remotes"]["remote2"].is_object());

    Ok(())
}

#[test]
#[serial]
fn test_remote_has_timestamp() -> Result<()> {
    let _temp_dir = setup();

    add_remote("origin", "https://example.com/repo.git")?;

    let content = fs::read_to_string(".gitehr/remotes.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let added_at = &config["remotes"]["origin"]["added_at"];
    assert!(!added_at.is_null(), "added_at should be present");

    let time_str = added_at.as_str().unwrap();
    assert!(time_str.contains("T"), "Should be ISO 8601 format");
    assert!(time_str.contains("Z"), "Should be in UTC");

    Ok(())
}

#[test]
#[serial]
fn test_remote_config_fails_without_gitehr() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = add_remote("origin", "https://example.com/repo.git");
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}
