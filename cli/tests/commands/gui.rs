// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use serial_test::serial;
use std::fs;
use tempfile::tempdir;

use gitehr::commands::gui::find_gui_binary;

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    temp_dir
}

/// Prepends `dir` to `$PATH` and returns the previous value, so the caller
/// can restore it once the test is done with it.
fn prepend_to_path(dir: &std::path::Path) -> String {
    let original = std::env::var("PATH").unwrap_or_default();
    let mut paths = vec![dir.to_path_buf()];
    paths.extend(std::env::split_paths(&original));
    let joined = std::env::join_paths(paths).unwrap();
    unsafe { std::env::set_var("PATH", &joined) };
    original
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
#[serial]
fn find_gui_binary_returns_none_when_absent() {
    let _temp_dir = setup();

    // Isolate from the real $PATH so a developer's own gitehr-gui install
    // can't make this test flaky.
    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe { std::env::set_var("PATH", "") };

    assert!(find_gui_binary().is_none());

    unsafe { std::env::set_var("PATH", original_path) };
}

#[test]
#[serial]
fn find_gui_binary_prefers_bundled_path() {
    let temp_dir = setup();

    fs::create_dir_all(".gitehr").unwrap();
    fs::write(".gitehr/gitehr-gui", b"fake binary").unwrap();

    let found = find_gui_binary().expect("bundled binary should be found");
    assert_eq!(found, std::path::Path::new(".gitehr/gitehr-gui"));

    drop(temp_dir);
}

#[test]
#[serial]
#[cfg(unix)]
fn find_gui_binary_falls_back_to_path() {
    let temp_dir = setup();

    let bin_dir = tempdir().unwrap();
    let bin_path = bin_dir.path().join("gitehr-gui");
    fs::write(&bin_path, b"#!/bin/sh\n").unwrap();
    make_executable(&bin_path);

    let original_path = prepend_to_path(bin_dir.path());

    let found = find_gui_binary().expect("PATH binary should be found");
    assert_eq!(found, bin_path);

    unsafe { std::env::set_var("PATH", original_path) };
    drop(temp_dir);
}
