use anyhow::Result;
use chrono::Utc;
use std::{fs, path::PathBuf};
use uuid::Uuid;

use super::JournalEntry;
use crate::commands::{contributor, git};

pub fn run(file: String) -> Result<()> {
    let file = if PathBuf::from(&file).is_absolute() {
        file
    } else {
        super::resolve_entry(&file, true)?
    };

    let draft_path = {
        let p = PathBuf::from(&file);
        if p.is_absolute() {
            p
        } else {
            PathBuf::from("tmp/journal").join(&file)
        }
    };

    if !draft_path.exists() {
        anyhow::bail!("Draft not found: {}", draft_path.display());
    }

    let content = fs::read_to_string(&draft_path)?;

    let entry = JournalEntry {
        timestamp: Utc::now(),
        author: contributor::get_current_contributor(),
        documents: None,
    };

    let dest_filename = format!(
        "journal/{}-{}.md",
        entry.timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
        Uuid::new_v4()
    );

    let yaml = serde_yaml_ng::to_string(&entry)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, content);

    fs::create_dir_all("journal")?;
    fs::write(&dest_filename, &file_content)?;
    fs::remove_file(&draft_path)?;

    git::git_add(&dest_filename)?;
    git::git_commit(&format!("Journal entry: {}", dest_filename))?;

    println!("Committed: {}", dest_filename);
    Ok(())
}
