// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod acquisitions;
pub mod allergies;
pub mod conditions;
pub mod config;
pub mod contributor;
pub mod demographics;
pub mod document;
pub mod encrypt;
pub mod family_history;
pub mod gui;
pub mod import;
pub mod journal;
pub mod mcp;
pub mod medications;
pub mod observations;
#[cfg(unix)]
pub mod plugin;
pub mod remote;
pub mod state;
pub mod status;
pub mod store;
pub mod transport;
pub mod upgrade;
pub mod vaccinations;
pub mod verify;

/// Make every later commit in the current directory's repository fail, by
/// installing a `pre-commit` hook that exits non-zero.
pub fn fail_later_commits() -> anyhow::Result<()> {
    let hook = std::path::Path::new(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap())?;
    std::fs::write(hook, "#!/bin/sh\nexit 1\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(hook, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}
