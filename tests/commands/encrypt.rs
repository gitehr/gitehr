use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::decrypt::decrypt_repository;
use gitehr::commands::encrypt::encrypt_repository;

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    fs::create_dir_all("journal").ok();
    temp_dir
}

#[test]
#[serial]
fn test_encrypt_creates_marker_file() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(None)?;

    let marker_path = Path::new(".gitehr/ENCRYPTED");
    assert!(marker_path.exists(), "ENCRYPTED marker file should exist");

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_marker_contains_timestamp() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(None)?;

    let content = fs::read_to_string(".gitehr/ENCRYPTED")?;
    assert!(
        content.contains("encrypted_at="),
        "Should contain encrypted_at timestamp"
    );
    assert!(content.contains("T"), "Should be ISO 8601 format");
    assert!(content.contains("Z"), "Should be in UTC");

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_marker_contains_key_source() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(Some("azure-keyvault"))?;

    let content = fs::read_to_string(".gitehr/ENCRYPTED")?;
    assert!(
        content.contains("key_source=azure-keyvault"),
        "Should contain key_source"
    );

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_marker_default_key_source() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(None)?;

    let content = fs::read_to_string(".gitehr/ENCRYPTED")?;
    assert!(
        content.contains("key_source=local"),
        "Should default to local key source"
    );

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_fails_if_already_encrypted() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(None)?;

    let result = encrypt_repository(None);
    assert!(result.is_err(), "Should fail if already encrypted");

    Ok(())
}

#[test]
#[serial]
fn test_decrypt_removes_marker_file() -> Result<()> {
    let _temp_dir = setup();

    encrypt_repository(None)?;
    assert!(Path::new(".gitehr/ENCRYPTED").exists());

    decrypt_repository(None)?;

    assert!(
        !Path::new(".gitehr/ENCRYPTED").exists(),
        "ENCRYPTED marker should be removed"
    );

    Ok(())
}

#[test]
#[serial]
fn test_decrypt_fails_if_not_encrypted() -> Result<()> {
    let _temp_dir = setup();

    let result = decrypt_repository(None);
    assert!(result.is_err(), "Should fail if not encrypted");

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_decrypt_roundtrip() -> Result<()> {
    let _temp_dir = setup();

    let marker_path = Path::new(".gitehr/ENCRYPTED");

    assert!(!marker_path.exists(), "Should not be encrypted initially");

    encrypt_repository(None)?;
    assert!(marker_path.exists(), "Should be encrypted");

    decrypt_repository(None)?;
    assert!(
        !marker_path.exists(),
        "Should not be encrypted after decrypt"
    );

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_without_gitehr_fails() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = encrypt_repository(None);
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}

#[test]
#[serial]
fn test_decrypt_without_gitehr_fails() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = decrypt_repository(None);
    assert!(result.is_err(), "Should fail without .gitehr directory");

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_encrypt_actually_encrypts_files() -> Result<()> {
    let _temp_dir = setup();

    fs::write("journal/test.md", "Secret content")?;
    fs::write("state/data.txt", "Sensitive data")?;

    encrypt_repository(None)?;

    let journal_content = fs::read_to_string("journal/test.md")?;
    assert!(
        !journal_content.contains("Secret content"),
        "Journal should be encrypted"
    );

    let state_content = fs::read_to_string("state/data.txt")?;
    assert!(
        !state_content.contains("Sensitive data"),
        "State should be encrypted"
    );

    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_decrypt_actually_decrypts_files() -> Result<()> {
    let _temp_dir = setup();

    let secret = "Secret content";
    fs::write("journal/test.md", secret)?;

    encrypt_repository(None)?;

    decrypt_repository(None)?;

    let decrypted = fs::read_to_string("journal/test.md")?;
    assert_eq!(decrypted, secret, "Content should be decrypted");

    Ok(())
}
