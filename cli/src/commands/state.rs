// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub name: String,
    pub content: String,
    pub last_modified: Option<String>,
}

fn get_state_dir() -> PathBuf {
    PathBuf::from("state")
}

fn is_gitehr_repo() -> bool {
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

pub fn run_state_list() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let files = list_state_files()?;

    if files.is_empty() {
        println!("No state files found.");
        println!("Use 'gitehr state set <filename> <content>' to create one.");
        return Ok(());
    }

    println!("State files:");
    for file in &files {
        println!("  - {}", file.name);
        if let Some(modified) = &file.last_modified {
            println!("    Last modified: {}", modified);
        }
    }

    Ok(())
}

pub fn run_state_get(filename: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let file = view_state_file(filename)?;
    println!("{}", file.content);

    Ok(())
}

pub fn run_state_set(filename: &str, content: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    update_state_file(filename, content)?;
    Ok(())
}
