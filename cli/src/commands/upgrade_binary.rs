// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
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
    super::upgrade::update_bundled_binary()?;
    println!("  Copied current executable to .gitehr/gitehr");

    fs::write(".gitehr/GITEHR_VERSION", cli_version)?;
    println!("  Updated version file to {}", cli_version);

    println!();
    println!("Binary upgrade complete!");

    Ok(())
}
