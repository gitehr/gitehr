use crate::config;
use crate::error::Result;
use crate::fs_layout::FsLayout;
use crate::repo;

pub fn run() -> Result<()> {
    let repo = repo::ensure_repo()?;
    let layout = FsLayout::new();
    layout.ensure_structure()?;
    config::ensure_config(&layout.config_path())?;

    println!(
        "Initialized GitEHR repository at '{}' (Git dir: '{}')",
        layout.root().display(),
        repo.path().display()
    );

    Ok(())
}
