// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

use super::{RemoteEntry, is_gitehr_repo, load_config, save_config};

pub fn run(name: &str, url: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    if config.remotes.contains_key(name) {
        anyhow::bail!(
            "Remote '{}' already exists. Use 'gitehr remote remove {}' first.",
            name,
            name
        );
    }

    let entry = RemoteEntry {
        url: url.to_string(),
        added_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    config.remotes.insert(name.to_string(), entry);
    save_config(&config)?;

    println!("Added remote '{}' -> {}", name, url);
    Ok(())
}
