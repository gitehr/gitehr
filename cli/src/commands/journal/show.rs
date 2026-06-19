use anyhow::Result;
use std::{fs, path::PathBuf};

use super::parse_journal_file;

pub fn run(filename: String, drafts: bool, raw: bool, metadata: bool) -> Result<()> {
    let filename = super::resolve_entry(&filename, drafts)?;
    let path = if drafts {
        PathBuf::from("tmp/journal").join(&filename)
    } else {
        PathBuf::from("journal").join(&filename)
    };

    if !path.exists() {
        if drafts {
            anyhow::bail!("Draft not found: {}", filename);
        } else {
            anyhow::bail!("Journal entry not found: {}", filename);
        }
    }

    if raw {
        let content = fs::read_to_string(&path)?;
        print!("{}", content);
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

    // default: body only
    let parsed = parse_journal_file(&path)?;
    println!("{}", parsed.content);
    Ok(())
}
