use anyhow::{bail, Result};
use std::path::Path;

use super::{collect_refs, ensure_gitehr_repository};

pub fn run(path: &str) -> Result<()> {
    ensure_gitehr_repository()?;

    let path = path.trim_end_matches('/');
    let refs: Vec<(String, crate::commands::journal::DocumentRef)> = collect_refs()?
        .into_iter()
        .filter(|(_, doc)| doc.path == path)
        .collect();

    if refs.is_empty() {
        if Path::new(path).exists() {
            println!(
                "{} exists on disk but is not referenced by any journal entry.",
                path
            );
            return Ok(());
        }
        bail!("No Document found at {}", path);
    }

    let doc = &refs[0].1;
    println!("Document: {}", path);
    println!("SHA-256: {}", doc.sha256);
    if let Some(original) = &doc.original_filename {
        println!("Original filename: {}", original);
    }
    if !Path::new(path).exists() {
        println!("Status: removed from working tree (retained in Git history)");
    }
    println!("Referenced by:");
    for (entry_file, _) in &refs {
        println!("  journal/{}", entry_file);
    }
    Ok(())
}
