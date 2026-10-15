// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

use gitehr::commands::verify::{check_append_only, check_history_rewrite};

fn git(args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
        ])
        .args(args)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} failed");
}

#[test]
#[serial]
fn detects_changes_to_a_journal_path_git_would_quote() -> Result<()> {
    let dir = tempdir()?;
    std::env::set_current_dir(&dir)?;
    git(&["init", "-q"]);

    fs::create_dir("journal")?;
    let path = "journal/entry\nwith-newline.md";
    fs::write(path, "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "add"]);

    fs::write(path, "changed")?;
    git(&["commit", "-qam", "edit"]);

    let violations = check_append_only()?;
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].change, 'M');
    assert_eq!(violations[0].path, path);
    Ok(())
}

#[test]
#[serial]
fn detects_renamed_entries() -> Result<()> {
    let dir = tempdir()?;
    std::env::set_current_dir(&dir)?;
    git(&["init", "-q"]);

    fs::create_dir("journal")?;
    fs::write("journal/original.md", "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "add"]);

    fs::rename("journal/original.md", "journal/renamed.md")?;
    git(&["add", "-A"]);
    git(&["commit", "-qm", "rename"]);

    let violations = check_append_only()?;
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].change, 'D');
    assert_eq!(violations[0].path, "journal/original.md");
    Ok(())
}

#[test]
#[serial]
fn detects_modified_and_deleted_entries() -> Result<()> {
    let dir = tempdir()?;
    std::env::set_current_dir(&dir)?;
    git(&["init", "-q"]);
    assert!(check_append_only()?.is_empty());

    fs::create_dir("journal")?;
    fs::write("journal/a.md", "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "add"]);
    assert!(check_append_only()?.is_empty());

    fs::write("journal/b.md", "two")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "add b"]);
    assert!(check_append_only()?.is_empty());

    fs::write("journal/a.md", "changed")?;
    git(&["commit", "-qam", "edit"]);
    fs::remove_file("journal/b.md")?;
    git(&["commit", "-qam", "delete"]);

    let v = check_append_only()?;
    assert_eq!(v.len(), 2);
    assert_eq!((v[0].change, v[0].path.as_str()), ('M', "journal/a.md"));
    assert_eq!((v[1].change, v[1].path.as_str()), ('D', "journal/b.md"));
    Ok(())
}

#[test]
#[serial]
fn no_upstream_means_no_rewrite_findings() -> Result<()> {
    let dir = tempdir()?;
    std::env::set_current_dir(&dir)?;
    git(&["init", "-q"]);
    assert!(check_history_rewrite()?.is_empty());
    fs::write("a.md", "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "add"]);
    assert!(check_history_rewrite()?.is_empty());
    Ok(())
}

#[test]
#[serial]
fn detects_divergence_from_upstream_after_a_rewrite() -> Result<()> {
    let dir = tempdir()?;
    let remote = dir.path().join("remote.git");
    let work = dir.path().join("work");
    git(&["init", "-q", "--bare", remote.to_str().unwrap()]);
    git(&["init", "-q", work.to_str().unwrap()]);
    std::env::set_current_dir(&work)?;
    git(&["checkout", "-qb", "main"]);
    fs::write("a.md", "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "one"]);
    fs::write("a.md", "two")?;
    git(&["commit", "-qam", "two"]);
    git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&["push", "-q", "-u", "origin", "main"]);
    assert!(check_history_rewrite()?.is_empty());

    // Rewrite the last commit locally: now diverged from origin/main.
    fs::write("a.md", "rewritten")?;
    git(&["commit", "-qa", "--amend", "-m", "two'"]);
    let findings = check_history_rewrite()?;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("diverged"));
    Ok(())
}

#[test]
#[serial]
fn detects_a_forced_update_of_the_upstream_tracking_ref() -> Result<()> {
    let dir = tempdir()?;
    let remote = dir.path().join("remote.git");
    let work = dir.path().join("work");
    let writer = dir.path().join("writer");
    git(&["init", "-q", "--bare", remote.to_str().unwrap()]);
    git(&["init", "-q", work.to_str().unwrap()]);
    std::env::set_current_dir(&work)?;
    git(&["checkout", "-qb", "main"]);
    fs::write("a.md", "one")?;
    git(&["add", "."]);
    git(&["commit", "-qm", "one"]);
    fs::write("a.md", "two")?;
    git(&["commit", "-qam", "two"]);
    git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    git(&["push", "-q", "-u", "origin", "main"]);
    git(&[
        "clone",
        "-q",
        "--branch",
        "main",
        remote.to_str().unwrap(),
        writer.to_str().unwrap(),
    ]);

    std::env::set_current_dir(&writer)?;
    git(&["reset", "-q", "--hard", "HEAD~"]);
    fs::write("a.md", "rewritten")?;
    git(&["commit", "-qam", "rewrite"]);
    git(&["push", "-q", "--force", "origin", "main"]);

    std::env::set_current_dir(&work)?;
    git(&["fetch", "-q", "origin"]);
    git(&["reset", "-q", "--hard", "origin/main"]);
    let findings = check_history_rewrite()?;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("force-updated"));
    Ok(())
}
