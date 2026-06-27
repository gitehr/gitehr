// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;

pub fn run(_repo_path: Option<PathBuf>) -> Result<()> {
    anyhow::bail!(
        "gitehr mcp serve is temporarily unavailable while the gitehr-mcp crate is prepared for release.\n\
         This keeps the CLI packageable by release-plz until gitehr-mcp is published to crates.io."
    )
}
