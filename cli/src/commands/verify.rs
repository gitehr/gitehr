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

/// Parse NUL-delimited `git log --name-status --no-renames` output into the
/// journal changes that are not additions. `-z` preserves unusual pathnames
/// instead of Git's quoted display form, so a pathname cannot evade the check.
fn parse_violations(log: &[u8]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut commit = String::new();
    let mut pending_change = None;
    for field in log.split(|byte| *byte == b'\0') {
        if let Some(change) = pending_change.take() {
            if change != 'A' && field.starts_with(b"journal/") {
                violations.push(Violation {
                    commit: commit.clone(),
                    change,
                    path: String::from_utf8_lossy(field).into_owned(),
                });
            }
            continue;
        }
        if let Some(hash) = field.strip_prefix(b"\x01") {
            commit = String::from_utf8_lossy(hash).trim().to_string();
            continue;
        }
        if field.is_empty() {
            continue;
        }
        pending_change = field
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            .map(char::from);
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
            "-z",
            "--format=%x01%H%x00",
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
    Ok(parse_violations(&output.stdout))
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
        let log = b"\x01aaa\0\0\nA\0journal/a.md\0\x01bbb\0\0\nM\0journal/a.md\0\nD\0journal/b.md\0\nM\0state/x.md\0";
        let v = parse_violations(log);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].change, 'M');
        assert_eq!(v[1].change, 'D');
        assert_eq!(v[1].commit, "bbb");
    }

    #[test]
    fn flags_a_journal_path_with_a_newline() {
        let log = b"\x01aaa\0\0\nM\0journal/entry\nwith-newline.md\0";
        let v = parse_violations(log);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].path, "journal/entry\nwith-newline.md");
    }
}
