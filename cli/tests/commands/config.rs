// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::Command;
use tempfile::tempdir;

fn gitehr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitehr"))
}

#[test]
fn config_set_store_writes_toml_and_show_resolves_it() {
    let store = tempdir().unwrap();
    std::fs::write(store.path().join("gitehr-mpi.json"), r#"{"patients":[]}"#).unwrap();
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    let out = gitehr()
        .args(["config", "set-store", store.path().to_str().unwrap()])
        .env("GITEHR_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");

    let config = std::fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("store_path"));
    assert!(config.contains(store.path().to_str().unwrap()));

    let out = gitehr()
        .args(["config", "show"])
        .env("GITEHR_CONFIG", &config_path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("config_path:"));
    assert!(stdout.contains(store.path().to_str().unwrap()));
}

#[test]
fn store_commands_fall_back_to_configured_store() {
    let store = tempdir().unwrap();
    std::fs::write(
        store.path().join("gitehr-mpi.json"),
        r#"{"version":1,"updated_at":"2026-06-27T00:00:00Z","patients":[]}"#,
    )
    .unwrap();
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        format!("store_path = \"{}\"\n", store.path().display()),
    )
    .unwrap();

    let cwd = tempdir().unwrap();
    let out = gitehr()
        .args(["store", "list"])
        .current_dir(cwd.path())
        .env("GITEHR_CONFIG", &config_path)
        .output()
        .unwrap();

    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("This Store has no subjects yet"),
        "{out:?}"
    );
}
