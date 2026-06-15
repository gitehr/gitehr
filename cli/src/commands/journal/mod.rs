use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use super::{contributor, git};
use crate::utils::sha256_hex;

pub mod add;
pub mod cat;
pub mod show;
pub mod verify;

#[derive(Subcommand)]
pub enum JournalCommands {
    Add {
        #[arg(help = "The content to add to the journal (use --file for file input)")]
        content: Option<String>,
        #[arg(short, long, help = "Read content from a file (use - for stdin)")]
        file: Option<String>,
    },
    Show {
        #[arg(
            short = 'n',
            long,
            default_value = "10",
            help = "Maximum number of entries to display"
        )]
        limit: usize,
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Number of entries to skip from the start"
        )]
        offset: usize,
        #[arg(short, long, help = "Show newest entries first")]
        reverse: bool,
        #[arg(short, long, help = "Show all entries (ignores --limit)")]
        all: bool,
    },
    #[command(
        about = "Play back the full journal, oldest first",
        long_about = "Print the full body of every journal entry in chronological order. \
                      Use this to read the patient's record end to end."
    )]
    Cat {
        #[arg(short, long, help = "Show newest entries first")]
        reverse: bool,
    },
    Verify,
}

pub fn run(command: JournalCommands) -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
        anyhow::bail!(
            "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
        );
    }

    match command {
        JournalCommands::Add { content, file } => add::run(content, file),
        JournalCommands::Show {
            limit,
            offset,
            reverse,
            all,
        } => show::run(limit, offset, reverse, all),
        JournalCommands::Cat { reverse } => cat::run(reverse),
        JournalCommands::Verify => verify::run(),
    }
}

// ── Core data structures ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub parent_hash: Option<String>,
    pub parent_entry: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<DocumentRef>>,
}

/// A reference from a journal entry to a Document in the record.
/// The sha256 is a verifiability proof: for a file Document it hashes the
/// file itself, for a directory Document it hashes the manifest (ADR-0003).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRef {
    pub path: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
}

/// Parsed journal entry with metadata and content
pub struct ParsedEntry {
    pub filename: String,
    pub metadata: JournalEntry,
    pub content: String,
}

// ── Core helper functions (used by children and siblings) ────────────────────

/// Parse a journal file into metadata and content
pub fn parse_journal_file(path: &PathBuf) -> Result<ParsedEntry> {
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

    let metadata: JournalEntry = serde_yaml_ng::from_str(yaml_content)?;

    Ok(ParsedEntry {
        filename,
        metadata,
        content: body_content,
    })
}

pub fn is_journal_entry_file(filename: &str) -> bool {
    filename.contains('T') && filename.contains('-') && filename.ends_with(".md")
}

/// Parse every journal entry, oldest first. Entries that fail to parse are
/// skipped with a warning on stderr; callers needing strict validation should
/// use `gitehr journal verify` instead.
pub fn parsed_entries() -> Result<Vec<ParsedEntry>> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths: Vec<_> = fs::read_dir(&journal_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(is_journal_entry_file)
                .unwrap_or(false)
        })
        .collect();
    paths.sort();

    let mut entries = Vec::new();
    for path in &paths {
        match parse_journal_file(path) {
            Ok(parsed) => entries.push(parsed),
            Err(e) => eprintln!("Warning: skipping {}: {}", path.display(), e),
        }
    }
    Ok(entries)
}

pub fn create_journal_entry(content: &str, parent_hash: Option<String>) -> Result<()> {
    create_journal_entry_with_documents(content, parent_hash, Vec::new())
}

pub fn create_journal_entry_with_documents(
    content: &str,
    parent_hash: Option<String>,
    documents: Vec<DocumentRef>,
) -> Result<()> {
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
                let hash = sha256_hex(content.as_bytes());
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
        documents: if documents.is_empty() {
            None
        } else {
            Some(documents)
        },
    };

    let filename = format!(
        "journal/{}-{}.md",
        entry.timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
        Uuid::new_v4()
    );

    let yaml = serde_yaml_ng::to_string(&entry)?;
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
        let hash = sha256_hex(content.as_bytes());
        Ok(Some((
            latest.file_name().to_string_lossy().to_string(),
            hash,
        )))
    } else {
        Ok(None)
    }
}
