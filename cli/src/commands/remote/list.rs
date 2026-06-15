use anyhow::Result;

use super::{is_gitehr_repo, load_config};

pub fn run() -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let config = load_config()?;

    if config.remotes.is_empty() {
        println!("No remotes configured.");
        println!("Use 'gitehr remote add <name> <url>' to add one.");
        return Ok(());
    }

    println!("Configured remotes:");
    for (name, entry) in &config.remotes {
        println!("  {} -> {}", name, entry.url);
    }

    Ok(())
}
