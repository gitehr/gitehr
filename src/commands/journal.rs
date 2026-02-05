use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use super::{contributor, git};

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub parent_hash: Option<String>,
    pub parent_entry: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Parsed journal entry with metadata and content
pub struct ParsedEntry {
    pub filename: String,
    pub metadata: JournalEntry,
    pub content: String,
}

/// Parse a journal file into metadata and content
fn parse_journal_file(path: &PathBuf) -> Result<ParsedEntry> {
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let file_content = fs::read_to_string(path)?;

    // Split on YAML front matter delimiters
    let parts: Vec<&str> = file_content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid journal entry format: missing YAML front matter");
    }

    let yaml_content = parts[1].trim();
    let body_content = parts[2].trim().to_string();

    let metadata: JournalEntry = serde_yml::from_str(yaml_content)?;

    Ok(ParsedEntry {
        filename,
        metadata,
        content: body_content,
    })
}

fn is_journal_entry_file(filename: &str) -> bool {
    filename.contains('T') && filename.contains('-') && filename.ends_with(".md")
}

pub fn create_journal_entry(content: &str, parent_hash: Option<String>) -> Result<()> {
    let parent_entry = {
        let entries: Vec<_> = fs::read_dir("journal")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(is_journal_entry_file)
                    .unwrap_or(false)
            })
            .collect();
        entries
            .iter()
            .filter_map(|entry| {
                let content = fs::read_to_string(entry.path()).ok()?;
                let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
                if Some(hash) == parent_hash {
                    Some(entry.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .next()
    };

    let entry = JournalEntry {
        parent_hash,
        parent_entry,
        timestamp: Utc::now(),
        author: contributor::get_current_contributor(),
    };

    let filename = format!(
        "journal/{}-{}.md",
        entry.timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
        Uuid::new_v4()
    );

    let yaml = serde_yml::to_string(&entry)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, content);

    fs::write(&filename, file_content)?;
    println!("Created journal entry: {}", filename);

    git::git_add(&filename)?;
    let commit_message = format!("Journal entry: {}", filename);
    git::git_commit(&commit_message)?;

    Ok(())
}

pub fn get_latest_journal_entry() -> Result<Option<(String, String)>> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        return Ok(None);
    }

    let mut entries: Vec<_> = fs::read_dir(&journal_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(is_journal_entry_file)
                .unwrap_or(false)
        })
        .collect();

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

pub fn show_journal_entries(limit: usize, offset: usize, reverse: bool, all: bool) -> Result<()> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        println!("No journal entries found.");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&journal_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.contains('T') && name.contains('-') && name.ends_with(".md"))
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("No journal entries found.");
        return Ok(());
    }

    entries.sort();

    if reverse {
        entries.reverse();
    }

    let total_count = entries.len();

    let entries_to_show: Vec<_> = if all {
        entries.into_iter().skip(offset).collect()
    } else {
        entries.into_iter().skip(offset).take(limit).collect()
    };

    if entries_to_show.is_empty() {
        println!(
            "No entries to display (offset {} exceeds total {}).",
            offset, total_count
        );
        return Ok(());
    }

    for (idx, path) in entries_to_show.iter().enumerate() {
        let entry_num = offset + idx + 1;

        match parse_journal_file(path) {
            Ok(parsed) => {
                let parent_display = parsed
                    .metadata
                    .parent_entry
                    .as_deref()
                    .unwrap_or("(genesis)");

                let preview: String = parsed
                    .content
                    .chars()
                    .take(80)
                    .collect::<String>()
                    .replace('\n', " ");
                let preview_suffix = if parsed.content.len() > 80 { "..." } else { "" };

                println!("[{}] {}", entry_num, parsed.filename);
                println!(
                    "    Timestamp: {}",
                    parsed.metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!("    Parent: {}", parent_display);
                println!("    Preview: {}{}", preview, preview_suffix);
                println!();
            }
            Err(e) => {
                let filename = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                println!("[{}] {} (error: {})", entry_num, filename, e);
                println!();
            }
        }
    }

    let shown = entries_to_show.len();
    println!("Showing {} of {} entries.", shown, total_count);

    Ok(())
}
