// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::typed_state::refuse_symlinked_path;

pub mod get;
pub mod list;
pub mod set;

#[derive(Subcommand)]
pub enum StateCommands {
    /// List mutable state files
    List,
    /// Print one mutable state file
    Get {
        #[arg(help = "Name of the state file")]
        filename: String,
    },
    /// Overwrite one mutable state file
    Set {
        #[arg(help = "Name of the state file")]
        filename: String,
        #[arg(required_unless_present = "file", help = "Content to write")]
        content: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            conflicts_with = "content",
            help = "Read content from a file, or '-' for stdin"
        )]
        file: Option<String>,
        #[arg(long, help = "Commit only this state file after writing it")]
        commit: bool,
    },
}

pub fn run(command: Option<StateCommands>) -> Result<()> {
    match command {
        Some(StateCommands::List) | None => list::run(),
        Some(StateCommands::Get { filename }) => get::run(&filename),
        Some(StateCommands::Set {
            filename,
            content,
            file,
            commit,
        }) => set::run(&filename, content.as_deref(), file.as_deref(), commit),
    }
}

// ── Shared data structures ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub name: String,
    pub content: String,
    pub last_modified: Option<String>,
}

// ── Shared helper functions ───────────────────────────────────────────────────

pub(super) fn get_state_dir() -> PathBuf {
    PathBuf::from("state")
}

pub(super) fn is_gitehr_repo() -> bool {
    PathBuf::from(".gitehr").exists()
}

pub fn list_state_files() -> Result<Vec<StateFile>> {
    let state_dir = get_state_dir();
    if !state_dir.exists() {
        return Ok(vec![]);
    }

    refuse_symlinked_path(&state_dir)?;
    let mut files = Vec::new();
    collect_state_files(&state_dir, Path::new(""), &mut files)?;

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

pub fn view_state_file(filename: &str) -> Result<StateFile> {
    let file_path = state_file_path(filename)?;
    refuse_symlinked_path(&file_path)?;

    if !file_path.exists() {
        anyhow::bail!("State file '{}' not found", filename);
    }

    let content = fs::read_to_string(&file_path)?;
    let metadata = fs::metadata(&file_path).ok();
    let last_modified = metadata.and_then(|m| m.modified().ok()).map(|t| {
        chrono::DateTime::<chrono::Utc>::from(t)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    });

    Ok(StateFile {
        name: filename.to_string(),
        content,
        last_modified,
    })
}

pub fn update_state_file(filename: &str, content: &str) -> Result<()> {
    let file_path = state_file_path(filename)?;
    refuse_symlinked_path(&file_path)?;
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&file_path, content)?;

    println!("Updated state file: {}", filename);
    Ok(())
}

/// Resolve a state-relative filename under `state/`, allowing subdirectories
/// (e.g. `calculations/feverpain-latest.json`) but rejecting an absolute
/// path or any `..` component that would escape `state/`.
pub(crate) fn state_file_path(filename: &str) -> Result<PathBuf> {
    let relative = Path::new(filename);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        anyhow::bail!("Invalid state filename: {filename}");
    }
    Ok(get_state_dir().join(relative))
}

fn collect_state_files(dir: &Path, relative_dir: &Path, files: &mut Vec<StateFile>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = relative_dir.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            anyhow::bail!("Refusing to access through symlink {}", path.display());
        }
        if file_type.is_dir() {
            collect_state_files(&path, &relative_path, files)?;
            continue;
        }
        if !file_type.is_file() || relative_path == Path::new("README.md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let metadata = fs::metadata(&path)?;
        let last_modified = metadata.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        });
        files.push(StateFile {
            name: relative_path.to_string_lossy().to_string(),
            content,
            last_modified,
        });
    }
    Ok(())
}
