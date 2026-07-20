// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

use super::{MpiInfo, MpiPatient};
use crate::commands::scaffold;

/// Bootstrap a new GitEHR Store in the current directory: the MPI index and the
/// first subject's repo (see spec/adr/0005).
pub fn run(name: Option<&str>) -> Result<()> {
    if Path::new("gitehr-mpi.json").exists() {
        bail!("This directory is already a GitEHR Store (gitehr-mpi.json exists)");
    }
    if Path::new(".gitehr").exists() {
        bail!(
            "This directory is a GitEHR repository, not a Store root. Create a Store in an empty directory."
        );
    }
    if fs::read_dir(".")?.next().transpose()?.is_some() {
        bail!("A GitEHR Store must be created in an empty directory");
    }

    let prompted = if name.is_none() {
        prompt_first_subject_name()?
    } else {
        None
    };
    let name = name.or(prompted.as_deref());

    let (dir, id) = scaffold::create_subject_repo(Path::new("."), name)?;

    let now = chrono::Utc::now().to_rfc3339();
    let mpi = MpiInfo {
        version: 1,
        updated_at: now.clone(),
        patients: vec![MpiPatient {
            patient_id: id.clone(),
            repo_path: dir.clone(),
            status: "active".to_string(),
            merged_into: None,
            updated_at: now,
            identifiers: Vec::new(),
        }],
    };
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    println!("Initialised GitEHR Store with first subject '{dir}' ({id}).");
    Ok(())
}

/// On a terminal, ask for the first subject's name; blank or non-interactive
/// yields `None`, so the subject gets an auto-generated id.
fn prompt_first_subject_name() -> Result<Option<String>> {
    use std::io::{self, IsTerminal, Write};
    if !io::stdin().is_terminal() {
        return Ok(None);
    }
    print!("Name for the first subject (a person or pet; blank for an auto-generated id): ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    Ok((!trimmed.is_empty()).then(|| trimmed.to_string()))
}
