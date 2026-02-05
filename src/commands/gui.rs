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

pub fn launch_gui() -> Result<()> {
    let current_dir = env::current_dir()?;

    if !is_gitehr_repo() {
        println!("Warning: Not in a GitEHR repository. Opening GUI without repository context.");
    }

    match find_gui_binary() {
        Some(gui_path) => {
            println!("Launching GitEHR GUI...");

            let mut cmd = Command::new(&gui_path);
            cmd.current_dir(&current_dir);

            #[cfg(target_os = "linux")]
            {
                use std::os::unix::process::CommandExt;
                let err = cmd.exec();
                anyhow::bail!("Failed to launch GUI: {}", err);
            }

            #[cfg(not(target_os = "linux"))]
            {
                cmd.spawn()?;
                Ok(())
            }
        }
        None => {
            println!("GitEHR GUI not found.");
            println!();
            println!("The GUI binary is not bundled in this repository and is not in PATH.");
            println!("To use the GUI, either:");
            println!("  1. Run 'gitehr upgrade' to bundle the latest GUI binary");
            println!("  2. Install gitehr-gui and ensure it's in your PATH");
            println!("  3. Build the GUI from source in the gui/ directory");
            Ok(())
        }
    }
}
