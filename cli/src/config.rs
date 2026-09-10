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
    /// File extensions (without the leading dot, case-insensitive) that
    /// `gitehr import --mode documents` will accept. `None` (the field is
    /// absent from the TOML) accepts any format, matching the pre-whitelist
    /// behaviour.
    #[serde(default)]
    pub document_whitelist: Option<Vec<String>>,
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

/// The configured document-format whitelist, normalised to lowercase
/// extensions with no leading dot. `None` means no whitelist is configured,
/// so `--mode documents` should accept any format.
///
/// An unusable whitelist is an error rather than a filter that silently
/// matches nothing: a typo here would otherwise skip every document the
/// setting was meant to admit, and the import would report only a skip count.
pub fn configured_document_whitelist() -> Result<Option<Vec<String>>> {
    let config = load()?;
    let path = config_path()?;
    config
        .document_whitelist
        .map(|extensions| normalise_document_whitelist(extensions, &path))
        .transpose()
}

fn normalise_document_whitelist(extensions: Vec<String>, path: &Path) -> Result<Vec<String>> {
    if extensions.is_empty() {
        bail!(
            "document_whitelist in {} is empty, which would reject every document. \
             Remove the field to accept any format.",
            path.display()
        );
    }

    extensions
        .iter()
        .map(|entry| {
            let extension = entry.trim().trim_start_matches('.').to_lowercase();
            // `Path::extension` yields the final component only, so anything
            // that is not a bare extension can never match a file.
            if extension.is_empty()
                || extension
                    .chars()
                    .any(|c| c == '.' || c == '*' || c == '/' || c == '\\' || c.is_whitespace())
            {
                bail!(
                    "document_whitelist in {} contains {:?}, which is not a bare file extension. \
                     Use entries like \"pdf\" or \"jpg\" (a leading dot is allowed).",
                    path.display(),
                    entry
                );
            }
            Ok(extension)
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::normalise_document_whitelist;
    use std::path::Path;

    fn normalise(entries: &[&str]) -> anyhow::Result<Vec<String>> {
        let entries = entries.iter().map(|e| e.to_string()).collect();
        normalise_document_whitelist(entries, Path::new("/tmp/config.toml"))
    }

    #[test]
    fn accepts_bare_extensions_with_or_without_a_leading_dot() {
        assert_eq!(
            normalise(&["pdf", ".JPG", " png "]).unwrap(),
            vec!["pdf", "jpg", "png"]
        );
    }

    #[test]
    fn rejects_entries_that_could_never_match_a_file_extension() {
        // Each of these would otherwise silently reject every document.
        for entry in ["", ".", "*", "*.pdf", "tar.gz", "scans/pdf", "p df"] {
            let error = normalise(&[entry]).expect_err(entry);
            assert!(
                error.to_string().contains("not a bare file extension"),
                "{entry:?} produced: {error}"
            );
        }
    }

    #[test]
    fn rejects_an_empty_whitelist_rather_than_matching_nothing() {
        let error = normalise(&[]).expect_err("empty whitelist");
        assert!(error.to_string().contains("would reject every document"));
    }
}
