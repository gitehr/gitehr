use anyhow::Result;
use fs_extra::dir::{self, CopyOptions};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use super::journal;

pub fn initialise() -> Result<()> {
    // Check if .gitehr directory already exists
    let gitehr_dir = PathBuf::from(".gitehr");
    if gitehr_dir.exists() {
        anyhow::bail!("GitEHR repository already exists in this directory");
    }

    // Get the path to the folder structure template
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gitehr-folder-structure");

    if !template_path.exists() {
        anyhow::bail!("Could not find template directory");
    }

    // Create initial directories
    std::fs::create_dir(".gitehr")?;

    // Record the CLI version used for initialization
    std::fs::write(".gitehr/GITEHR_VERSION", env!("CARGO_PKG_VERSION"))?;

    // Placeholder for future bundled binary install
    std::fs::write(
        ".gitehr/GITEHR_BINARY_PLACEHOLDER",
        "Binary bundling not implemented yet. This file is a placeholder.",
    )?;

    // Read and copy contents from template directory
    for entry in std::fs::read_dir(&template_path)? {
        let entry = entry?;
        let target_name = entry.file_name();

        if entry.file_type()?.is_dir() {
            let dir_options = CopyOptions::new();
            dir::copy(entry.path(), ".", &dir_options)?;
        } else {
            let file_options = fs_extra::file::CopyOptions::new();
            fs_extra::file::copy(entry.path(), target_name, &file_options)?;
        }
    }

    // Create genesis entry with random seed
    let mut rng = rand::rng();
    let mut seed = [0u8; 32];
    rng.fill(&mut seed);
    let seed_hash = format!("{:x}", Sha256::digest(&seed));

    journal::create_journal_entry("Genesis entry for GitEHR repository", Some(seed_hash))?;

    println!("Initialized empty GitEHR repository");
    Ok(())
}
