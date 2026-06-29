// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::demographics::{DemographicsUpdate, load, update};
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
fn demographics_update_writes_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    update(DemographicsUpdate {
        title: Some("Mr".to_string()),
        full_name: Some("Alex Smith".to_string()),
        preferred_name: Some("Alex".to_string()),
        date_of_birth: Some("1970-01-01".to_string()),
        nhs_number: Some("1234567890".to_string()),
        note: Some("Updated patient demographics from registration form.".to_string()),
        ..DemographicsUpdate::default()
    })?;

    let demographics = load()?;
    assert_eq!(demographics.title.as_deref(), Some("Mr"));
    assert_eq!(demographics.full_name.as_deref(), Some("Alex Smith"));
    assert_eq!(demographics.date_of_birth.as_deref(), Some("1970-01-01"));
    assert_eq!(demographics.nhs_number.as_deref(), Some("1234567890"));
    assert!(
        demographics
            .identifiers
            .iter()
            .any(|id| id.id_type == "NHS" && id.value == "1234567890")
    );

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].content,
        "Updated patient demographics from registration form."
    );

    Ok(())
}

#[test]
#[serial]
fn demographics_update_rejects_invalid_date() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let result = update(DemographicsUpdate {
        date_of_birth: Some("01/01/1970".to_string()),
        ..DemographicsUpdate::default()
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("YYYY-MM-DD"));

    Ok(())
}
