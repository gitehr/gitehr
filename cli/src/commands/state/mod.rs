// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub mod get;
pub mod list;
pub mod set;

#[derive(Subcommand)]
pub enum StateCommands {
    List,
    Get {
        #[arg(help = "Name of the state file")]
        filename: String,
    },
    Set {
        #[arg(help = "Name of the state file")]
        filename: String,
        #[arg(help = "Content to write")]
        content: String,
    },
}

pub fn run(command: Option<StateCommands>) -> Result<()> {
    match command {
        Some(StateCommands::List) | None => list::run(),
        Some(StateCommands::Get { filename }) => get::run(&filename),
        Some(StateCommands::Set { filename, content }) => set::run(&filename, &content),
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

    let mut files = Vec::new();
    for entry in fs::read_dir(&state_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && entry.file_name() != "README.md" {
            let name = entry.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            let metadata = fs::metadata(&path).ok();
            let last_modified = metadata.and_then(|m| m.modified().ok()).map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string()
            });

            files.push(StateFile {
                name,
                content,
                last_modified,
            });
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

pub fn view_state_file(filename: &str) -> Result<StateFile> {
    let state_dir = get_state_dir();
    let file_path = state_dir.join(filename);

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
    let state_dir = get_state_dir();
    if !state_dir.exists() {
        fs::create_dir_all(&state_dir)?;
    }

    let file_path = state_dir.join(filename);
    fs::write(&file_path, content)?;

    println!("Updated state file: {}", filename);
    Ok(())
}
