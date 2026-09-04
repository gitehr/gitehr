// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Review MCP-authored drafts (ADR-0007). MCP write tools produce uncommitted
//! draft entries; a human approves (strips the draft marker, stages, commits)
//! or rejects (deletes) them here. See `spec/adr/0007-mcp-writes-are-drafts-until-approved.md`.

use anyhow::Result;
use std::path::Path;

use super::{approve_mcp_draft, mcp_drafts, reject_mcp_draft};

/// Show unapproved MCP drafts, or approve/reject one by filename.
pub fn run(repo_path: &Path, approve: Option<&str>, reject: Option<&str>) -> Result<()> {
    if let Some(filename) = approve {
        approve_mcp_draft(repo_path, filename)?;
        println!("Approved and committed journal/{filename}.");
        return Ok(());
    }

    if let Some(filename) = reject {
        reject_mcp_draft(repo_path, filename)?;
        println!("Rejected (deleted) journal/{filename}. The record is unchanged.");
        return Ok(());
    }

    let drafts = mcp_drafts(repo_path)?;
    if drafts.is_empty() {
        println!("No pending MCP drafts.");
        return Ok(());
    }

    println!("{} pending MCP draft(s):", drafts.len());
    println!();
    for (filename, parsed) in &drafts {
        let author = parsed
            .metadata
            .author
            .as_deref()
            .unwrap_or("(no author recorded)");
        let first_line = parsed
            .content
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("(empty)");
        println!("  {}", filename);
        println!("    author: {}", author);
        println!("    {}", truncate(first_line, 72));
    }
    println!();
    println!("Approve with: gitehr journal drafts --approve <filename>");
    println!("Reject with:  gitehr journal drafts --reject <filename>");
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}
