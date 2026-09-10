// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use gitehr::commands::import::{ImportMode, run as import_run};
use gitehr::config::CONFIG_ENV;

/// Restores the process-wide current directory and `GITEHR_CONFIG` on drop,
/// since `configured_document_whitelist` reads the config path from the
/// environment and `import::run` resolves paths against the current
/// directory — both process-global state these tests have to mutate.
struct EnvGuard {
    original_dir: PathBuf,
    original_config: Option<OsString>,
}

impl EnvGuard {
    fn new() -> Self {
        Self {
            original_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
            original_config: std::env::var_os(CONFIG_ENV),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_dir);
        match &self.original_config {
            Some(value) => unsafe { std::env::set_var(CONFIG_ENV, value) },
            None => unsafe { std::env::remove_var(CONFIG_ENV) },
        }
    }
}

fn setup_with_git() -> Result<(tempfile::TempDir, EnvGuard)> {
    let temp_dir = tempdir()?;
    let guard = EnvGuard::new();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir(".gitehr")?;
    fs::create_dir("journal")?;
    std::process::Command::new("git").args(["init"]).output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .output()?;
    Ok((temp_dir, guard))
}

/// Points `GITEHR_CONFIG` at a fresh config file in its own directory
/// containing the given TOML body (empty string for "no config file").
fn write_config(body: &str) {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");
    if !body.is_empty() {
        fs::write(&config_path, body).unwrap();
    }
    unsafe { std::env::set_var(CONFIG_ENV, &config_path) };
    // Leak the tempdir so it outlives the config file reference for the test.
    std::mem::forget(config_dir);
}

#[test]
#[serial]
fn import_documents_accepts_any_format_without_configured_whitelist() -> Result<()> {
    let (_temp_dir, _guard) = setup_with_git()?;
    write_config("");

    let source = tempdir()?;
    fs::write(source.path().join("scan.pdf"), b"pdf content")?;
    fs::write(source.path().join("notes.txt"), b"text content")?;

    import_run(ImportMode::Documents, source.path())?;

    assert!(PathBuf::from("documents/scan.pdf").exists());
    assert!(PathBuf::from("documents/notes.txt").exists());
    Ok(())
}

#[test]
#[serial]
fn import_documents_filters_by_configured_whitelist() -> Result<()> {
    let (_temp_dir, _guard) = setup_with_git()?;
    write_config("document_whitelist = [\"pdf\", \"jpg\"]\n");

    let source = tempdir()?;
    fs::write(source.path().join("scan.pdf"), b"pdf content")?;
    fs::write(source.path().join("photo.JPG"), b"jpg content")?;
    fs::write(source.path().join("notes.txt"), b"text content")?;

    import_run(ImportMode::Documents, source.path())?;

    assert!(PathBuf::from("documents/scan.pdf").exists());
    assert!(PathBuf::from("documents/photo.JPG").exists());
    assert!(!PathBuf::from("documents/notes.txt").exists());
    Ok(())
}

#[test]
#[serial]
fn import_documents_whitelist_rejects_extensionless_files() -> Result<()> {
    let (_temp_dir, _guard) = setup_with_git()?;
    write_config("document_whitelist = [\"pdf\"]\n");

    let source = tempdir()?;
    fs::write(source.path().join("README"), b"no extension")?;

    import_run(ImportMode::Documents, source.path())?;

    assert!(!PathBuf::from("documents/README").exists());
    Ok(())
}
