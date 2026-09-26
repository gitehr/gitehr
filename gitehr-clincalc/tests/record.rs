// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! End-to-end test that `gitehr-clincalc record` shells out to
//! `gitehr journal add` and `gitehr state set` with the expected arguments,
//! driving the real `gitehr-clincalc` binary against a stub `gitehr`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn gitehr_clincalc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr-clincalc"))
}

/// A stub `gitehr` that appends every invocation's argv as one line to
/// `log_path` and discards stdin, so `journal add --file -` still succeeds.
fn write_stub_gitehr(dir: &Path, log_path: &Path) {
    let script = format!(
        "#!/bin/sh\necho \"$*\" >> {}\ncat > /dev/null\n",
        log_path.display()
    );
    let p = dir.join("gitehr");
    fs::write(&p, script).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn path_with(dir: &Path) -> String {
    let cur = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), cur)
}

#[test]
fn record_writes_a_journal_entry_and_the_latest_calculation_state() {
    let bin_dir = tempdir().unwrap();
    let log = bin_dir.path().join("calls.log");
    write_stub_gitehr(bin_dir.path(), &log);

    let out = gitehr_clincalc()
        .args([
            "record",
            "feverpain",
            "--input",
            r#"{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}"#,
        ])
        .env("PATH", path_with(bin_dir.path()))
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "record failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let calls = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = calls.lines().collect();
    assert_eq!(lines.len(), 2, "expected two `gitehr` calls, got: {calls}");

    assert_eq!(lines[0], "journal add --file -");

    assert!(
        lines[1].starts_with("state set calculations/feverpain-latest.json "),
        "unexpected second call: {}",
        lines[1]
    );
    assert!(lines[1].contains(r#""calculator":"feverpain""#));
    assert!(lines[1].contains(r#""recorded_at""#));
}

#[test]
fn record_does_not_touch_state_when_the_calculator_is_unknown() {
    let bin_dir = tempdir().unwrap();
    let log = bin_dir.path().join("calls.log");
    write_stub_gitehr(bin_dir.path(), &log);

    let out = gitehr_clincalc()
        .args(["record", "not-a-real-calculator", "--input", "{}"])
        .env("PATH", path_with(bin_dir.path()))
        .output()
        .unwrap();

    assert!(!out.status.success());
    assert!(!log.exists(), "gitehr should never have been invoked");
}
