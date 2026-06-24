// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

use super::{is_gitehr_repo, view_state_file};

pub fn run(filename: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let file = view_state_file(filename)?;
    println!("{}", file.content);

    Ok(())
}
