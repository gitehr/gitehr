// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! End-to-end tests for the `$PATH` plugin dispatch, driving the real binary.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn gitehr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr"))
}

fn write_plugin(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

/// PATH with `dir` prepended, so the test's plugins are found first.
fn path_with(dir: &Path) -> String {
    let cur = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), cur)
}

#[test]
fn dispatches_to_plugin_and_passes_args_through() {
    let dir = tempdir().unwrap();
    write_plugin(dir.path(), "gitehr-greet", "#!/bin/sh\necho \"greet:$*\"\n");

    let out = gitehr()
        .args(["greet", "alpha", "beta"])
        .env("PATH", path_with(dir.path()))
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "greet:alpha beta"
    );
}

#[test]
fn builtin_wins_over_a_shadowing_plugin() {
    let dir = tempdir().unwrap();
    // A decoy that would run if a plugin could shadow the built-in `journal`.
    write_plugin(dir.path(), "gitehr-journal", "#!/bin/sh\necho DECOY-RAN\n");

    let out = gitehr()
        .args(["journal", "show", "x"]) // built-in; errors (not in a repo here)
        .env("PATH", path_with(dir.path()))
        .output()
        .unwrap();

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("DECOY-RAN"),
        "a plugin must never shadow a built-in; got: {combined}"
    );
}

#[test]
fn unknown_subcommand_without_a_plugin_errors_helpfully() {
    let dir = tempdir().unwrap(); // empty: no plugins
    let out = gitehr()
        .arg("definitelynotacommand")
        .env("PATH", path_with(dir.path()))
        .output()
        .unwrap();

    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unrecognized subcommand"));
    assert!(err.contains("gitehr plugins"));
}

#[test]
fn plugins_lists_installed_and_excludes_builtins() {
    let dir = tempdir().unwrap();
    write_plugin(dir.path(), "gitehr-export", "#!/bin/sh\n");
    write_plugin(dir.path(), "gitehr-status", "#!/bin/sh\n"); // `status` is a built-in

    let out = gitehr()
        .arg("plugins")
        .env("PATH", path_with(dir.path()))
        .output()
        .unwrap();

    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("export"), "installed plugin should be listed");
    assert!(
        !s.contains("status"),
        "a name shadowed by a built-in must not be listed as a plugin"
    );
}

#[test]
fn plugin_exit_code_propagates() {
    let dir = tempdir().unwrap();
    write_plugin(dir.path(), "gitehr-boom", "#!/bin/sh\nexit 3\n");

    let out = gitehr()
        .arg("boom")
        .env("PATH", path_with(dir.path()))
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(3));
}
