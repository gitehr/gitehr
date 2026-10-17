// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;

use super::{medmd, parse_journal_file};

/// Print the Medical Markdown structure of an entry as JSON. Read-only: the
/// entry on disk is never modified (ADR-0004).
pub fn run(filename: String) -> Result<()> {
    let filename = super::resolve_entry(&filename)?;
    let path = PathBuf::from("journal").join(&filename);

    if !path.exists() {
        anyhow::bail!("Journal entry not found: {}", filename);
    }

    let parsed = parse_journal_file(&path)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&medmd::extract(&parsed.content))?
    );
    Ok(())
}
