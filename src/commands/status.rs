// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct RepoStatus {
    pub is_gitehr_repo: bool,
    pub gitehr_version: Option<String>,
    pub journal_entry_count: usize,
    pub state_files: Vec<String>,
    pub has_uncommitted_changes: bool,
    pub uncommitted_files: Vec<String>,
    pub is_encrypted: bool,
}

impl RepoStatus {
    pub fn gather() -> Result<Self> {
        let gitehr_dir = PathBuf::from(".gitehr");
        let is_gitehr_repo = gitehr_dir.exists();

        if !is_gitehr_repo {
            return Ok(Self {
                is_gitehr_repo: false,
                gitehr_version: None,
                journal_entry_count: 0,
                state_files: vec![],
                has_uncommitted_changes: false,
                uncommitted_files: vec![],
                is_encrypted: false,
            });
        }

        let gitehr_version = fs::read_to_string(".gitehr/GITEHR_VERSION").ok();
        let journal_entry_count = count_journal_entries()?;
        let state_files = list_state_files()?;
        let (has_uncommitted_changes, uncommitted_files) = check_git_status()?;
        let is_encrypted = gitehr_dir.join("ENCRYPTED").exists();

        Ok(Self {
            is_gitehr_repo,
            gitehr_version,
            journal_entry_count,
            state_files,
            has_uncommitted_changes,
            uncommitted_files,
            is_encrypted,
        })
    }
}

fn count_journal_entries() -> Result<usize> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        return Ok(0);
    }

    let count = fs::read_dir(&journal_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .count();

    Ok(count)
}

fn list_state_files() -> Result<Vec<String>> {
    let state_dir = PathBuf::from("state");
    if !state_dir.exists() {
        return Ok(vec![]);
    }

    let files: Vec<String> = fs::read_dir(&state_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| e.file_name() != "README.md")
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    Ok(files)
}

fn check_git_status() -> Result<(bool, Vec<String>)> {
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output();

    match git_check {
        Ok(output) if output.status.success() => {
            let status_output = Command::new("git")
                .args(["status", "--porcelain"])
                .output()?;

            let output_str = String::from_utf8_lossy(&status_output.stdout);
            let uncommitted_files: Vec<String> = output_str
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| line.to_string())
                .collect();

            let has_changes = !uncommitted_files.is_empty();
            Ok((has_changes, uncommitted_files))
        }
        _ => Ok((false, vec![])),
    }
}

pub fn run_status() -> Result<()> {
    let status = RepoStatus::gather()?;

    if !status.is_gitehr_repo {
        println!("Not a GitEHR repository (or not in the repository root).");
        println!("Run 'gitehr init' to create a new repository.");
        return Ok(());
    }

    println!("GitEHR Repository Status");
    println!("========================");
    println!();

    if let Some(version) = &status.gitehr_version {
        println!("Repository version: {}", version.trim());
    }

    println!(
        "Encryption: {}",
        if status.is_encrypted {
            "Encrypted"
        } else {
            "Not encrypted"
        }
    );
    println!();

    println!("Journal entries: {}", status.journal_entry_count);

    if status.state_files.is_empty() {
        println!("State files: None");
    } else {
        println!("State files: {}", status.state_files.len());
        for file in &status.state_files {
            println!("  - {}", file);
        }
    }
    println!();

    if status.has_uncommitted_changes {
        println!("Uncommitted changes:");
        for file in &status.uncommitted_files {
            println!("  {}", file);
        }
    } else {
        println!("Working directory clean (no uncommitted changes)");
    }

    Ok(())
}
