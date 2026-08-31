// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::journal;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

/// Refuses to proceed if `path` exists and is a symlink. `.gitehr/` control
/// files are repo-local and can be attacker-supplied (a received or cloned
/// repository); following a symlink there would read or write whatever
/// arbitrary path the symlink points to instead of the repo's own file.
pub(super) fn reject_symlink(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        anyhow::bail!(
            "Refusing to use '{}': it is a symlink, which could point outside the repository.",
            path.display()
        );
    }
    Ok(())
}

fn get_current_version() -> Result<Option<String>> {
    let path = Path::new(".gitehr/GITEHR_VERSION");
    reject_symlink(path)?;
    Ok(fs::read_to_string(path).ok().map(|s| s.trim().to_string()))
}

fn get_current_exe_path() -> Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))
}

pub(super) fn update_bundled_binary() -> Result<()> {
    let source = get_current_exe_path()?;
    let dest = PathBuf::from(".gitehr/gitehr");
    reject_symlink(&dest)?;

    fs::copy(&source, &dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;
    }

    Ok(())
}

pub fn run() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let current_version = get_current_version()?;
    let new_version = env!("CARGO_PKG_VERSION");

    println!("GitEHR Repository Upgrade");
    println!("=========================");
    println!();

    if let Some(ref current) = current_version {
        println!("Current version: {}", current);
    } else {
        println!("Current version: unknown");
    }
    println!("New version: {}", new_version);
    println!();

    if current_version.as_deref() == Some(new_version) {
        println!("Repository is already at the latest version.");
        return Ok(());
    }

    println!("Performing upgrade...");

    let version_path = Path::new(".gitehr/GITEHR_VERSION");
    reject_symlink(version_path)?;
    fs::write(version_path, new_version)?;
    println!("  Updated version file.");

    update_bundled_binary()?;
    println!("  Updated bundled binary.");

    let upgrade_message = format!(
        "Repository upgraded from {} to {}",
        current_version.as_deref().unwrap_or("unknown"),
        new_version
    );

    journal::create_journal_entry(&upgrade_message)?;
    println!("  Recorded upgrade in journal.");

    println!();
    println!("Upgrade complete!");

    Ok(())
}
