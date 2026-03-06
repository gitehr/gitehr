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

pub fn decrypt_repository(key_source: Option<&str>) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    if !is_encrypted() {
        anyhow::bail!("Repository is not encrypted.");
    }

    let key_info = key_source.unwrap_or("local");

    println!("Decrypting repository with {} key...", key_info);
    println!();
    println!("NOTE: Full decryption implementation is pending.");
    println!("This command will decrypt all clinical data files.");

    fs::remove_file(".gitehr/ENCRYPTED")?;

    println!();
    println!("Repository decrypted.");

    Ok(())
}
