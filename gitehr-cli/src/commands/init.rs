use anyhow::Result;
use fs_extra::dir::{self, CopyOptions};
use rand::RngExt;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use super::{git, journal};

fn get_current_exe_path() -> Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))
}

fn copy_binary_to_repo() -> Result<()> {
    let source = get_current_exe_path()?;
    let dest = PathBuf::from(".gitehr/gitehr");

    std::fs::copy(&source, &dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }

    Ok(())
}

pub fn initialise() -> Result<()> {
    let gitehr_dir = PathBuf::from(".gitehr");
    if gitehr_dir.exists() {
        anyhow::bail!("GitEHR repository already exists in this directory");
    }

    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gitehr-folder-structure");

    if !template_path.exists() {
        anyhow::bail!("Could not find template directory");
    }

    std::fs::create_dir(".gitehr")?;

    git::git_init()?;

    std::fs::write(".gitehr/GITEHR_VERSION", env!("CARGO_PKG_VERSION"))?;

    copy_binary_to_repo()?;

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

    let mut rng = rand::rng();
    let mut seed = [0u8; 32];
    rng.fill(&mut seed);
    let seed_hash = format!("{:x}", Sha256::digest(&seed));

    journal::create_journal_entry("Genesis entry for GitEHR repository", Some(seed_hash))?;

    println!("Initialized empty GitEHR repository");
    Ok(())
}
