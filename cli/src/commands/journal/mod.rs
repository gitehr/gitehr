use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use super::{contributor, git};

pub mod commit;
pub mod list;
pub mod new_entry;
pub mod show;

#[derive(Subcommand)]
pub enum JournalCommands {
    #[command(name = "new-entry", aliases = ["new", "touch"], about = "Create a draft journal entry and open it in your editor")]
    NewEntry,
    #[command(about = "Commit a draft: prepend frontmatter, move to journal/, and git-commit")]
    Commit {
        #[arg(help = "Draft filename (relative to tmp/journal/) or absolute path")]
        file: String,
    },
    #[command(aliases = ["ls", "cat"], about = "List journal entries, or read one with --raw / --metadata")]
    List {
        #[arg(help = "Journal entry filename (required with --raw or --metadata)")]
        filename: Option<String>,
        #[arg(long, help = "Operate on drafts in tmp/journal/ instead")]
        drafts: bool,
        #[arg(long, help = "Print raw file content including frontmatter")]
        raw: bool,
        #[arg(long, help = "Print only the frontmatter")]
        metadata: bool,
    },
    #[command(about = "Show the body of a journal entry")]
    Show {
        #[arg(help = "Journal entry filename")]
        filename: String,
        #[arg(long, help = "Show a draft from tmp/journal/ instead")]
        drafts: bool,
    },
}

pub fn run(command: JournalCommands) -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
        anyhow::bail!(
            "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
        );
    }

    match command {
        JournalCommands::NewEntry => new_entry::run(),
        JournalCommands::Commit { file } => commit::run(file),
        JournalCommands::List { filename, drafts, raw, metadata } => list::run(filename, drafts, raw, metadata),
        JournalCommands::Show { filename, drafts } => show::run(filename, drafts),
    }
}

// ── Core data structures ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
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
/// skipped with a warning on stderr.
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

pub fn create_journal_entry(content: &str) -> Result<()> {
    create_journal_entry_with_documents(content, Vec::new())
}

pub fn create_journal_entry_with_documents(
    content: &str,
    documents: Vec<DocumentRef>,
) -> Result<()> {
    let entry = JournalEntry {
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
