// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use super::server_impl::{McpServer, ServerConfig};

pub fn run(repo_path: Option<PathBuf>) -> Result<()> {
    super::init_tracing();

    let repo_path = repo_path.unwrap_or_else(|| PathBuf::from("."));
    validate_repo(&repo_path)?;

    let config = ServerConfig {
        repo_path,
        server_name: "gitehr".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
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
    if repo_path.join(".gitehr/ENCRYPTED").exists() {
        bail!(
            "{} is marked as encrypted (.gitehr/ENCRYPTED present). \
             GitEHR MCP does not yet support encrypted repositories.",
            repo_path.display()
        );
    }
    Ok(())
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
}
