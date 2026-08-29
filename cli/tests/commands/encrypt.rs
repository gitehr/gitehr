// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::decrypt::run as decrypt_repository;
use gitehr::commands::encrypt::run as encrypt_repository;

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    fs::create_dir_all("journal").ok();
    temp_dir
}

/// Write the marker an earlier placeholder `gitehr encrypt` used to leave.
fn write_stale_marker() {
    fs::write(
        ".gitehr/ENCRYPTED",
        "encrypted_at=2026-01-01T00:00:00Z\nkey_source=local\n",
    )
    .unwrap();
}

#[test]
#[serial]
fn test_encrypt_refuses_until_implemented() -> Result<()> {
    let _temp_dir = setup();

    let err = encrypt_repository(None).unwrap_err();
    assert!(
        err.to_string().contains("not yet implemented"),
        "encrypt must refuse until R67/R68 land, got: {err}"
    );

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_writes_no_marker() -> Result<()> {
    let _temp_dir = setup();

    let _ = encrypt_repository(None);

    assert!(
        !Path::new(".gitehr/ENCRYPTED").exists(),
        "encrypt must not claim encryption that never happened"
    );

    Ok(())
}

#[test]
#[serial]
fn test_encrypt_fails_if_already_encrypted() -> Result<()> {
    let _temp_dir = setup();
    write_stale_marker();

    let err = encrypt_repository(None).unwrap_err();
    assert!(
        err.to_string().contains("already encrypted"),
        "should report the existing marker, got: {err}"
    );

    Ok(())
}

#[test]
#[serial]
fn test_decrypt_removes_stale_marker() -> Result<()> {
    let _temp_dir = setup();
    write_stale_marker();

    decrypt_repository(None)?;

    assert!(
        !Path::new(".gitehr/ENCRYPTED").exists(),
        "decrypt should clean up a stale placeholder marker"
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

// The two tests below describe the behaviour R68 must deliver; they stay
// ignored until real encryption exists.

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
