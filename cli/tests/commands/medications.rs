// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::journal::parsed_entries;
use gitehr::commands::medications::{MedicationInput, MedicationStatus, add, list, stop};

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

fn medication_input() -> MedicationInput {
    MedicationInput {
        name: "Atorvastatin".to_string(),
        dose: Some("20mg".to_string()),
        route: Some("oral".to_string()),
        frequency: Some("once daily at night".to_string()),
        indication: Some("Hypercholesterolaemia".to_string()),
        prescriber: Some("Dr Example".to_string()),
        started: Some("2026-01-15".to_string()),
        supplement: false,
        note: None,
    }
}

#[test]
#[serial]
fn medication_add_writes_active_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let medication = add(medication_input())?;
    assert!(medication.id.starts_with("MED-"));

    let active = list(false)?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "Atorvastatin");
    assert_eq!(active[0].dose.as_deref(), Some("20mg"));
    assert_eq!(active[0].started.as_deref(), Some("2026-01-15"));
    assert_eq!(active[0].status, MedicationStatus::Active);
    assert!(!active[0].supplement);

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0]
            .content
            .contains("Added medication: Atorvastatin")
    );

    Ok(())
}

#[test]
#[serial]
fn medication_add_rejects_malformed_started_date() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = medication_input();
    input.started = Some("15-01-2026".to_string());
    let result = add(input);

    assert!(result.is_err());
    assert!(list(true)?.is_empty());

    Ok(())
}

#[test]
#[serial]
fn medication_can_be_flagged_as_supplement() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = medication_input();
    input.name = "Vitamin D".to_string();
    input.indication = None;
    input.prescriber = None;
    input.supplement = true;
    let medication = add(input)?;

    assert!(medication.supplement);

    Ok(())
}

#[test]
#[serial]
fn medication_stop_hides_from_default_list_but_keeps_history() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let medication = add(medication_input())?;
    stop(
        &medication.id,
        Some("2026-06-30"),
        Some("Statin intolerance"),
    )?;

    assert!(list(false)?.is_empty());
    let all = list(true)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, MedicationStatus::Stopped);
    assert_eq!(all[0].stopped.as_deref(), Some("2026-06-30"));
    assert_eq!(all[0].stopped_reason.as_deref(), Some("Statin intolerance"));

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn medication_stop_defaults_date_to_today_when_omitted() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let medication = add(medication_input())?;
    let stopped = stop(&medication.id, None, None)?;

    assert!(stopped.stopped.is_some());

    Ok(())
}
