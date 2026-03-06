// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

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
pub fn launch_gui() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    if !is_gitehr_repo() {
        println!("Warning: Not in a GitEHR repository. Opening GUI without repository context.");
    }
    // Development mode: run tauri dev
    let status = Command::new("npm")
        .arg("run")
        .arg("tauri")
        .arg("dev")
        .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
        .current_dir("gui/gitehr-gui")
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to launch GUI in dev mode.");
    }
    Ok(())
}
