use anyhow::Result;

use super::{is_gitehr_repo, load_config, save_config};

pub fn run(name: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let mut config = load_config()?;

    if !config.remotes.contains_key(name) {
        anyhow::bail!("Remote '{}' does not exist.", name);
    }

    config.remotes.remove(name);
    save_config(&config)?;

    println!("Removed remote '{}'", name);
    Ok(())
}
