// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP audit logging (R33).
//!
//! Every successful MCP tool call is recorded as a dedicated journal entry,
//! distinct from any journal entry the tool itself may have written, per the
//! `mcp_audit` front matter shape in `spec/mcp.md`. Fields that require
//! client/session context GitEHR does not yet track (client name/version,
//! token, IP - see R32) are omitted rather than faked.

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

use crate::commands::git;

#[derive(Debug, Serialize)]
struct AuditFrontMatter {
    timestamp: chrono::DateTime<Utc>,
    author: &'static str,
    mcp_audit: McpAudit,
}

#[derive(Debug, Serialize)]
struct McpAudit {
    method: &'static str,
    tool: String,
    result: &'static str,
}

/// Record a successful `tools/call` invocation as an audit journal entry.
/// Best-effort: a failure to write the audit trail is logged to stderr and
/// never propagated, so it can't turn a successful tool call into a failure.
pub fn record_tool_call(repo_path: &Path, tool: &str, detail: &str) {
    if let Err(e) = try_record(repo_path, tool, detail) {
        eprintln!("Warning: failed to write MCP audit entry: {e}");
    }
}

fn try_record(repo_path: &Path, tool: &str, detail: &str) -> Result<()> {
    let timestamp = Utc::now();
    let front = AuditFrontMatter {
        timestamp,
        author: "mcp-server",
        mcp_audit: McpAudit {
            method: "tools/call",
            tool: tool.to_string(),
            result: "success",
        },
    };

    let yaml = serde_yaml_ng::to_string(&front)?;
    let body = format!(
        "# MCP Audit Log\n\n**Operation**: {tool}\n**Result**: Success\n**Detail**: {detail}\n"
    );
    let file_content = format!("---\n{yaml}---\n\n{body}");

    let relative_filename = format!(
        "journal/{}-{}.md",
        timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
        Uuid::new_v4()
    );

    std::fs::write(repo_path.join(&relative_filename), file_content)?;
    git::git_add_in(repo_path, &relative_filename)?;
    git::git_commit_in(repo_path, &format!("MCP audit: {tool}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_git_repo(dir: &Path) {
        for args in [
            vec!["init"],
            vec!["config", "user.name", "Test User"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "commit.gpgsign", "false"],
        ] {
            std::process::Command::new("git")
                .current_dir(dir)
                .args(&args)
                .output()
                .unwrap();
        }
    }

    #[test]
    fn record_tool_call_writes_and_commits_an_audit_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("journal")).unwrap();
        init_git_repo(dir.path());

        record_tool_call(dir.path(), "add_journal_entry", "Created journal entry: x");

        let entries: Vec<_> = std::fs::read_dir(dir.path().join("journal"))
            .unwrap()
            .collect();
        assert_eq!(entries.len(), 1);

        let content = std::fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert!(content.contains("mcp_audit:"));
        assert!(content.contains("tool: add_journal_entry"));
        assert!(content.contains("result: success"));
        assert!(content.contains("# MCP Audit Log"));

        let log = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["log", "--oneline"])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&log.stdout).lines().count(),
            1,
            "expected the audit entry to be committed"
        );
    }

    #[test]
    fn record_tool_call_does_not_panic_when_repo_is_unwritable() {
        // No journal directory, no git repo at all: the write/commit fail,
        // and record_tool_call must swallow the error rather than panic.
        let dir = tempfile::tempdir().unwrap();
        record_tool_call(dir.path(), "search_repository", "no results");
    }
}
