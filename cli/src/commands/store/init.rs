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
