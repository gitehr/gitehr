// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_gitehr_repository() -> Result<()> {
    if !Path::new(".gitehr").exists() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }
    Ok(())
}

pub fn state_path(filename: &str) -> PathBuf {
    PathBuf::from("state").join(filename)
}

pub fn read_front_matter<T>(filename: &str) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let path = state_path(filename);
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read state file {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(T::default());
    }

    let yaml = extract_front_matter(&content).unwrap_or(content.as_str());
    if yaml.trim().is_empty() {
        return Ok(T::default());
    }

    serde_yaml_ng::from_str(yaml)
        .with_context(|| format!("Failed to parse YAML front matter in {}", path.display()))
}

pub fn write_front_matter<T>(filename: &str, value: &T) -> Result<PathBuf>
where
    T: Serialize,
{
    let path = state_path(filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml = serde_yaml_ng::to_string(value)?;
    let content = format!("---\n{}---\n", yaml);
    fs::write(&path, content)
        .with_context(|| format!("Failed to write state file {}", path.display()))?;
    Ok(path)
}

fn extract_front_matter(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}
