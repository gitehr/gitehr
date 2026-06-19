use anyhow::Result;
use std::collections::BTreeMap;

use super::{collect_refs, ensure_gitehr_repository, files_on_disk};

pub fn run() -> Result<()> {
    ensure_gitehr_repository()?;

    let refs = collect_refs()?;
    let mut by_path: BTreeMap<String, (crate::commands::journal::DocumentRef, Vec<String>)> =
        BTreeMap::new();
    for (entry_file, doc) in refs {
        by_path
            .entry(doc.path.clone())
            .or_insert_with(|| (doc, Vec::new()))
            .1
            .push(entry_file);
    }

    if by_path.is_empty() {
        println!("No Documents referenced by journal entries.");
    } else {
        println!("Documents ({} total):", by_path.len());
        for (path, (doc, entries)) in &by_path {
            println!();
            println!("  {}", path);
            println!("    SHA-256: {}", doc.sha256);
            if let Some(original) = &doc.original_filename {
                println!("    Original filename: {}", original);
            }
            println!(
                "    Referenced by {} journal entr{}",
                entries.len(),
                if entries.len() == 1 { "y" } else { "ies" }
            );
        }
    }

    let unreferenced: Vec<String> = files_on_disk()?
        .into_iter()
        .filter(|p| !by_path.contains_key(p))
        .collect();
    if !unreferenced.is_empty() {
        println!();
        println!("Unreferenced (present on disk, not linked from any journal entry):");
        for path in unreferenced {
            println!("  {}", path);
        }
    }

    Ok(())
}
