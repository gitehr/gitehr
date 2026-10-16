// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Client-side policy check over Git history (R40, see
//! `spec/repository-verification.md`). Asserts two invariants: no commit ever
//! modified, deleted, or renamed a committed journal entry (ADR-0002, the
//! record only grows), and journal entries are added in timestamp order
//! (a new entry is never named earlier than one already in the record). It
//! also flags likely history rewrites against the branch's upstream:
//! divergence from it, or a forced update of its remote-tracking ref. Object
//! integrity is left to `git fsck`.

use anyhow::{Context, Result};
use std::process::{Command, Output};

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

#[derive(Debug, PartialEq, Eq)]
pub struct OrderViolation {
    pub commit: String,
    pub path: String,
    pub latest_earlier: String,
}

/// The fixed-width UTC timestamp a journal filename starts with
/// (`%Y%m%dT%H%M%S%.3fZ-<uuid>.md`), or `None` for any other name.
fn entry_timestamp(name: &str) -> Option<&str> {
    let ts = name.split('-').next()?;
    (ts.len() == 20 && ts.ends_with('Z') && ts.as_bytes()[8] == b'T').then_some(ts)
}

/// Parse NUL-delimited `git log --name-only` output (oldest first) and report
/// each added entry whose timestamp precedes one added by an earlier commit.
/// Entries added by the same commit are not compared with each other.
fn parse_order_violations(log: &[u8]) -> Vec<OrderViolation> {
    let mut violations = Vec::new();
    let mut commit = String::new();
    let mut latest = String::new(); // newest timestamp from earlier commits
    let mut latest_in_commit = String::new();
    for field in log.split(|byte| *byte == b'\0') {
        if let Some(hash) = field.strip_prefix(b"\x01") {
            if latest_in_commit > latest {
                latest = std::mem::take(&mut latest_in_commit);
            }
            latest_in_commit.clear();
            commit = String::from_utf8_lossy(hash).trim().to_string();
            continue;
        }
        let field = field.strip_prefix(b"\n").unwrap_or(field);
        let Some(name) = field.strip_prefix(b"journal/") else {
            continue;
        };
        let name = String::from_utf8_lossy(name);
        let Some(ts) = entry_timestamp(&name) else {
            continue;
        };
        if ts < latest.as_str() {
            violations.push(OrderViolation {
                commit: commit.clone(),
                path: format!("journal/{name}"),
                latest_earlier: latest.clone(),
            });
        }
        if ts > latest_in_commit.as_str() {
            latest_in_commit = ts.to_string();
        }
    }
    violations
}

/// Walk the first-parent history and return every journal entry added with a
/// timestamp earlier than one already in the record. Merge commits are diffed
/// against their first parent, so entries brought in by a merge are checked
/// when they land; a genuinely concurrent writer can therefore be reported.
pub fn check_monotonic_order() -> Result<Vec<OrderViolation>> {
    let output = git_output(&[
        "log",
        "--first-parent",
        "-m",
        "--reverse",
        "--diff-filter=A",
        "--name-only",
        "-z",
        "--format=%x01%H%x00",
        "--",
        "journal",
    ])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have any commits") {
            return Ok(Vec::new());
        }
        anyhow::bail!("git log failed: {}", stderr.trim());
    }
    Ok(parse_order_violations(&output.stdout))
}

fn git_output(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))
}

fn git_stdout(args: &[&str]) -> Result<String> {
    let output = git_output(args)?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn upstream() -> Result<Option<String>> {
    let output = git_output(&["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    if output.status.code() == Some(1) {
        return Ok(None);
    }
    if !output.status.success() {
        anyhow::bail!(
            "git symbolic-ref failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let upstream = git_stdout(&[
        "for-each-ref",
        "--format=%(upstream:short)",
        &format!("refs/heads/{branch}"),
    ])?;
    Ok((!upstream.is_empty()).then_some(upstream))
}

fn is_ancestor(ancestor: &str, descendant: &str) -> Result<bool> {
    let output = git_output(&["merge-base", "--is-ancestor", ancestor, descendant])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => anyhow::bail!(
            "git merge-base failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

/// Detect signs of a history rewrite relative to the current branch's
/// upstream. Returns one human-readable finding per problem; a branch with no
/// upstream (or a repository with no commits) has nothing to compare and
/// yields none.
pub fn check_history_rewrite() -> Result<Vec<String>> {
    let mut findings = Vec::new();
    let Some(upstream) = upstream()? else {
        return Ok(findings);
    };
    if !is_ancestor(&upstream, "HEAD")? && !is_ancestor("HEAD", &upstream)? {
        findings.push(format!(
            "HEAD and {upstream} have diverged (history was rewritten or written concurrently)"
        ));
    }
    let reflog = git_stdout(&["reflog", "show", "--format=%gs", &upstream])?;
    if reflog.lines().any(|l| l.contains("forced-update")) {
        findings.push(format!(
            "{upstream} was force-updated at some point (its reflog records a forced-update)"
        ));
    }
    Ok(findings)
}

pub fn run() -> Result<()> {
    let violations = check_append_only()?;
    let rewrites = check_history_rewrite()?;
    let misordered = check_monotonic_order()?;
    for v in &misordered {
        let short = &v.commit[..v.commit.len().min(12)];
        eprintln!(
            "FAIL: {} added out of order in commit {} (an entry stamped {} already exists)",
            v.path, short, v.latest_earlier
        );
    }
    for finding in &rewrites {
        eprintln!("FAIL: {finding}");
    }
    if violations.is_empty() && rewrites.is_empty() && misordered.is_empty() {
        println!(
            "OK: no committed journal entry has been modified or deleted, and entries are in order."
        );
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
        "{} append-only violation(s), {} history-rewrite finding(s) and {} ordering violation(s)",
        violations.len(),
        rewrites.len(),
        misordered.len()
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

    const A: &str = "20260101T000000.000Z-aaaa.md";
    const B: &str = "20260102T000000.000Z-bbbb.md";

    #[test]
    fn accepts_entries_added_in_order() {
        let log = format!("\x01c1\0\0\njournal/{A}\0\x01c2\0\0\njournal/{B}\0");
        assert!(parse_order_violations(log.as_bytes()).is_empty());
    }

    #[test]
    fn flags_a_backdated_entry() {
        let log = format!("\x01c1\0\0\njournal/{B}\0\x01c2\0\0\njournal/{A}\0");
        let v = parse_order_violations(log.as_bytes());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].commit, "c2");
        assert_eq!(v[0].path, format!("journal/{A}"));
        assert_eq!(v[0].latest_earlier, &B[..20]);
    }

    #[test]
    fn does_not_compare_entries_within_one_commit() {
        let log = format!("\x01c1\0\0\njournal/{B}\0journal/{A}\0");
        assert!(parse_order_violations(log.as_bytes()).is_empty());
    }

    #[test]
    fn flags_a_journal_path_with_a_newline() {
        let log = b"\x01aaa\0\0\nM\0journal/entry\nwith-newline.md\0";
        let v = parse_violations(log);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].path, "journal/entry\nwith-newline.md");
    }
}
