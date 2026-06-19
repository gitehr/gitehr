use anyhow::Result;
use std::{fs, path::PathBuf};

use super::parse_journal_file;

pub fn run(filename: String, drafts: bool) -> Result<()> {
    if drafts {
        let path = PathBuf::from("tmp/journal").join(&filename);
        if !path.exists() {
            anyhow::bail!("Draft not found: {}", filename);
        }
        let content = fs::read_to_string(&path)?;
        print!("{}", content);
    } else {
        let path = PathBuf::from("journal").join(&filename);
        if !path.exists() {
            anyhow::bail!("Journal entry not found: {}", filename);
        }
        let parsed = parse_journal_file(&path)?;
        println!("{}", parsed.content);
    }
    Ok(())
}
