// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::journal;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn get_current_version() -> Option<String> {
    fs::read_to_string(".gitehr/GITEHR_VERSION")
        .ok()
        .map(|s| s.trim().to_string())
}

fn get_current_exe_path() -> Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))
}

fn update_bundled_binary() -> Result<()> {
    let source = get_current_exe_path()?;
    let dest = PathBuf::from(".gitehr/gitehr");

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

pub fn upgrade_repository() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let current_version = get_current_version();
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

    fs::write(".gitehr/GITEHR_VERSION", new_version)?;
    println!("  Updated version file.");

    update_bundled_binary()?;
    println!("  Updated bundled binary.");

    let upgrade_message = format!(
        "Repository upgraded from {} to {}",
        current_version.as_deref().unwrap_or("unknown"),
        new_version
    );

    let latest = journal::get_latest_journal_entry()?;
    let parent_hash = latest.map(|(_, hash)| hash);
    journal::create_journal_entry(&upgrade_message, parent_hash)?;
    println!("  Recorded upgrade in journal.");

    println!();
    println!("Upgrade complete!");

    Ok(())
}

pub fn upgrade_binary() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let cli_version = env!("CARGO_PKG_VERSION");
    let bundled_path = PathBuf::from(".gitehr/gitehr");

    println!("GitEHR Binary Upgrade");
    println!("=====================");
    println!();
    println!("CLI version: {}", cli_version);

    if bundled_path.exists() {
        println!("Bundled binary: exists");
    } else {
        println!("Bundled binary: not found");
    }
    println!();

    println!("Updating bundled binary...");
    update_bundled_binary()?;
    println!("  Copied current executable to .gitehr/gitehr");

    fs::write(".gitehr/GITEHR_VERSION", cli_version)?;
    println!("  Updated version file to {}", cli_version);

    println!();
    println!("Binary upgrade complete!");

    Ok(())
}
