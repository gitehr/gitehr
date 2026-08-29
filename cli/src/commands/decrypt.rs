// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn is_encrypted() -> bool {
    PathBuf::from(".gitehr/ENCRYPTED").exists()
}

pub fn run(key_source: Option<&str>) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    if !is_encrypted() {
        anyhow::bail!("Repository is not encrypted.");
    }

    let _ = key_source;

    println!("Removing stale .gitehr/ENCRYPTED marker.");
    println!();
    println!("Note: encryption at rest was never implemented (roadmap R67/R68),");
    println!("so no data was actually encrypted and there is nothing to decrypt.");
    println!("This marker was written by an earlier placeholder version of");
    println!("`gitehr encrypt`.");

    fs::remove_file(".gitehr/ENCRYPTED")?;

    println!();
    println!("Marker removed.");

    Ok(())
}
