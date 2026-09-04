// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

/// The bundled GUI binary path for this platform, if one exists in `.gitehr/`.
fn bundled_gui_path() -> Option<PathBuf> {
    let bundled_path = PathBuf::from(".gitehr/gitehr-gui");
    if bundled_path.exists() {
        return Some(bundled_path);
    }

    #[cfg(target_os = "windows")]
    {
        let bundled_exe = PathBuf::from(".gitehr/gitehr-gui.exe");
        if bundled_exe.exists() {
            return Some(bundled_exe);
        }
    }

    None
}

/// Locates a bundled or installed GUI binary, preferring the bundled path.
///
/// This is a discovery helper only, used for the `find_gui_binary().is_some()`
/// style checks callers outside this module may want. `run()` does not launch
/// a bundled binary found this way unless `allow_bundled` is set - see its
/// doc comment for why.
pub fn find_gui_binary() -> Option<PathBuf> {
    bundled_gui_path().or_else(|| which::which("gitehr-gui").ok())
}

/// Launch the GitEHR GUI application.
///
/// Prefers an installed `gitehr-gui` on `$PATH`. Only falls back to a bundled
/// binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows)
/// when `allow_bundled` is set: a GitEHR repository received from another
/// party (a clone or a transport archive) can carry an untrusted executable
/// at that path, so launching it automatically - without the caller opting
/// in - would be arbitrary code execution with the caller's privileges (see
/// roadmap R78). If no binary can be launched, returns an error carrying
/// guidance, so that callers and scripts see a non-zero exit. For running the
/// GUI from source during development, use `s/gui-dev` instead of this
/// command.
pub fn run(allow_bundled: bool) -> Result<()> {
    if !is_gitehr_repo() {
        eprintln!("Warning: Not in a GitEHR repository. Opening GUI without repository context.");
    }

    if allow_bundled && let Some(path) = bundled_gui_path() {
        return launch(&path);
    }

    if let Ok(path) = which::which("gitehr-gui") {
        return launch(&path);
    }

    match find_gui_binary() {
        Some(path) => anyhow::bail!(
            "Found a bundled GUI binary at {} but did not launch it.\n\n\
             A GitEHR repository received from another party (a clone or transport \
             archive) can carry an untrusted executable at .gitehr/gitehr-gui; launching \
             it automatically would run arbitrary code with your privileges. Re-run \
             `gitehr gui --allow-bundled` if you trust this repository's origin, or \
             install gitehr-gui on $PATH instead.",
            path.display()
        ),
        None => anyhow::bail!(
            "No GitEHR GUI binary found.\n\n\
             Looked for gitehr-gui on $PATH and a bundled binary at .gitehr/gitehr-gui.\n\
             To install the GUI, see https://gitehr.org/install/gui/\n\
             To build and run it from source, run `s/gui-dev` from the repository root."
        ),
    }
}

fn launch(path: &Path) -> Result<()> {
    let status = Command::new(path).status()?;
    if !status.success() {
        anyhow::bail!("GUI exited with an error.");
    }
    Ok(())
}
