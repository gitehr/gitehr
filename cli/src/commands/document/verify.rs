// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};
use std::path::Path;

use super::{collect_refs, ensure_gitehr_repository, hash_file, verify_directory_document};

/// Verify every Document reference in the journal (or just `filter`).
/// A reference whose target was removed from the working tree is reported but
/// is not a failure - deletion only ever touches the working tree (ADR-0002).
/// Returns true if no integrity failures were found.
pub fn run(filter: Option<&str>) -> Result<bool> {
    ensure_gitehr_repository()?;

    let mut refs = collect_refs()?;
    if let Some(filter) = filter {
        let filter = filter.trim_end_matches('/');
        refs.retain(|(_, doc)| doc.path == filter);
        if refs.is_empty() {
            bail!("No journal entry references a Document at {}", filter);
        }
    }

    // The same (path, sha256) pair may be referenced by many entries but only
    // needs checking once.
    let mut checked = std::collections::BTreeSet::new();
    let mut ok = 0usize;
    let mut missing = 0usize;
    let mut failures = Vec::new();

    for (_, doc) in &refs {
        if !checked.insert((doc.path.clone(), doc.sha256.clone())) {
            continue;
        }
        let path = Path::new(&doc.path);
        if !path.exists() {
            println!(
                "MISSING  {} (removed from working tree; retained in Git history)",
                doc.path
            );
            missing += 1;
        } else if path.is_dir() {
            match verify_directory_document(path, &doc.sha256) {
                Ok(errors) if errors.is_empty() => {
                    println!("OK       {}", doc.path);
                    ok += 1;
                }
                Ok(errors) => {
                    for e in errors {
                        println!("FAILED   {}: {}", doc.path, e);
                        failures.push(doc.path.clone());
                    }
                }
                Err(e) => {
                    println!("FAILED   {}: {}", doc.path, e);
                    failures.push(doc.path.clone());
                }
            }
        } else {
            let actual = hash_file(path)?;
            if actual == doc.sha256 {
                println!("OK       {}", doc.path);
                ok += 1;
            } else {
                println!(
                    "FAILED   {}: hash mismatch (expected {}, found {})",
                    doc.path, doc.sha256, actual
                );
                failures.push(doc.path.clone());
            }
        }
    }

    println!();
    println!(
        "Checked {} Document reference{}: {} ok, {} missing from working tree, {} failed",
        checked.len(),
        if checked.len() == 1 { "" } else { "s" },
        ok,
        missing,
        failures.len()
    );

    Ok(failures.is_empty())
}
