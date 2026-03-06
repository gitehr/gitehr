// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

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
    public_key: Option<&str>,
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
        public_key: public_key.map(|s| s.to_string()),
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

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_required(message: &str) -> Result<String> {
    loop {
        let value = prompt(message)?;
        if !value.is_empty() {
            return Ok(value);
        }
        println!("Please enter a value.");
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn derive_user_id(name: &str, email: &str, config: &ContributorConfig) -> String {
    let base_from_email = email.split('@').next().unwrap_or("");
    let mut base = if !base_from_email.is_empty() {
        slugify(base_from_email)
    } else {
        slugify(name)
    };

    if base.is_empty() {
        base = "user".to_string();
    }

    if !config.contributors.contains_key(&base) {
        return base;
    }

    let mut rng = rand::rng();
    loop {
        let suffix: u16 = rng.random();
        let candidate = format!("{}-{:04x}", base, suffix);
        if !config.contributors.contains_key(&candidate) {
            return candidate;
        }
    }
}

fn get_home_dir() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return Ok(PathBuf::from(home));
    }
    anyhow::bail!("Could not determine home directory.");
}

fn generate_keypair(id: &str, email: &str) -> Result<String> {
    let home = get_home_dir()?;
    let key_dir = home.join(".gitehr").join("keys");
    fs::create_dir_all(&key_dir)?;

    let key_path = key_dir.join(format!("{}_ed25519", id));
    let key_path_str = key_path.to_string_lossy().to_string();

    let output = Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-f", &key_path_str, "-N", "", "-C", email])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "ssh-keygen not found. Install OpenSSH or paste an existing public key."
                )
            } else {
                anyhow::anyhow!("Failed to execute ssh-keygen: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ssh-keygen failed: {}", stderr.trim());
    }

    let pub_key_path = key_path.with_extension("pub");
    let public_key = fs::read_to_string(&pub_key_path)?.trim().to_string();

    println!("Key pair created at {}", key_path_str);
    Ok(public_key)
}

pub fn create_user_interactive() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let name = prompt_required("Name: ")?;
    let email = prompt_required("Email: ")?;

    let config = load_config()?;
    let id = derive_user_id(&name, &email, &config);

    println!("User ID: {}", id);

    let key_input = prompt("Public key (leave blank to generate): ")?;
    let public_key = if !key_input.is_empty() {
        Some(key_input)
    } else {
        let choice = prompt("Generate an elliptic curve key pair now? [Y/n]: ")?;
        if choice.is_empty()
            || choice.eq_ignore_ascii_case("y")
            || choice.eq_ignore_ascii_case("yes")
        {
            Some(generate_keypair(&id, &email)?)
        } else {
            None
        }
    };

    add_contributor(&id, &name, None, Some(&email), public_key.as_deref())?;
    Ok(())
}
