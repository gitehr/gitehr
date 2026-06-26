// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::git;
use super::journal::{self, is_journal_entry_file, parse_journal_file};

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ImportMode {
    /// Import well-formed gitehr journal entries, preserved verbatim.
    Journal,
    /// Import documents of any format; each gets a journal entry linking to it.
    Documents,
}

/// Import journal entries or documents from another gitehr instance.
///
/// `source` may be a single file or a directory. Directories are walked
/// recursively; files that don't match the mode are skipped.
pub fn run(mode: ImportMode, source: &Path) -> Result<()> {
    if !PathBuf::from(".gitehr").exists() {
        bail!(
            "Not a GitEHR repository (or not in the repository root). Run 'gitehr store init' to create a new repository."
        );
    }
    if !source.exists() {
        bail!("No such file or directory: {}", source.display());
    }

    let files = collect_files(source);

    match mode {
        ImportMode::Journal => import_journal(&files),
        ImportMode::Documents => import_documents(&files),
    }
}

/// All regular files under `source`, recursively, ignoring hidden files and
/// directories (so pointing at a whole repo doesn't pull in `.git`, etc.).
/// A single file source yields just that file.
fn collect_files(source: &Path) -> Vec<PathBuf> {
    WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(DirEntry::into_path)
        .collect()
}

fn is_hidden(entry: &DirEntry) -> bool {
    // depth 0 is the source itself (e.g. ".") — never treat it as hidden.
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
}

/// Copy well-formed journal entries verbatim into `journal/`, keeping their
/// original filename (and thus timestamp/UUID). Entries already present are
/// skipped, as are files that aren't valid journal entries.
fn import_journal(files: &[PathBuf]) -> Result<()> {
    fs::create_dir_all("journal")?;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for file in files {
        let Some(filename) = file.file_name().and_then(|n| n.to_str()) else {
            skipped += 1;
            continue;
        };

        if !is_journal_entry_file(filename) {
            skipped += 1;
            continue;
        }
        if let Err(e) = parse_journal_file(file) {
            eprintln!("Skipping (not a valid journal entry) {}: {}", filename, e);
            skipped += 1;
            continue;
        }

        let dest = PathBuf::from("journal").join(filename);
        if dest.exists() {
            println!("Skipping (already present): {}", filename);
            skipped += 1;
            continue;
        }

        fs::copy(file, &dest).with_context(|| format!("Failed to copy {}", file.display()))?;
        let dest_str = dest.to_string_lossy().to_string();
        git::git_add(&dest_str)?;
        git::git_commit(&format!("Import journal entry: {}", filename))?;
        println!("Imported journal entry: {}", filename);
        imported += 1;
    }

    println!(
        "Imported {} journal {}, skipped {}.",
        imported,
        if imported == 1 { "entry" } else { "entries" },
        skipped
    );
    Ok(())
}

/// Copy each file into `documents/` (any format) and create a journal entry
/// whose body is just a markdown link to the document — no `documents:`
/// frontmatter. Files already present in `documents/` are skipped.
fn import_documents(files: &[PathBuf]) -> Result<()> {
    fs::create_dir_all("documents")?;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for file in files {
        let Some(filename) = file.file_name().and_then(|n| n.to_str()) else {
            skipped += 1;
            continue;
        };

        let dest = PathBuf::from("documents").join(filename);
        if dest.exists() {
            println!("Skipping (already present): documents/{}", filename);
            skipped += 1;
            continue;
        }

        fs::copy(file, &dest).with_context(|| format!("Failed to copy {}", file.display()))?;
        let dest_str = dest.to_string_lossy().to_string();
        git::git_add(&dest_str)?;

        // Body link only — the GUI decides whether to follow it.
        let body = format!("[{0}](/documents/{0})", filename);
        journal::create_journal_entry_with_documents(&body, Vec::new())?;
        println!("Imported document: documents/{}", filename);
        imported += 1;
    }

    println!(
        "Imported {} {}, skipped {}.",
        imported,
        if imported == 1 {
            "document"
        } else {
            "documents"
        },
        skipped
    );
    Ok(())
}
