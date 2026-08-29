// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;

fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn is_encrypted() -> bool {
    PathBuf::from(".gitehr/ENCRYPTED").exists()
}

pub fn run(_key_source: Option<&str>) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    if is_encrypted() {
        anyhow::bail!("Repository is already encrypted.");
    }

    // Refusing outright is deliberate (roadmap R79): the earlier placeholder
    // wrote the ENCRYPTED marker while leaving every clinical file plaintext,
    // which falsely assured users their data was protected.
    anyhow::bail!(
        "Encryption at rest is not yet implemented (roadmap R67/R68). \
Refusing to mark the repository as encrypted; no files were changed."
    );
}
