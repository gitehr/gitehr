// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

// Locates a bundled or installed GUI binary for the release launch path
// described in `run()`. Not yet wired up (dev mode runs `npm run tauri dev`).
#[allow(dead_code)]
fn find_gui_binary() -> Option<PathBuf> {
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

/// Launch the GitEHR GUI application
/// For development, launches with: WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
/// For release, should launch the compiled, OS-appropriate GUI binary
pub fn run() -> Result<()> {
    if !is_gitehr_repo() {
        println!("Warning: Not in a GitEHR repository. Opening GUI without repository context.");
    }
    // Development mode: run tauri dev
    let status = Command::new("npm")
        .arg("run")
        .arg("tauri")
        .arg("dev")
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
        .current_dir("gui")
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to launch GUI in dev mode.");
    }
    Ok(())
}
