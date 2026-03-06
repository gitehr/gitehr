// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConfig {
    pub remotes: HashMap<String, RemoteEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntry {
    pub url: String,
    pub added_at: String,
}

fn get_config_path() -> PathBuf {
    PathBuf::from(".gitehr/remotes.json")
}

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn load_config() -> Result<RemoteConfig> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(RemoteConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: RemoteConfig = serde_json::from_str(&content)?;
    Ok(config)
}

fn save_config(config: &RemoteConfig) -> Result<()> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

pub fn add_remote(name: &str, url: &str) -> Result<()> {
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

pub fn remove_remote(name: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    if !config.remotes.contains_key(name) {
        anyhow::bail!("Remote '{}' does not exist.", name);
    }

    config.remotes.remove(name);
    save_config(&config)?;

    println!("Removed remote '{}'", name);
    Ok(())
}

pub fn list_remotes() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let config = load_config()?;

    if config.remotes.is_empty() {
        println!("No remotes configured.");
        println!("Use 'gitehr remote add <name> <url>' to add one.");
        return Ok(());
    }

    println!("Configured remotes:");
    for (name, entry) in &config.remotes {
        println!("  {} -> {}", name, entry.url);
    }

    Ok(())
}
