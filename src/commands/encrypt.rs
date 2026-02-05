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

pub fn encrypt_repository(key_source: Option<&str>) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    if is_encrypted() {
        anyhow::bail!("Repository is already encrypted.");
    }

    let key_info = key_source.unwrap_or("local");

    println!("Encrypting repository with {} key...", key_info);
    println!();
    println!("NOTE: Full encryption implementation is pending.");
    println!("This command will encrypt all clinical data files using AES-256-GCM.");
    println!();
    println!("Planned encryption scope:");
    println!("  - journal/ directory contents");
    println!("  - state/ directory contents");
    println!("  - imaging/ directory contents");
    println!("  - documents/ directory contents");
    println!();
    println!("The .gitehr/ configuration directory will remain unencrypted");
    println!("to allow repository detection and key management.");

    fs::write(
        ".gitehr/ENCRYPTED",
        format!(
            "encrypted_at={}\nkey_source={}\n",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            key_info
        ),
    )?;

    println!();
    println!("Repository marked as encrypted (placeholder).");

    Ok(())
}
