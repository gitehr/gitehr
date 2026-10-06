// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::allergies::{AllergySeverity, AllergyStatus, add, inactive, list};
use gitehr::commands::journal::parsed_entries;

fn setup_with_git() -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir()?;
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir(".gitehr")?;
    fs::create_dir("journal")?;
    fs::create_dir("state")?;
    std::process::Command::new("git").args(["init"]).output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .output()?;
    Ok(temp_dir)
}

#[test]
#[serial]
fn allergy_add_writes_active_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let allergy = add("Penicillin", "Rash", AllergySeverity::High, None, None)?;
    assert!(allergy.id.starts_with("ALG-"));

    let active = list(false)?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].agent, "Penicillin");
    assert_eq!(active[0].reaction, "Rash");
    assert_eq!(active[0].severity, AllergySeverity::High);
    assert_eq!(active[0].status, AllergyStatus::Active);

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains("Added allergy: Penicillin"));

    Ok(())
}

#[test]
#[serial]
fn allergy_inactive_hides_from_active_list_but_keeps_history() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let allergy = add("Penicillin", "Rash", AllergySeverity::High, None, None)?;
    inactive(&allergy.id, Some("Entered in error"))?;

    assert!(list(false)?.is_empty());
    let all = list(true)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, AllergyStatus::Inactive);
    assert_eq!(all[0].inactive_reason.as_deref(), Some("Entered in error"));

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn allergy_add_records_provenance_only_when_supplied() -> Result<()> {
    use gitehr::commands::typed_state::{EvidenceLevel, Provenance, SourceType};

    let _temp_dir = setup_with_git()?;

    let plain = add("Penicillin", "Rash", AllergySeverity::High, None, None)?;
    assert!(plain.provenance.is_none());

    let provenance = Provenance::from_parts(
        Some(SourceType::Sar),
        Some("Example GP Practice"),
        Some("ACQ-1"),
        None,
        Some(EvidenceLevel::Documented),
        None,
    )?;
    let sourced = add("Latex", "Hives", AllergySeverity::Low, None, provenance)?;
    let state = std::fs::read_to_string("state/allergies.md")?;
    assert!(state.contains("source_type: sar"));
    let reloaded = list(false)?;
    let found = reloaded.iter().find(|a| a.id == sourced.id).unwrap();
    assert_eq!(found.provenance, sourced.provenance);
    let plain_found = reloaded.iter().find(|a| a.id == plain.id).unwrap();
    assert!(plain_found.provenance.is_none());
    Ok(())
}
