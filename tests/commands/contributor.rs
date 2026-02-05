use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::contributor::{
    activate_contributor, add_contributor, deactivate_contributor, disable_contributor,
    enable_contributor, get_current_contributor,
};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    temp_dir
}

#[test]
#[serial]
fn test_add_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor(
        "doc001",
        "Dr. Smith",
        Some("Physician"),
        Some("smith@example.com"),
    )?;

    let config_path = Path::new(".gitehr/contributors.json");
    assert!(config_path.exists(), "contributors.json should be created");

    let content = fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(
        config["contributors"]["doc001"].is_object(),
        "Contributor should exist"
    );
    assert_eq!(config["contributors"]["doc001"]["name"], "Dr. Smith");
    assert_eq!(config["contributors"]["doc001"]["role"], "Physician");
    assert_eq!(
        config["contributors"]["doc001"]["email"],
        "smith@example.com"
    );
    assert_eq!(config["contributors"]["doc001"]["enabled"], true);
    assert_eq!(config["contributors"]["doc001"]["active"], false);

    Ok(())
}

#[test]
#[serial]
fn test_add_contributor_with_minimal_info() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("nurse001", "Jane Doe", None, None)?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert!(config["contributors"]["nurse001"].is_object());
    assert_eq!(config["contributors"]["nurse001"]["name"], "Jane Doe");
    assert!(config["contributors"]["nurse001"]["role"].is_null());
    assert!(config["contributors"]["nurse001"]["email"].is_null());

    Ok(())
}

#[test]
#[serial]
fn test_add_duplicate_contributor_fails() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;

    let result = add_contributor("doc001", "Different Name", None, None);
    assert!(result.is_err(), "Adding duplicate contributor should fail");

    Ok(())
}

#[test]
#[serial]
fn test_enable_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    disable_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["enabled"], false);

    enable_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["enabled"], true);

    Ok(())
}

#[test]
#[serial]
fn test_disable_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    disable_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert_eq!(config["contributors"]["doc001"]["enabled"], false);
    assert_eq!(config["contributors"]["doc001"]["active"], false);

    Ok(())
}

#[test]
#[serial]
fn test_disable_deactivates_active_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    activate_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["active"], true);

    disable_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["active"], false);

    Ok(())
}

#[test]
#[serial]
fn test_activate_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    activate_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert_eq!(config["contributors"]["doc001"]["active"], true);
    assert_eq!(config["current_contributor"], "doc001");

    Ok(())
}

#[test]
#[serial]
fn test_activate_disabled_contributor_fails() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    disable_contributor("doc001")?;

    let result = activate_contributor("doc001");
    assert!(
        result.is_err(),
        "Activating disabled contributor should fail"
    );

    Ok(())
}

#[test]
#[serial]
fn test_activate_replaces_current_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    add_contributor("doc002", "Dr. Jones", None, None)?;

    activate_contributor("doc001")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["active"], true);
    assert_eq!(config["contributors"]["doc002"]["active"], false);

    activate_contributor("doc002")?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(config["contributors"]["doc001"]["active"], false);
    assert_eq!(config["contributors"]["doc002"]["active"], true);
    assert_eq!(config["current_contributor"], "doc002");

    Ok(())
}

#[test]
#[serial]
fn test_deactivate_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;
    activate_contributor("doc001")?;
    deactivate_contributor()?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    assert_eq!(config["contributors"]["doc001"]["active"], false);
    assert!(config["current_contributor"].is_null());

    Ok(())
}

#[test]
#[serial]
fn test_get_current_contributor() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;

    let current = get_current_contributor();
    assert!(current.is_none(), "No active contributor initially");

    activate_contributor("doc001")?;

    let current = get_current_contributor();
    assert_eq!(current, Some("doc001".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_contributor_has_timestamp() -> Result<()> {
    let _temp_dir = setup();

    add_contributor("doc001", "Dr. Smith", None, None)?;

    let content = fs::read_to_string(".gitehr/contributors.json")?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let added_at = &config["contributors"]["doc001"]["added_at"];
    assert!(!added_at.is_null(), "added_at should be present");

    let time_str = added_at.as_str().unwrap();
    assert!(time_str.contains("T"), "Should be ISO 8601 format");
    assert!(time_str.contains("Z"), "Should be in UTC");

    Ok(())
}

#[test]
#[serial]
fn test_contributor_operations_fail_without_gitehr() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = add_contributor("doc001", "Dr. Smith", None, None);
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}
