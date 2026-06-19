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
    #[command(name = "list-entry", aliases = ["list", "ls"], about = "List journal entries")]
    List {
        #[arg(long, help = "List drafts in tmp/journal/ instead")]
        drafts: bool,
    },
    #[command(aliases = ["cat"], about = "Show a journal entry (body by default; --raw or --metadata for more)")]
    Show {
        #[arg(help = "Journal entry filename")]
        filename: String,
        #[arg(long, help = "Operate on drafts in tmp/journal/ instead")]
        drafts: bool,
        #[arg(long, help = "Print raw file content including frontmatter")]
        raw: bool,
        #[arg(long, help = "Print only the frontmatter")]
        metadata: bool,
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
        JournalCommands::List { drafts } => list::run(drafts),
        JournalCommands::Show { filename, drafts, raw, metadata } => show::run(filename, drafts, raw, metadata),
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

// ── Entry resolution (LATEST syntax) ─────────────────────────────────────────

/// Returns filenames sorted newest-first.
/// For committed entries, reads `journal/`; for drafts, `tmp/journal/`.
pub fn sorted_entries(drafts: bool) -> Result<Vec<String>> {
    let dir = if drafts {
        PathBuf::from("tmp/journal")
    } else {
        PathBuf::from("journal")
    };

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if drafts {
                if name.ends_with(".md") { Some(name) } else { None }
            } else {
                if is_journal_entry_file(&name) { Some(name) } else { None }
            }
        })
        .collect();

    entries.sort();
    entries.reverse();
    Ok(entries)
}

/// Splits an entry reference into `(anchor, offset)`.
///
/// Recognised suffixes (applied after stripping the anchor):
///   `^`, `^^`, `^^^^` … → offset = number of carets
///   `~N`                → offset = N
///
/// Examples: `"LATEST"` → `("LATEST", 0)`, `"foo.md^^^"` → `("foo.md", 3)`,
/// `"foo.md~5"` → `("foo.md", 5)`.
fn parse_entry_ref(input: &str) -> Result<(&str, usize)> {
    // ~N suffix takes priority
    if let Some(tilde) = input.rfind('~') {
        let after = &input[tilde + 1..];
        if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
            let n: usize = after.parse()?;
            return Ok((&input[..tilde], n));
        }
    }

    // trailing ^ characters
    let carets = input.chars().rev().take_while(|&c| c == '^').count();
    if carets > 0 {
        return Ok((&input[..input.len() - carets], carets));
    }

    Ok((input, 0))
}

/// Resolve a filename or LATEST expression to a concrete filename.
///
/// Anchor may be `LATEST` (most recent) or any literal filename.
/// Offset moves toward older entries: `LATEST^` = one before most recent,
/// `some-file.md~3` = three entries older than `some-file.md`.
pub fn resolve_entry(input: &str, drafts: bool) -> Result<String> {
    let (anchor, offset) = parse_entry_ref(input)?;

    // No LATEST and no offset — plain filename, return as-is.
    if anchor != "LATEST" && offset == 0 {
        return Ok(input.to_string());
    }

    let entries = sorted_entries(drafts)?;

    if entries.is_empty() {
        anyhow::bail!("no entries found");
    }

    let base_idx = if anchor == "LATEST" {
        0
    } else {
        entries
            .iter()
            .position(|e| e == anchor)
            .ok_or_else(|| anyhow::anyhow!("entry not found: {}", anchor))?
    };

    let target = base_idx + offset;
    entries.get(target).cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "'{}' is out of range: only {} entr{}",
            input,
            entries.len(),
            if entries.len() == 1 { "y" } else { "ies" }
        )
    })
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
