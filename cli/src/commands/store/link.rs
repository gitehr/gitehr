// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};

use super::{MpiIdentifier, load_mpi, save_mpi};

/// Link an identifier (type:value) to a subject. Fails if the identifier is
/// already linked to a different subject - an identifier must resolve to at
/// most one subject.
pub fn run(subject: &str, id_type: String, value: &str) -> Result<()> {
    let mut mpi = load_mpi()?;

    if let Some(holder) = mpi.patients.iter().find(|p| {
        p.identifiers
            .iter()
            .any(|id| id.id_type == id_type && id.value == value)
    }) {
        if holder.patient_id == subject || holder.repo_path == subject {
            println!("Identifier {id_type}:{value} is already linked to this subject.");
            return Ok(());
        }
        bail!(
            "Identifier {id_type}:{value} is already linked to subject '{}' - unlink it first",
            holder.patient_id
        );
    }

    let target = mpi
        .patients
        .iter_mut()
        .find(|p| p.patient_id == subject || p.repo_path == subject)
        .ok_or_else(|| {
            anyhow::anyhow!("Subject '{subject}' not found in the MPI (tried both id and name)")
        })?;

    target.identifiers.push(MpiIdentifier {
        id_type: id_type.clone(),
        value: value.to_string(),
    });
    target.updated_at = chrono::Utc::now().to_rfc3339();
    mpi.updated_at = chrono::Utc::now().to_rfc3339();
    save_mpi(&mpi)?;

    println!("Linked {id_type}:{value} to subject.");
    Ok(())
}
