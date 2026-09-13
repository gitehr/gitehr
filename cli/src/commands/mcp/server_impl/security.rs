// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Repository-level guards shared by the resource and tool handlers.
//!
//! See "Security and Access Control" in [`spec/mcp.md`](../../../../../spec/mcp.md).

use std::path::Path;

/// Refuse an MCP operation against an encrypted repository.
///
/// `gitehr mcp serve` already refuses to start on an encrypted repository
/// (see `mcp::serve::validate_repo`, R76), but the server is a long-lived
/// stdio process: `.gitehr/ENCRYPTED` can appear after the server has
/// already started serving that repository. Per spec/mcp.md, "all MCP
/// operations should re-check encryption status per request", so resource
/// reads and tool calls call this on every request rather than relying on
/// the once-at-startup check alone.
pub fn ensure_not_encrypted(repo_path: &Path) -> anyhow::Result<()> {
    if repo_path.join(".gitehr/ENCRYPTED").exists() {
        anyhow::bail!(
            "Repository encrypted: {} is marked as encrypted (.gitehr/ENCRYPTED present). \
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
    fn allows_unencrypted_repository() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ensure_not_encrypted(dir.path()).is_ok());
    }

    #[test]
    fn refuses_encrypted_repository() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".gitehr")).unwrap();
        std::fs::write(dir.path().join(".gitehr/ENCRYPTED"), "").unwrap();

        let err = ensure_not_encrypted(dir.path()).unwrap_err();
        assert!(err.to_string().contains("Repository encrypted"));
    }
}
