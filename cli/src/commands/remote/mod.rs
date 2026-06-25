// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod add;
pub mod list;
pub mod remove;

#[derive(Subcommand)]
pub enum RemoteCommands {
    Add {
        #[arg(help = "Name for the remote")]
        name: String,
        #[arg(help = "URL of the remote")]
        url: String,
    },
    #[command(visible_alias = "rm")]
    Remove {
        #[arg(help = "Name of the remote to remove")]
        name: String,
    },
    List,
}

pub fn run(command: Option<RemoteCommands>) -> Result<()> {
    match command {
        Some(RemoteCommands::Add { name, url }) => add::run(&name, &url),
        Some(RemoteCommands::Remove { name }) => remove::run(&name),
        Some(RemoteCommands::List) | None => list::run(),
    }
}

// ── Shared data structures ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConfig {
    pub remotes: HashMap<String, RemoteEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntry {
    pub url: String,
    pub added_at: String,
}

// ── Shared helper functions ───────────────────────────────────────────────────

pub(super) fn get_config_path() -> PathBuf {
    PathBuf::from(".gitehr/remotes.json")
}

pub(super) fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

pub(super) fn load_config() -> Result<RemoteConfig> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(RemoteConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: RemoteConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub(super) fn save_config(config: &RemoteConfig) -> Result<()> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}
