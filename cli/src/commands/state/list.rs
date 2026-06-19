use anyhow::Result;

use super::{is_gitehr_repo, list_state_files};

pub fn run() -> Result<()> {
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
