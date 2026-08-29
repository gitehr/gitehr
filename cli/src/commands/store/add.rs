// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::Path;

use super::{MpiIdentifier, MpiPatient, load_mpi, save_mpi};
use crate::commands::scaffold;

/// Create a new subject repo and register it in the Store's MPI.
pub fn run(name: Option<&str>, identifiers: Vec<(String, String)>) -> Result<()> {
    let mut mpi = load_mpi()?;

    let (dir, id) = scaffold::create_subject_repo(Path::new("."), name)?;

    let now = chrono::Utc::now().to_rfc3339();
    mpi.patients.push(MpiPatient {
        patient_id: id.clone(),
        repo_path: dir.clone(),
        status: "active".to_string(),
        merged_into: None,
        updated_at: now.clone(),
        identifiers: identifiers
            .into_iter()
            .map(|(id_type, value)| MpiIdentifier { id_type, value })
            .collect(),
    });
    mpi.updated_at = now;
    save_mpi(&mpi)?;

    println!("Added subject '{dir}' ({id}).");
    Ok(())
}
