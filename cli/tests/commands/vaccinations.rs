// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::journal::parsed_entries;
use gitehr::commands::vaccinations::{
    VaccinationInput, VaccinationStatus, add, entered_in_error, list,
};

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

fn vaccination_input() -> VaccinationInput {
    VaccinationInput {
        vaccine: "MMR".to_string(),
        date: "2026-06-30".to_string(),
        dose_sequence: Some(1),
        target_disease: vec![
            "measles".to_string(),
            "mumps".to_string(),
            "rubella".to_string(),
        ],
        anatomical_site: Some("left deltoid".to_string()),
        route: Some("intramuscular".to_string()),
        product: Some("Priorix".to_string()),
        manufacturer: Some("GSK".to_string()),
        batch_number: Some("ABC123".to_string()),
        performer: Some("Nurse Example".to_string()),
        fhir_json: None,
        note: None,
    }
}

#[test]
#[serial]
fn vaccination_add_writes_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let vaccination = add(vaccination_input())?;
    assert!(vaccination.id.starts_with("VAC-"));

    let current = list(false)?;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].vaccine, "MMR");
    assert_eq!(current[0].date, "2026-06-30");
    assert_eq!(current[0].dose_sequence, Some(1));
    assert_eq!(current[0].anatomical_site.as_deref(), Some("left deltoid"));
    assert_eq!(current[0].product.as_deref(), Some("Priorix"));
    assert_eq!(current[0].batch_number.as_deref(), Some("ABC123"));
    assert_eq!(current[0].status, VaccinationStatus::Completed);

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains("Recorded vaccination: MMR"));

    Ok(())
}

#[test]
#[serial]
fn vaccination_can_embed_fhir_r4_immunization_json() -> Result<()> {
    let temp_dir = setup_with_git()?;
    let fhir_path = temp_dir.path().join("mmr.fhir.json");
    fs::write(
        &fhir_path,
        r#"{
  "resourceType": "Immunization",
  "status": "completed",
  "vaccineCode": {
    "text": "MMR"
  },
  "occurrenceDateTime": "2026-06-30"
}
"#,
    )?;

    let mut input = vaccination_input();
    input.fhir_json = Some(fhir_path);
    let vaccination = add(input)?;

    assert_eq!(
        vaccination
            .fhir_r4
            .as_ref()
            .and_then(|value| value.get("resourceType"))
            .and_then(|value| value.as_str()),
        Some("Immunization")
    );

    Ok(())
}

#[test]
#[serial]
fn vaccination_entered_in_error_hides_from_default_list() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let vaccination = add(vaccination_input())?;
    entered_in_error(&vaccination.id, Some("Wrong patient"))?;

    assert!(list(false)?.is_empty());
    let all = list(true)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, VaccinationStatus::EnteredInError);
    assert_eq!(
        all[0].entered_in_error_reason.as_deref(),
        Some("Wrong patient")
    );

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);

    Ok(())
}
