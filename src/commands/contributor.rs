// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub public_key: Option<String>,
    pub enabled: bool,
    pub active: bool,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContributorConfig {
    pub contributors: HashMap<String, Contributor>,
    pub current_contributor: Option<String>,
}

fn get_config_path() -> PathBuf {
    PathBuf::from(".gitehr/contributors.json")
}

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn load_config() -> Result<ContributorConfig> {
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(ContributorConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: ContributorConfig = serde_json::from_str(&content)?;
    Ok(config)
}

fn save_config(config: &ContributorConfig) -> Result<()> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

/// Get the currently active contributor, if any.
/// Returns the contributor ID.
pub fn get_current_contributor() -> Option<String> {
    load_config()
        .ok()
        .and_then(|config| config.current_contributor)
}

pub fn add_contributor(
    id: &str,
    name: &str,
    role: Option<&str>,
    email: Option<&str>,
) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    if config.contributors.contains_key(id) {
        anyhow::bail!("User '{}' already exists.", id);
    }

    let contributor = Contributor {
        id: id.to_string(),
        name: name.to_string(),
        role: role.map(|s| s.to_string()),
        email: email.map(|s| s.to_string()),
        public_key: None,
        enabled: true,
        active: false,
        added_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    };

    config.contributors.insert(id.to_string(), contributor);
    save_config(&config)?;

    println!("Added user '{}' ({})", id, name);
    Ok(())
}

pub fn enable_contributor(id: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    let contributor = config
        .contributors
        .get_mut(id)
        .ok_or_else(|| anyhow::anyhow!("User '{}' not found.", id))?;

    contributor.enabled = true;
    save_config(&config)?;

    println!("Enabled user '{}'", id);
    Ok(())
}

pub fn disable_contributor(id: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    let contributor = config
        .contributors
        .get_mut(id)
        .ok_or_else(|| anyhow::anyhow!("User '{}' not found.", id))?;

    contributor.enabled = false;
    contributor.active = false;
    save_config(&config)?;

    println!("Disabled user '{}'", id);
    Ok(())
}

pub fn activate_contributor(id: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    {
        let contributor = config
            .contributors
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found.", id))?;

        if !contributor.enabled {
            anyhow::bail!("User '{}' is disabled. Enable it first.", id);
        }
    }

    for c in config.contributors.values_mut() {
        c.active = false;
    }

    config.contributors.get_mut(id).unwrap().active = true;
    config.current_contributor = Some(id.to_string());
    save_config(&config)?;

    println!("Activated user '{}' as current author", id);
    Ok(())
}

pub fn deactivate_contributor() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    for c in config.contributors.values_mut() {
        c.active = false;
    }
    config.current_contributor = None;
    save_config(&config)?;

    println!("Deactivated current user");
    Ok(())
}

pub fn list_contributors() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let config = load_config()?;

    if config.contributors.is_empty() {
        println!("No users configured.");
        println!("Use 'gitehr user add <id> <name>' to add one.");
        return Ok(());
    }

    println!("Users:");
    for (id, contributor) in &config.contributors {
        let status = match (contributor.enabled, contributor.active) {
            (true, true) => "[active]",
            (true, false) => "[enabled]",
            (false, _) => "[disabled]",
        };
        println!(
            "  {} - {} {} {}",
            id,
            contributor.name,
            contributor.role.as_deref().unwrap_or(""),
            status
        );
    }

    Ok(())
}
