use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    parent_hash: Option<String>,
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
}

pub fn create_journal_entry(content: &str, parent_hash: Option<String>) -> Result<()> {
    let entry = JournalEntry {
        parent_hash,
        timestamp: Utc::now(),
        author: None, // TODO: Implement author management
    };

    // Create filename with timestamp and UUID
    let filename = format!(
        "journal/{}-{}.md",
        entry.timestamp.format("%Y%m%dT%H%M%SZ"),
        Uuid::new_v4()
    );

    // Create YAML front matter
    let yaml = serde_yaml::to_string(&entry)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, content);

    fs::write(&filename, file_content)?;
    println!("Created journal entry: {}", filename);
    Ok(())
}

pub fn get_latest_journal_entry() -> Result<Option<(String, String)>> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        return Ok(None);
    }

    let mut entries: Vec<_> = fs::read_dir(&journal_dir)?.filter_map(|e| e.ok()).collect();

    // Sort by filename (which contains timestamp)
    entries.sort_by_key(|e| e.file_name());

    if let Some(latest) = entries.last() {
        let content = fs::read_to_string(latest.path())?;
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        Ok(Some((
            latest.file_name().to_string_lossy().to_string(),
            hash,
        )))
    } else {
        Ok(None)
    }
}
