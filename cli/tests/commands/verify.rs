// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

use gitehr::commands::verify::check_append_only;

fn git(args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} failed");
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
