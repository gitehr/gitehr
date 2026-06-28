// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Global/local GitEHR configuration.
//!
//! The default config path follows XDG on Unix-like systems:
//! `$XDG_CONFIG_HOME/gitehr/config.toml`, falling back to
//! `$HOME/.config/gitehr/config.toml`. `GITEHR_CONFIG` can point at a specific
//! file, and `GITEHR_STORE_PATH` overrides the configured Store for one process.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONFIG_ENV: &str = "GITEHR_CONFIG";
pub const STORE_PATH_ENV: &str = "GITEHR_STORE_PATH";

const CONFIG_DIR: &str = "gitehr";
const CONFIG_FILE: &str = "config.toml";
const STORE_MARKER: &str = "gitehr-mpi.json";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub store_path: Option<PathBuf>,
}

pub fn config_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(CONFIG_ENV).filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(xdg).join(CONFIG_DIR).join(CONFIG_FILE));
    }

    Ok(home_dir()?
        .join(".config")
        .join(CONFIG_DIR)
        .join(CONFIG_FILE))
}

pub fn load() -> Result<AppConfig> {
    load_from_path(&config_path()?)
}

pub fn load_from_path(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read GitEHR config at {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Failed to parse GitEHR config at {}", path.display()))
}

pub fn save(config: &AppConfig) -> Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create GitEHR config directory {}",
                parent.display()
            )
        })?;
    }

    let content = toml::to_string_pretty(config).context("Failed to serialise GitEHR config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write GitEHR config at {}", path.display()))?;
    Ok(path)
}

pub fn configured_store_path() -> Result<Option<PathBuf>> {
    if let Some(path) = std::env::var_os(STORE_PATH_ENV).filter(|v| !v.is_empty()) {
        return Ok(Some(absolute_path(&PathBuf::from(path))?));
    }

    let config = load()?;
    config.store_path.as_deref().map(absolute_path).transpose()
}

pub fn set_store_path(path: &Path) -> Result<PathBuf> {
    let store_path = absolute_path(path)?;
    if !store_path.join(STORE_MARKER).exists() {
        bail!(
            "{} is not a GitEHR Store root ({} not found)",
            store_path.display(),
            STORE_MARKER
        );
    }

    let mut config = load()?;
    config.store_path = Some(store_path.clone());
    save(&config)?;
    Ok(store_path)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    let expanded = expand_tilde(path)?;
    if expanded.is_absolute() {
        Ok(expanded)
    } else {
        Ok(std::env::current_dir()?.join(expanded))
    }
}

fn expand_tilde(path: &Path) -> Result<PathBuf> {
    let Some(text) = path.to_str() else {
        return Ok(path.to_path_buf());
    };

    if text == "~" {
        return home_dir();
    }
    if let Some(rest) = text.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }
    Ok(path.to_path_buf())
}

fn home_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(home) = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty()) {
            return Ok(PathBuf::from(home));
        }
    }

    if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(home));
    }

    bail!("Cannot determine home directory for GitEHR config path")
}
