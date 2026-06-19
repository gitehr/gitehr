use anyhow::Result;
use std::{fs, path::PathBuf};

use super::is_journal_entry_file;

pub fn run(filename: Option<String>, drafts: bool, raw: bool, metadata: bool) -> Result<()> {
    if raw || metadata {
        let filename = filename
            .ok_or_else(|| anyhow::anyhow!("a filename is required with --raw or --metadata"))?;
        show_file(filename, drafts, raw)
    } else {
        list_all(drafts)
    }
}

fn list_all(drafts: bool) -> Result<()> {
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

fn show_file(filename: String, drafts: bool, raw: bool) -> Result<()> {
    let path = if drafts {
        PathBuf::from("tmp/journal").join(&filename)
    } else {
        PathBuf::from("journal").join(&filename)
    };

    if !path.exists() {
        if drafts {
            anyhow::bail!("Draft not found: {}", filename);
        } else {
            anyhow::bail!("Journal entry not found: {}", filename);
        }
    }

    let content = fs::read_to_string(&path)?;

    if raw {
        print!("{}", content);
        return Ok(());
    }

    // --metadata: extract and print only the frontmatter block
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("No frontmatter found in: {}", filename);
    }
    println!("---{}---", parts[1]);
    Ok(())
}
