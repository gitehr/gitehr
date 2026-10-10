// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Client-side policy check over Git history (R40, see
//! `spec/repository-verification.md`). Currently asserts one invariant:
//! no commit ever modified, deleted, or renamed a committed journal entry
//! (ADR-0002, the record only grows). Object integrity is left to `git fsck`.

use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct Violation {
    pub commit: String,
    pub change: char,
    pub path: String,
}

/// Parse `git log --name-status --no-renames --format=%x01%H` output into the
/// journal changes that are not additions.
fn parse_violations(log: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut commit = String::new();
    for line in log.lines() {
        if let Some(hash) = line.strip_prefix('\u{1}') {
            commit = hash.trim().to_string();
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let (Some(status), Some(path)) = (parts.next(), parts.next()) else {
            continue;
        };
        let change = status.chars().next().unwrap_or('?');
        if change != 'A' && path.starts_with("journal/") {
            violations.push(Violation {
                commit: commit.clone(),
                change,
                path: path.to_string(),
            });
        }
    }
    violations
}

/// Walk the whole history and return every non-additive change to `journal/`.
pub fn check_append_only() -> Result<Vec<Violation>> {
    let output = Command::new("git")
        .args([
            "log",
            "--reverse",
            "--no-renames",
            "--name-status",
            "--format=%x01%H",
            "--",
            "journal",
        ])
        .output()
        .context("Failed to run git")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // A repository with no commits yet has nothing to violate.
        if stderr.contains("does not have any commits") {
            return Ok(Vec::new());
        }
        anyhow::bail!("git log failed: {}", stderr.trim());
    }
    Ok(parse_violations(&String::from_utf8_lossy(&output.stdout)))
}

pub fn run() -> Result<()> {
    let violations = check_append_only()?;
    if violations.is_empty() {
        println!("OK: no committed journal entry has been modified or deleted.");
        return Ok(());
    }
    for v in &violations {
        let kind = match v.change {
            'M' => "modified",
            'D' => "deleted",
            'T' => "type-changed",
            _ => "altered",
        };
        let short = &v.commit[..v.commit.len().min(12)];
        eprintln!("FAIL: {} {} in commit {}", v.path, kind, short);
    }
    anyhow::bail!(
        "{} append-only violation(s) found in journal history",
        violations.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_modify_and_delete_but_not_add() {
        let log = "\u{1}aaa\nA\tjournal/a.md\n\u{1}bbb\nM\tjournal/a.md\nD\tjournal/b.md\nM\tstate/x.md\n";
        let v = parse_violations(log);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].change, 'M');
        assert_eq!(v[1].change, 'D');
        assert_eq!(v[1].commit, "bbb");
    }
}
