// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

use super::{load_mpi, save_mpi};

/// Remove an identifier link (type:value) from whichever subject holds it.
/// Does not touch the subject's record itself.
pub fn run(id_type: &str, value: &str) -> Result<()> {
    let mut mpi = load_mpi()?;

    let target = mpi
        .patients
        .iter_mut()
        .find(|p| {
            p.identifiers
                .iter()
                .any(|id| id.id_type == id_type && id.value == value)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("Identifier {id_type}:{value} is not linked to any subject")
        })?;

    target
        .identifiers
        .retain(|id| !(id.id_type == id_type && id.value == value));
    target.updated_at = chrono::Utc::now().to_rfc3339();
    mpi.updated_at = chrono::Utc::now().to_rfc3339();
    save_mpi(&mpi)?;

    println!("Unlinked {id_type}:{value}.");
    Ok(())
}
