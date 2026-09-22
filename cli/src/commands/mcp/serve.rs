// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use super::server_impl::{McpConfig, McpServer, ServerConfig, ensure_not_encrypted};

pub fn run(repo_path: Option<PathBuf>) -> Result<()> {
    super::init_tracing();

    let repo_path = repo_path.unwrap_or_else(|| PathBuf::from("."));
    validate_repo(&repo_path)?;
    let mcp_config = load_enabled_config(&repo_path)?;

    let config = ServerConfig {
        repo_path,
        server_name: "gitehr".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        mcp_config,
    };

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let mut server = McpServer::new(config);
        server.run_stdio().await
    })
}

/// Refuse to serve unless `repo_path` is an unencrypted GitEHR repository
/// (see spec/roadmap.md R76).
fn validate_repo(repo_path: &Path) -> Result<()> {
    if !repo_path.join(".gitehr").is_dir() {
        bail!(
            "{} is not a GitEHR repository (no .gitehr directory found). \
             Run `gitehr store init` to create one, or pass --repo-path to point at an existing repository.",
            repo_path.display()
        );
    }
    // The operator running the server gets the path; the per-request guard
    // deliberately withholds it from MCP clients.
    ensure_not_encrypted(repo_path).with_context(|| {
        format!(
            "{} is marked as encrypted (.gitehr/ENCRYPTED present)",
            repo_path.display()
        )
    })?;
    Ok(())
}

/// Load `.gitehr/mcp.json` (R35), refusing to serve when it sets
/// `"enabled": false`.
fn load_enabled_config(repo_path: &Path) -> Result<McpConfig> {
    let mcp_config = McpConfig::load(repo_path)
        .with_context(|| format!("Loading .gitehr/mcp.json for {}", repo_path.display()))?;
    if !mcp_config.enabled {
        bail!(
            "MCP server is disabled for {} (.gitehr/mcp.json sets \"enabled\": false)",
            repo_path.display()
        );
    }
    Ok(mcp_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_gitehr_directory() {
        let dir = tempfile::tempdir().unwrap();
        let err = validate_repo(dir.path()).unwrap_err();
        assert!(err.to_string().contains("not a GitEHR repository"));
    }

    #[test]
    fn rejects_encrypted_repository() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
        std::fs::write(dir.path().join(".gitehr/ENCRYPTED"), "").unwrap();
        let err = validate_repo(dir.path()).unwrap_err();
        assert!(err.to_string().contains("marked as encrypted"));
    }

    #[test]
    fn accepts_plain_gitehr_repository() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
        validate_repo(dir.path()).unwrap();
    }

    #[test]
    fn loads_default_config_when_mcp_json_absent() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_enabled_config(dir.path()).unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn refuses_to_serve_when_mcp_json_disables_the_server() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/mcp.json"),
            r#"{ "enabled": false }"#,
        )
        .unwrap();

        let err = load_enabled_config(dir.path()).unwrap_err();
        assert!(err.to_string().contains("MCP server is disabled"));
    }
}
