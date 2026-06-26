// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

use super::MpiInfo;

/// Remove a subject from the MPI by canonical id or friendly name. Does not
/// delete the subject's repository files - the record only grows (ADR-0002).
pub fn run(subject: &str) -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        bail!("Not a GitEHR Store root (gitehr-mpi.json not found)");
    }

    let mut mpi: MpiInfo = serde_json::from_str(&fs::read_to_string("gitehr-mpi.json")?)?;

    let before = mpi.patients.len();
    mpi.patients
        .retain(|p| p.patient_id != subject && p.repo_path != subject);
    if mpi.patients.len() == before {
        bail!("Subject '{subject}' not found in the MPI (tried both id and name)");
    }

    mpi.updated_at = chrono::Utc::now().to_rfc3339();
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    println!("✓ Removed subject '{subject}' from the MPI.");
    println!("  Note: the subject's repository files were not deleted.");
    Ok(())
}
