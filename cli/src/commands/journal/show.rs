// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::{fs, path::PathBuf};

use super::{medmd, parse_journal_file};

pub fn run(filename: String, raw: bool, metadata: bool, structured: bool) -> Result<()> {
    let filename = super::resolve_entry(&filename)?;
    let path = PathBuf::from("journal").join(&filename);

    if !path.exists() {
        anyhow::bail!("Journal entry not found: {}", filename);
    }

    if raw {
        let content = fs::read_to_string(&path)?;
        print!("{content}");
        return Ok(());
    }

    if metadata {
        let content = fs::read_to_string(&path)?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!("No frontmatter found in: {}", filename);
        }
        println!("---{}---", parts[1]);
        return Ok(());
    }

    if structured {
        let parsed = parse_journal_file(&path)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&medmd::extract(&parsed.content))?
        );
        return Ok(());
    }

    // default: body only
    let parsed = parse_journal_file(&path)?;
    println!("{}", parsed.content);
    Ok(())
}
