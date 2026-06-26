// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Remove a patient repository from the store (does not delete files)
pub fn run(patient_id: String) -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("Not a GitEHR store root (gitehr-mpi.json not found)");
    }

    // Read existing MPI
    let mpi_content = fs::read_to_string("gitehr-mpi.json")?;
    let mut mpi: super::MpiInfo = serde_json::from_str(&mpi_content)?;

    // Find and remove patient
    let initial_count = mpi.patients.len();
    mpi.patients.retain(|p| p.patient_id != patient_id);

    if mpi.patients.len() == initial_count {
        anyhow::bail!("Patient {} not found in MPI", patient_id);
    }

    mpi.updated_at = chrono::Utc::now().to_rfc3339();

    // Write updated MPI
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    println!("✓ Removed patient {} from MPI", patient_id);
    println!("  Note: Patient repository files were not deleted");

    Ok(())
}
