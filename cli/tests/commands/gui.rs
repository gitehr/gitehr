// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use serial_test::serial;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use gitehr::commands::gui::{self, find_gui_binary};

/// Restores the process-wide current directory and `$PATH` on drop.
///
/// `find_gui_binary` resolves against both, so these tests have to mutate
/// process-global state. Doing the restore in `Drop` rather than at the end of
/// each test means a failing assertion cannot leak a stripped `$PATH` or a
/// deleted working directory into the rest of the suite.
struct EnvGuard {
    original_dir: PathBuf,
    original_path: OsString,
}

impl EnvGuard {
    /// Capture the current state. Construct this *before* changing directory.
    ///
    /// `current_dir` can legitimately fail here: other tests in this suite
    /// chdir into a temp directory and let it be deleted without restoring,
    /// which leaves the process cwd pointing at a path that no longer exists.
    /// Fall back to the crate root so we still restore somewhere valid.
    fn new() -> Self {
        Self {
            original_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
            original_path: std::env::var_os("PATH").unwrap_or_default(),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // Deliberately not unwrapping: panicking in Drop during an already
        // failing test would mask the original assertion failure.
        let _ = std::env::set_current_dir(&self.original_dir);
        unsafe { std::env::set_var("PATH", &self.original_path) };
    }
}

/// Enters an isolated temp directory with an empty `$PATH`.
///
/// Returns the temp dir and the guard. Bind them as `(_dir, _guard)` so the
/// guard drops first and restores the cwd before the directory is removed.
fn isolated() -> (tempfile::TempDir, EnvGuard) {
    let temp_dir = tempdir().unwrap();
    let guard = EnvGuard::new();
    std::env::set_current_dir(&temp_dir).unwrap();
    // Empty PATH so a developer's own gitehr-gui install cannot make these
    // tests pass or fail by accident.
    unsafe { std::env::set_var("PATH", "") };
    (temp_dir, guard)
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
#[serial]
fn find_gui_binary_returns_none_when_absent() {
    let (_dir, _guard) = isolated();

    assert!(find_gui_binary().is_none());
}

#[test]
#[serial]
fn find_gui_binary_prefers_bundled_path() {
    let (_dir, _guard) = isolated();

    fs::create_dir_all(".gitehr").unwrap();
    fs::write(".gitehr/gitehr-gui", b"fake binary").unwrap();

    let found = find_gui_binary().expect("bundled binary should be found");
    assert_eq!(found, Path::new(".gitehr/gitehr-gui"));
}

#[test]
#[serial]
#[cfg(unix)]
fn find_gui_binary_prefers_bundled_over_path() {
    let (_dir, _guard) = isolated();

    // A bundled binary and a $PATH binary both exist; bundled must win.
    fs::create_dir_all(".gitehr").unwrap();
    fs::write(".gitehr/gitehr-gui", b"fake binary").unwrap();

    let bin_dir = tempdir().unwrap();
    let bin_path = bin_dir.path().join("gitehr-gui");
    fs::write(&bin_path, b"#!/bin/sh\n").unwrap();
    make_executable(&bin_path);
    unsafe { std::env::set_var("PATH", bin_dir.path()) };

    let found = find_gui_binary().expect("bundled binary should be found");
    assert_eq!(found, Path::new(".gitehr/gitehr-gui"));
}

#[test]
#[serial]
#[cfg(unix)]
fn find_gui_binary_falls_back_to_path() {
    let (_dir, _guard) = isolated();

    let bin_dir = tempdir().unwrap();
    let bin_path = bin_dir.path().join("gitehr-gui");
    fs::write(&bin_path, b"#!/bin/sh\n").unwrap();
    make_executable(&bin_path);
    unsafe { std::env::set_var("PATH", bin_dir.path()) };

    let found = find_gui_binary().expect("PATH binary should be found");
    assert_eq!(found, bin_path);
}

/// Regression test: a missing GUI must be a non-zero exit, not a silent
/// success, so scripts wrapping `gitehr gui` can detect the failure.
#[test]
#[serial]
fn run_errors_when_no_binary_found() {
    let (_dir, _guard) = isolated();

    let err = gui::run(false).expect_err("run() should fail when no GUI binary is present");
    assert!(
        err.to_string().contains("No GitEHR GUI binary found"),
        "error should explain the failure, got: {err}"
    );
}

/// A GitEHR repository received from another party can carry an untrusted
/// `.gitehr/gitehr-gui`; `run()` must not launch it without an explicit
/// `--allow-bundled` opt-in (roadmap R78).
#[test]
#[serial]
#[cfg(unix)]
fn run_refuses_bundled_binary_without_allow_bundled() {
    let (_dir, _guard) = isolated();

    fs::create_dir_all(".gitehr").unwrap();
    let bundled = PathBuf::from(".gitehr/gitehr-gui");
    fs::write(&bundled, "#!/bin/sh\nexit 0\n").unwrap();
    make_executable(&bundled);

    let err = gui::run(false).expect_err("run() should refuse an unallowed bundled binary");
    assert!(
        err.to_string().contains("--allow-bundled"),
        "error should point at the opt-in flag, got: {err}"
    );
}

#[test]
#[serial]
#[cfg(unix)]
fn run_launches_bundled_binary_with_allow_bundled() {
    let (_dir, _guard) = isolated();

    fs::create_dir_all(".gitehr").unwrap();
    let bundled = PathBuf::from(".gitehr/gitehr-gui");
    fs::write(&bundled, "#!/bin/sh\nexit 0\n").unwrap();
    make_executable(&bundled);

    gui::run(true).expect("run() should launch an explicitly allowed bundled binary");
}

/// Even without `--allow-bundled`, a trusted `$PATH` install should still be
/// usable, and must be preferred over the untrusted bundled binary.
#[test]
#[serial]
#[cfg(unix)]
fn run_prefers_path_binary_over_unallowed_bundled() {
    let (_dir, _guard) = isolated();

    fs::create_dir_all(".gitehr").unwrap();
    let bundled = PathBuf::from(".gitehr/gitehr-gui");
    fs::write(&bundled, "#!/bin/sh\n> ran-bundled\nexit 0\n").unwrap();
    make_executable(&bundled);

    let bin_dir = tempdir().unwrap();
    let bin_path = bin_dir.path().join("gitehr-gui");
    fs::write(&bin_path, "#!/bin/sh\n> ran-path\nexit 0\n").unwrap();
    make_executable(&bin_path);
    unsafe { std::env::set_var("PATH", bin_dir.path()) };

    gui::run(false).expect("run() should launch the $PATH binary");

    assert!(
        Path::new("ran-path").exists(),
        "the $PATH binary should have run"
    );
    assert!(
        !Path::new("ran-bundled").exists(),
        "the unallowed bundled binary must not have run"
    );
}
