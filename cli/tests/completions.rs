// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::Command;

#[test]
fn completions_generate_for_every_supported_shell() {
    let bin = env!("CARGO_BIN_EXE_gitehr");
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = Command::new(bin)
            .args(["completions", shell])
            .output()
            .unwrap_or_else(|error| panic!("run gitehr completions {shell}: {error}"));
        assert!(output.status.success(), "generate {shell} completions");
        let script = String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("{shell} completions are UTF-8: {error}"));
        assert!(
            script.contains("completions"),
            "{shell} completions include the completions command"
        );
    }
}

#[test]
fn completions_install_writes_shell_filename() {
    let bin = env!("CARGO_BIN_EXE_gitehr");
    let out_dir = tempfile::tempdir().expect("create completion directory");

    let output = Command::new(bin)
        .args([
            "completions",
            "install",
            "--shell",
            "zsh",
            "--dir",
            out_dir.path().to_str().expect("temp path"),
        ])
        .output()
        .expect("run gitehr completions install");
    assert!(output.status.success());
    assert!(out_dir.path().join("_gitehr").exists());
}
