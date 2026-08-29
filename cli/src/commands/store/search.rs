// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

use super::load_mpi;

/// Find subjects by identifier (exact `type:value`), or by substring match on
/// canonical id, friendly name, or identifier value.
pub fn run(query: &str) -> Result<()> {
    let mpi = load_mpi()?;

    let matches: Vec<_> = mpi
        .patients
        .iter()
        .filter(|p| {
            p.patient_id.eq_ignore_ascii_case(query)
                || p.repo_path.eq_ignore_ascii_case(query)
                || p.identifiers.iter().any(|id| {
                    format!("{}:{}", id.id_type, id.value) == query || id.value.contains(query)
                })
                || (query.len() >= 3 && p.repo_path.contains(query))
        })
        .collect();

    if matches.is_empty() {
        anyhow::bail!("No subjects match '{query}'");
    }

    println!("{} match(es) for '{}':", matches.len(), query);
    println!();
    for subject in &matches {
        println!("  {}  ({})", subject.repo_path, subject.patient_id);
        for id in &subject.identifiers {
            println!("    {}: {}", id.id_type, id.value);
        }
        if let Some(merged) = &subject.merged_into {
            println!("    merged into: {merged}");
        }
    }
    Ok(())
}
