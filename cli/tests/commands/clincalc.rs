// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! End-to-end tests for `gitehr clincalc record` (R25), driving the real
//! binary against a fake `gitehr-clincalc` plugin on `PATH`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn gitehr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr"))
}

/// A fake `gitehr-clincalc` that ignores its arguments and prints a fixed
/// result, so tests do not depend on the real plugin being installed.
fn write_fake_clincalc(dir: &Path) {
    let script = r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "gitehr-clincalc 9.9.9"
  exit 0
fi
cat <<'EOF'
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "A score of 3 is associated with 34-40% isolation of streptococcus.",
  "working": {"score": 3},
  "reference": "Little P, et al. Lancet Infect Dis. 2014."
}
EOF
"#;
    let p = dir.join("gitehr-clincalc");
    fs::write(&p, script).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn path_with(dir: &Path) -> String {
    let cur = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), cur)
}

/// A minimal repo: `.gitehr` + `journal/`, with git configured so a commit
/// can succeed.
fn init_repo() -> tempfile::TempDir {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".gitehr")).unwrap();
    fs::create_dir(repo.path().join("journal")).unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.name", "Test User"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "commit.gpgsign", "false"],
    ] {
        Command::new("git")
            .current_dir(repo.path())
            .args(&args)
            .output()
            .unwrap();
    }
    repo
}

fn journal_files(repo: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(repo.join("journal"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect()
}

#[test]
fn record_writes_and_commits_a_clincalc_journal_entry() {
    let plugin_dir = tempdir().unwrap();
    write_fake_clincalc(plugin_dir.path());
    let repo = init_repo();

    let out = gitehr()
        .current_dir(repo.path())
        .args([
            "clincalc",
            "record",
            "feverpain",
            "--input",
            r#"{"fever":true,"purulence":true}"#,
        ])
        .env("PATH", path_with(plugin_dir.path()))
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let entries = journal_files(repo.path());
    assert_eq!(entries.len(), 1, "expected exactly one journal entry");

    let content = fs::read_to_string(&entries[0]).unwrap();
    assert!(content.contains("calculator: feverpain"));
    assert!(content.contains("version: gitehr-clincalc 9.9.9"));
    assert!(content.contains("result: 3"));
    assert!(content.contains("34-40% isolation of streptococcus"));
    assert!(content.contains("Little P, et al. Lancet Infect Dis. 2014."));
    assert!(content.contains(r#""fever": true"#) || content.contains("fever: true"));

    // The entry must be committed (per R25's "immutable entry"), unlike the
    // uncommitted MCP draft path.
    let log = Command::new("git")
        .current_dir(repo.path())
        .args(["log", "--oneline"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&log.stdout).lines().count(),
        1,
        "the clincalc record should be committed immediately"
    );
}

#[test]
fn record_reads_a_relative_input_file_from_the_original_cwd_not_the_repo_root() {
    // gitehr changes into the resolved repo root before dispatch (ADR-0005),
    // so a relative `--input <path>` must be absolutized against the caller's
    // cwd first, or a subdirectory invocation would look for the file in the
    // wrong place.
    let plugin_dir = tempdir().unwrap();
    write_fake_clincalc(plugin_dir.path());
    let repo = init_repo();

    let subdir = repo.path().join("sub");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("input.json"), r#"{"fever":true}"#).unwrap();

    let out = gitehr()
        .current_dir(&subdir)
        .args(["clincalc", "record", "feverpain", "--input", "input.json"])
        .env("PATH", path_with(plugin_dir.path()))
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(journal_files(repo.path()).len(), 1);
}

#[test]
fn record_reports_a_clear_error_when_the_plugin_is_missing() {
    let empty = tempdir().unwrap(); // no gitehr-clincalc on this PATH
    let repo = init_repo();

    let out = gitehr()
        .current_dir(repo.path())
        .args([
            "clincalc",
            "record",
            "feverpain",
            "--input",
            r#"{"fever":true}"#,
        ])
        .env("PATH", path_with(empty.path()))
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("gitehr-clincalc not found on PATH"));
    assert!(journal_files(repo.path()).is_empty());
}
