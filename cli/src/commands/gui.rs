// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

/// Locates a bundled or installed GUI binary for the release launch path
/// described in `run()`.
pub fn find_gui_binary() -> Option<PathBuf> {
    let bundled_path = PathBuf::from(".gitehr/gitehr-gui");
    if bundled_path.exists() {
        return Some(bundled_path);
    }

    #[cfg(target_os = "windows")]
    let bundled_exe = PathBuf::from(".gitehr/gitehr-gui.exe");
    #[cfg(target_os = "windows")]
    if bundled_exe.exists() {
        return Some(bundled_exe);
    }

    if let Ok(path) = which::which("gitehr-gui") {
        return Some(path);
    }

    None
}

/// Launch the GitEHR GUI application.
///
/// Prefers a bundled GUI binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe`
/// on Windows), then falls back to `gitehr-gui` on `$PATH`. If neither is found,
/// prints guidance on how to install or build one. For running the GUI from
/// source during development, use `s/gui-dev` instead of this command.
pub fn run() -> Result<()> {
    if !is_gitehr_repo() {
        println!("Warning: Not in a GitEHR repository. Opening GUI without repository context.");
    }

    match find_gui_binary() {
        Some(path) => {
            let status = Command::new(path).status()?;
            if !status.success() {
                anyhow::bail!("GUI exited with an error.");
            }
            Ok(())
        }
        None => {
            println!("No GitEHR GUI binary found.");
            println!();
            println!(
                "Looked for a bundled binary at .gitehr/gitehr-gui and for gitehr-gui on $PATH."
            );
            println!("To install the GUI, see https://gitehr.org/install/gui/");
            println!("To build and run it from source, run `s/gui-dev` from the repository root.");
            Ok(())
        }
    }
}
