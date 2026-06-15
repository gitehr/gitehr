use anyhow::Result;

use super::{create_journal_entry, get_latest_journal_entry};

pub fn run(content: Option<String>, file: Option<String>) -> Result<()> {
    let entry_content = match (content, file) {
        (Some(text), None) => text,
        (None, Some(path)) => {
            if path == "-" {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                std::fs::read_to_string(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?
            }
        }
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both content and --file");
        }
        (None, None) => {
            anyhow::bail!("Must provide content or use --file <path>");
        }
    };

    let latest = get_latest_journal_entry()?;
    let parent_hash = latest.map(|(_, hash)| hash);
    create_journal_entry(&entry_content, parent_hash)?;
    Ok(())
}
