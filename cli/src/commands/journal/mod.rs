// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use super::{contributor, git};

pub mod add;
pub mod draft;
pub mod list;
pub mod show;

#[derive(Subcommand)]
pub enum JournalCommands {
    #[command(
        about = "Add a journal entry (inline text, --file <path>, --file - for stdin, or your editor)"
    )]
    Add {
        #[arg(help = "Entry text. Omit (on a terminal) to open your $EDITOR, or use --file.")]
        text: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            conflicts_with = "text",
            help = "Read the entry from a file, or '-' for stdin"
        )]
        file: Option<String>,
    },
    #[command(name = "list-entry", aliases = ["list", "ls"], about = "List journal entries")]
    List,
    #[command(aliases = ["cat"], about = "Show a journal entry (body by default; --raw or --metadata for more)")]
    Show {
        #[arg(help = "Journal entry filename (or LATEST, LATEST^, LATEST~N)")]
        filename: String,
        #[arg(long, help = "Print raw file content including frontmatter")]
        raw: bool,
        #[arg(long, help = "Print only the frontmatter")]
        metadata: bool,
    },
    #[command(aliases = ["draft", "review"], about = "Review MCP-authored drafts (ADR-0007): list, approve, or reject")]
    Drafts {
        #[arg(
            long,
            conflicts_with = "reject",
            help = "Approve a draft by filename: strips the draft marker, stages, and commits it"
        )]
        approve: Option<String>,
        #[arg(
            long,
            conflicts_with = "approve",
            help = "Reject a draft by filename: deletes the uncommitted file"
        )]
        reject: Option<String>,
    },
}

