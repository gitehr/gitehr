use anyhow::Result;
use std::{fs, path::PathBuf};

use super::is_journal_entry_file;

pub fn run(drafts: bool) -> Result<()> {
    if drafts {
        let draft_dir = PathBuf::from("tmp/journal");
        if !draft_dir.exists() {
            println!("No drafts found.");
            return Ok(());
        }

        let mut entries: Vec<String> = fs::read_dir(&draft_dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") { Some(name) } else { None }
            })
            .collect();

        if entries.is_empty() {
            println!("No drafts found.");
            return Ok(());
        }

        entries.sort();
        for name in &entries {
            println!("{}", name);
        }
        println!("\n({} drafts)", entries.len());
    } else {
        let journal_dir = PathBuf::from("journal");
        if !journal_dir.exists() {
            println!("No journal entries found.");
            return Ok(());
        }

        let mut entries: Vec<String> = fs::read_dir(&journal_dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if is_journal_entry_file(&name) { Some(name) } else { None }
            })
            .collect();

        if entries.is_empty() {
            println!("No journal entries found.");
            return Ok(());
        }

        entries.sort();
        for name in &entries {
            println!("{}", name);
        }
        println!("\n({} entries)", entries.len());
    }
    Ok(())
}
