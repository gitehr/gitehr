// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Repository-level guards shared by the resource and tool handlers.
//!
//! See "Security and Access Control" in [`spec/mcp.md`](../../../../../spec/mcp.md).

use std::path::Path;

/// Refuse an MCP operation against a repository carrying the encryption
/// marker.
///
/// `gitehr mcp serve` already refuses to start on a marked repository (see
/// `mcp::serve::validate_repo`, R76), but the server is a long-lived stdio
/// process: `.gitehr/ENCRYPTED` can appear after the server has already
/// started serving that repository. Per spec/mcp.md, "all MCP operations
/// should re-check encryption status per request", so resource reads and
/// tool calls call this on every request rather than relying on the
/// once-at-startup check alone.
///
/// The message does not claim the repository's contents are encrypted,
/// because they are not: encryption at rest is unimplemented (R67/R68) and
/// the marker is a stale artefact of the placeholder `gitehr encrypt` that
/// R79 removed. Refusing to serve a repository in that state is still the
/// safe response - GitEHR cannot tell what a marked repository was meant to
/// be - but telling a user their data is encrypted would be the same false
/// assurance R79 exists to prevent.
///
/// It also names no path. This string is returned to the MCP client
/// verbatim, and a Store's subject directories are named after the people
/// whose records they hold.
pub fn ensure_not_encrypted(repo_path: &Path) -> anyhow::Result<()> {
    if repo_path.join(".gitehr/ENCRYPTED").exists() {
        anyhow::bail!(
            "Repository encrypted: the repository carries a .gitehr/ENCRYPTED marker, \
             which GitEHR MCP refuses to serve. Encryption at rest is not implemented \
             (roadmap R67/R68), so no data was encrypted and the marker is stale; \
             `gitehr decrypt` removes it."
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