pub fn run(command: JournalCommands) -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
        anyhow::bail!(
            "Not a GitEHR repository (or not in the repository root). Run 'gitehr store init' to create a new repository."
        );
    }

    match command {
        JournalCommands::Add { text, file } => add::run(text, file),
        JournalCommands::List => list::run(),
        JournalCommands::Show {
            filename,
            raw,
            metadata,
        } => show::run(filename, raw, metadata),
        JournalCommands::Drafts { approve, reject } => {
            draft::run(Path::new("."), approve.as_deref(), reject.as_deref())
        }
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
    /// True while an entry is a machine-authored draft (ADR-0007): written to
    /// disk but not committed, pending human approval. Never present on a
    /// committed entry - approval strips it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mcp_draft: bool,
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

/// Committed journal entry filenames, sorted newest-first.
pub fn sorted_entries() -> Result<Vec<String>> {
    let dir = PathBuf::from("journal");
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if is_journal_entry_file(&name) {
                Some(name)
            } else {
                None
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
pub fn resolve_entry(input: &str) -> Result<String> {
    let (anchor, offset) = parse_entry_ref(input)?;

    // No LATEST and no offset — plain filename, return as-is.
    if anchor != "LATEST" && offset == 0 {
        return Ok(input.to_string());
    }

    let entries = sorted_entries()?;

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
    let filename = create_journal_entry_at(
        Path::new("."),
        content,
        documents,
        contributor::get_current_contributor(),
    )?;
    println!("Created journal entry: {}", filename);
    Ok(())
}

/// Create a journal entry rooted at `repo_path`, without depending on the
/// process's current directory (the MCP server may be given a `--repo-path`
/// that differs from cwd). Writes the entry, stages it, and commits it, then
/// returns the entry's repo-relative filename (e.g. `journal/<name>.md`).
///
/// Unlike [`create_journal_entry_with_documents`], this does not print to
/// stdout: an MCP server speaking JSON-RPC over stdio would have any stray
/// stdout output corrupt the protocol stream.
pub fn create_journal_entry_at(
    repo_path: &Path,
    content: &str,
    documents: Vec<DocumentRef>,
    author: Option<String>,
) -> Result<String> {
    let relative_filename = write_journal_entry_at(repo_path, content, documents, author, false)?;

    git::git_add_in(repo_path, &relative_filename)?;
    let commit_message = format!("Journal entry: {relative_filename}");
    git::git_commit_in(repo_path, &commit_message)?;

    Ok(relative_filename)
}

pub(crate) fn write_journal_entry(content: &str) -> Result<String> {
    write_journal_entry_at(
        Path::new("."),
        content,
        Vec::new(),
        contributor::get_current_contributor(),
        false,
    )
}

fn write_journal_entry_at(
    repo_path: &Path,
    content: &str,
    documents: Vec<DocumentRef>,
    author: Option<String>,
    mcp_draft: bool,
) -> Result<String> {
    let entry = JournalEntry {
        timestamp: Utc::now(),
        author,
        documents: if documents.is_empty() {
            None
        } else {
            Some(documents)
        },
        mcp_draft,
    };

    let relative_filename = format!(
        "journal/{}-{}.md",
        entry.timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
        Uuid::new_v4()
    );

    let yaml = serde_yaml_ng::to_string(&entry)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, content);

    fs::write(repo_path.join(&relative_filename), file_content)?;

    Ok(relative_filename)
}

// ── MCP drafts (ADR-0007) ─────────────────────────────────────────────────────

/// Write an MCP-authored entry as an **uncommitted draft** (ADR-0007): the
/// file lands on disk with `mcp_draft: true` front matter, is not staged or
/// committed, and is pending human approval through
/// [`super::draft::run`]. Returns the draft's repo-relative filename.
///
/// The record's custody layer (git) stays free of machine-authored content
/// until a human approves the draft.
pub fn create_mcp_draft_entry(
    repo_path: &Path,
    content: &str,
    author: Option<String>,
) -> Result<String> {
    write_journal_entry_at(repo_path, content, Vec::new(), author, true)
}

/// Every unapproved MCP draft in `repo_path`, oldest first.
pub fn mcp_drafts(repo_path: &Path) -> Result<Vec<(String, ParsedEntry)>> {
    let journal_dir = repo_path.join("journal");
    if !journal_dir.exists() {
        return Ok(Vec::new());
    }

    let mut drafts = Vec::new();
    for entry in fs::read_dir(&journal_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_journal_entry_file(name) {
            continue;
        }
        let Ok(parsed) = parse_journal_file(&path) else {
            continue;
        };
        if parsed.metadata.mcp_draft {
            drafts.push((name.to_string(), parsed));
        }
    }
    drafts.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(drafts)
}

/// Accept a draft filename either bare (`<name>.md`, as `journal drafts`
/// lists it) or repo-relative (`journal/<name>.md`, as the MCP tool reports it).
fn draft_filename(filename: &str) -> String {
    filename
        .strip_prefix("journal/")
        .unwrap_or(filename)
        .to_string()
}

/// Approve a draft: strip `mcp_draft: true`, stage, and commit. The human
/// approver's identity is the git committer. Any pending audit drafts (R33)
/// are committed in the same commit, so the audit trail for an approved
/// machine action is preserved alongside the content it describes.
pub fn approve_mcp_draft(repo_path: &Path, filename: &str) -> Result<()> {
    let filename = draft_filename(filename);
    let path = repo_path.join("journal").join(&filename);
    if !path.is_file() {
        anyhow::bail!("Draft not found: journal/{filename}");
    }

    let parsed = parse_journal_file(&path)?;
    let entry = JournalEntry {
        timestamp: parsed.metadata.timestamp,
        author: parsed.metadata.author,
        documents: parsed.metadata.documents,
        mcp_draft: false,
    };

    let yaml = serde_yaml_ng::to_string(&entry)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, parsed.content);
    fs::write(&path, file_content)?;

    git::git_add_in(repo_path, &format!("journal/{filename}"))?;

    // Commit pending audit drafts in the same commit so the audit trail
    // travels with the content it describes.
    for entry in fs::read_dir(repo_path.join("journal"))? {
        let entry = entry?;
        let p = entry.path();
        let Ok(content) = fs::read_to_string(&p) else {
            continue;
        };
        if content.contains("mcp_audit:")
            && content.contains("mcp_draft: true")
            && let Some(n) = p.file_name().and_then(|n| n.to_str())
        {
            // The audit draft's approval state lives in its front matter:
            // clear the marker as it is committed so it no longer lists
            // as pending.
            let cleared = content.replace("mcp_draft: true", "mcp_draft: false");
            fs::write(&p, cleared)?;
            git::git_add_in(repo_path, &format!("journal/{n}"))?;
        }
    }

    git::git_commit_in(
        repo_path,
        &format!("Journal entry (approved MCP draft): journal/{filename}"),
    )?;
    Ok(())
}

/// Reject a draft by deleting the file. The record only grows (ADR-0002): an
/// unapproved draft never entered the record, so deleting it loses nothing.
pub fn reject_mcp_draft(repo_path: &Path, filename: &str) -> Result<()> {
    let filename = draft_filename(filename);
    let path = repo_path.join("journal").join(&filename);
    if !path.is_file() {
        anyhow::bail!("Draft not found: journal/{filename}");
    }
    let parsed = parse_journal_file(&path)?;
    if !parsed.metadata.mcp_draft {
        anyhow::bail!("journal/{filename} is not an unapproved MCP draft; refusing to delete it");
    }
    fs::remove_file(&path)?;
    Ok(())
}
