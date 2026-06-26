// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

use super::MpiInfo;

/// List the subjects registered in the Store's MPI.
pub fn run() -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        bail!("Not a GitEHR Store root (gitehr-mpi.json not found)");
    }

    let mpi: MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;

    if mpi.patients.is_empty() {
        println!("This Store has no subjects yet. Add one with `gitehr store add [name]`.");
        return Ok(());
    }

    println!("GitEHR Store - {} subject(s):", mpi.patients.len());
    println!();
    for subject in &mpi.patients {
        println!("  {}  ({})", subject.repo_path, subject.patient_id);
        if subject.status != "active" {
            println!("    status: {}", subject.status);
        }
        for id in &subject.identifiers {
            println!("    {}: {}", id.id_type, id.value);
        }
        if let Some(merged) = &subject.merged_into {
            println!("    merged into: {merged}");
        }
    }
    Ok(())
}
