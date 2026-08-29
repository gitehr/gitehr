// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};

use super::{find_subject, load_mpi, save_mpi};

/// Merge one subject into another: the source is marked `merged` with
/// `merged_into` set, and its identifiers move to the target unless that
/// would create a conflict. Repository files are not touched.
pub fn run(from: &str, into: &str) -> Result<()> {
    let mut mpi = load_mpi()?;

    if mpi.patients.iter().any(|p| {
        (p.patient_id == from || p.repo_path == from)
            && (p.patient_id == into || p.repo_path == into)
    }) {
        bail!("Cannot merge a subject into itself");
    }

    let source = find_subject(&mpi, from).ok_or_else(|| {
        anyhow::anyhow!("Subject '{from}' not found in the MPI (tried both id and name)")
    })?;
    let source_id = source.patient_id.clone();
    let source_identifiers = source.identifiers.clone();
    if source.status == "merged" {
        bail!("Subject '{from}' is already merged");
    }
    if let Some(existing) = &source.merged_into {
        bail!("Subject '{from}' was already merged into '{existing}'");
    }

    let target = find_subject(&mpi, into).ok_or_else(|| {
        anyhow::anyhow!("Subject '{into}' not found in the MPI (tried both id and name)")
    })?;
    let target_id = target.patient_id.clone();

    // Refuse rather than silently dropping identifiers that would clash.
    for id in &source_identifiers {
        if target
            .identifiers
            .iter()
            .any(|t| t.id_type == id.id_type && t.value == id.value)
        {
            bail!(
                "Identifier {}:{} already exists on the target subject; resolve the conflict first",
                id.id_type,
                id.value
            );
        }
    }

    let now = chrono::Utc::now().to_rfc3339();

    let target = mpi
        .patients
        .iter_mut()
        .find(|p| p.patient_id == target_id)
        .expect("target subject found above");
    target.identifiers.extend(source_identifiers);
    target.updated_at = now.clone();

    let source = mpi
        .patients
        .iter_mut()
        .find(|p| p.patient_id == source_id)
        .expect("source subject found above");
    source.status = "merged".to_string();
    source.merged_into = Some(target_id.clone());
    source.identifiers.clear();
    source.updated_at = now.clone();

    mpi.updated_at = now;
    save_mpi(&mpi)?;

    println!("Merged subject '{from}' into '{into}' ({}).", target_id);
    println!("  Identifiers moved; source marked as merged.");
    println!("  Repository files were not deleted (the record only grows).");
    Ok(())
}
