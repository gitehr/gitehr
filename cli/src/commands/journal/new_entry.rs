use anyhow::Result;
use std::{fs, path::PathBuf};
use uuid::Uuid;

pub fn run() -> Result<()> {
    let draft_dir = PathBuf::from("tmp/journal");
    fs::create_dir_all(&draft_dir)?;

    let filename = format!("draft-{}.md", Uuid::new_v4());
    let path = draft_dir.join(&filename);
    fs::write(&path, "")?;

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = std::process::Command::new(&editor).arg(&path).status()?;
    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    println!("Draft saved: {}", path.display());
    Ok(())
}
