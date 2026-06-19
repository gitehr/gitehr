use anyhow::Result;

use super::{is_gitehr_repo, update_state_file};

pub fn run(filename: &str, content: &str) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    update_state_file(filename, content)?;
    Ok(())
}
