// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `.gitehr/mcp.json` server configuration (R35).
//!
//! Per-repository, since the repository owner - not a global machine
//! setting - decides which MCP surfaces to expose. Only the fields the
//! server actually enforces are modelled here: a top-level `enabled` kill
//! switch, and per-resource-group/per-tool `enabled` flags gating
//! `resources/list`, `resources/read`, `tools/list`, and `tools/call`.
//!
//! A field in a server configuration must always take effect. Unsupported or
//! misspelled fields are rejected rather than silently ignored: accepting an
//! `auth` or `transport` policy while continuing without it would be false
//! assurance in a clinical-data service.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_FILENAME: &str = "mcp.json";

fn default_true() -> bool {
    true
}

/// A single `{ "enabled": true }` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureFlag {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for FeatureFlag {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcesConfig {
    #[serde(default)]
    pub journal: FeatureFlag,
    #[serde(default)]
    pub state: FeatureFlag,
    #[serde(default)]
    pub documents: FeatureFlag,
    #[serde(default)]
    pub imaging: FeatureFlag,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolsConfig {
    #[serde(default, rename = "add_journal_entry")]
    pub add_journal_entry: FeatureFlag,
    #[serde(default, rename = "update_state")]
    pub update_state: FeatureFlag,
    #[serde(default, rename = "search_repository")]
    pub search_repository: FeatureFlag,
}

/// MCP server configuration loaded from `.gitehr/mcp.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub resources: ResourcesConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resources: ResourcesConfig::default(),
            tools: ToolsConfig::default(),
        }
    }
}

impl McpConfig {
    /// Load `.gitehr/mcp.json` from `repo_path`, or the default
    /// (everything enabled) when the file does not exist.
    pub fn load(repo_path: &Path) -> Result<Self> {
        let path = repo_path.join(".gitehr").join(CONFIG_FILENAME);
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Failed to inspect MCP config at {}", path.display())
                });
            }
        };
        if metadata.file_type().is_symlink() {
            bail!(
                "Refusing MCP config at {}: it is a symlink, which could point outside the repository",
                path.display()
            );
        }
        if !metadata.is_file() {
            bail!("MCP config at {} must be a regular file", path.display());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read MCP config at {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse MCP config at {}", path.display()))
    }

    pub fn is_resource_enabled(&self, name: &str) -> bool {
        match name {
            "journal" => self.resources.journal.enabled,
            "state" => self.resources.state.enabled,
            "documents" => self.resources.documents.enabled,
            "imaging" => self.resources.imaging.enabled,
            // "status" and any resource this server doesn't model a flag
            // for are not gateable and stay enabled.
            _ => true,
        }
    }

    pub fn is_tool_enabled(&self, name: &str) -> bool {
        match name {
            "add_journal_entry" => self.tools.add_journal_entry.enabled,
            "update_state" => self.tools.update_state.enabled,
            "search_repository" => self.tools.search_repository.enabled,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_everything_enabled_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = McpConfig::load(dir.path()).unwrap();
        assert!(config.enabled);
        assert!(config.is_resource_enabled("journal"));
        assert!(config.is_resource_enabled("state"));
        assert!(config.is_resource_enabled("documents"));
        assert!(config.is_resource_enabled("imaging"));
        assert!(config.is_tool_enabled("add_journal_entry"));
        assert!(config.is_tool_enabled("update_state"));
        assert!(config.is_tool_enabled("search_repository"));
    }

    #[test]
    fn loads_partial_config_and_defaults_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/mcp.json"),
            r#"{
                "resources": { "imaging": { "enabled": false } },
                "tools": { "update_state": { "enabled": false } }
            }"#,
        )
        .unwrap();

        let config = McpConfig::load(dir.path()).unwrap();
        assert!(config.enabled);
        assert!(config.is_resource_enabled("journal"));
        assert!(!config.is_resource_enabled("imaging"));
        assert!(config.is_tool_enabled("add_journal_entry"));
        assert!(!config.is_tool_enabled("update_state"));
    }

    #[test]
    fn top_level_disabled_flag_is_read() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/mcp.json"),
            r#"{ "enabled": false }"#,
        )
        .unwrap();

        let config = McpConfig::load(dir.path()).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn rejects_unimplemented_or_unknown_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/mcp.json"),
            r#"{
                "enabled": true,
                "transport": "stdio",
                "port": 3000,
                "auth": { "method": "token", "token_file": ".gitehr/mcp-tokens.json" },
                "resources": { "journal": { "enabled": true, "max_entries": 1000 } },
                "tools": { "calculate_clinical": { "enabled": true } },
                "prompts": { "enabled": true, "custom_prompts_dir": ".gitehr/prompts/" },
                "audit": { "log_all_requests": true }
            }"#,
        )
        .unwrap();

        let err = McpConfig::load(dir.path()).unwrap_err();
        assert!(format!("{err:#}").contains("unknown field"));
    }

    #[test]
    fn rejects_unknown_nested_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/mcp.json"),
            r#"{ "resources": { "journal": { "max_entries": 1000 } } }"#,
        )
        .unwrap();

        let err = McpConfig::load(dir.path()).unwrap_err();
        assert!(format!("{err:#}").contains("unknown field `max_entries`"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_config() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        let target = dir.path().join("elsewhere.json");
        std::fs::write(&target, r#"{ "enabled": false }"#).unwrap();
        symlink(&target, dir.path().join(".gitehr/mcp.json")).unwrap();

        let err = McpConfig::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("it is a symlink"));
    }

    #[test]
    fn rejects_malformed_config_rather_than_silently_defaulting() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(dir.path().join(".gitehr/mcp.json"), "{ not json").unwrap();

        let err = McpConfig::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("Failed to parse MCP config"));
    }
}
