// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::family_history::{
    FamilyHistoryInput, FamilyHistoryStatus, add, entered_in_error, list,
};
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

fn family_history_input() -> FamilyHistoryInput {
    FamilyHistoryInput {
        relationship: "father".to_string(),
        condition: "Myocardial infarction".to_string(),
        onset_age: Some(54),
        contributed_to_death: false,
        fhir_json: None,
        note: None,
        provenance: None,
    }
}

#[test]
#[serial]
fn family_history_add_writes_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let entry = add(family_history_input())?;
    assert!(entry.id.starts_with("FH-"));

    let current = list(false)?;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].relationship, "father");
    assert_eq!(current[0].condition, "Myocardial infarction");
    assert_eq!(current[0].onset_age, Some(54));
    assert!(!current[0].contributed_to_death);
    assert_eq!(current[0].status, FamilyHistoryStatus::Completed);

    let state = fs::read_to_string("state/family-history.md")?;
    assert!(state.contains("family_history:"));

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0]
            .content
            .contains("Recorded family history: father - Myocardial infarction")
    );

    Ok(())
}

#[test]
#[serial]
fn family_history_add_accepts_the_scaffold_template() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write(
        "state/family-history.md",
        include_str!("../../../folder-structure/state/family-history.md"),
    )?;

    add(family_history_input())?;
    assert_eq!(list(false)?.len(), 1);

    Ok(())
}

#[test]
#[serial]
fn family_history_add_rejects_blank_fields_and_implausible_onset_age() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut blank_relationship = family_history_input();
    blank_relationship.relationship = "  ".to_string();
    assert!(add(blank_relationship).is_err());

    let mut blank_condition = family_history_input();
    blank_condition.condition = String::new();
    assert!(add(blank_condition).is_err());

    let mut year_not_age = family_history_input();
    year_not_age.onset_age = Some(1972);
    let error = add(year_not_age).unwrap_err().to_string();
    assert!(error.contains("--onset-age"), "{error}");

    assert!(list(true)?.is_empty());
    assert!(parsed_entries()?.is_empty());

    Ok(())
}

#[test]
#[serial]
fn family_history_can_embed_fhir_r4_family_member_history_json() -> Result<()> {
    let temp_dir = setup_with_git()?;
    let fhir_path = temp_dir.path().join("father.fhir.json");
    fs::write(
        &fhir_path,
        r#"{
  "resourceType": "FamilyMemberHistory",
  "status": "completed",
  "relationship": { "text": "father" },
  "condition": [
    { "code": { "text": "Myocardial infarction" }, "onsetAge": { "value": 54, "unit": "a" } }
  ]
}
"#,
    )?;

    let mut input = family_history_input();
    input.fhir_json = Some(fhir_path);
    let entry = add(input)?;
    assert_eq!(
        entry
            .fhir_r4
            .as_ref()
            .and_then(|value| value.get("resourceType"))
            .and_then(|value| value.as_str()),
        Some("FamilyMemberHistory")
    );

    let wrong_path = temp_dir.path().join("wrong.fhir.json");
    fs::write(&wrong_path, r#"{"resourceType": "Condition"}"#)?;
    let mut wrong = family_history_input();
    wrong.fhir_json = Some(wrong_path);
    assert!(add(wrong).is_err());

    Ok(())
}

#[test]
#[serial]
fn family_history_entered_in_error_hides_from_default_list() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let entry = add(family_history_input())?;
    entered_in_error(&entry.id, Some("Was the paternal uncle"))?;

    assert!(list(false)?.is_empty());
    let all = list(true)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, FamilyHistoryStatus::EnteredInError);
    assert_eq!(
        all[0].entered_in_error_reason.as_deref(),
        Some("Was the paternal uncle")
    );
    assert!(entered_in_error("FH-missing", None).is_err());

    assert_eq!(parsed_entries()?.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn family_history_add_records_provenance_only_when_supplied() -> Result<()> {
    use gitehr::commands::typed_state::{EvidenceLevel, Provenance, SourceType};

    let _temp_dir = setup_with_git()?;

    let plain = add(family_history_input())?;
    assert!(plain.provenance.is_none());

    let provenance = Provenance::from_parts(
        Some(SourceType::SelfReported),
        None,
        None,
        None,
        Some(EvidenceLevel::Documented),
        None,
    )?;
    let mut input = family_history_input();
    input.relationship = "mother".to_string();
    input.condition = "Breast cancer".to_string();
    input.provenance = provenance;
    let sourced = add(input)?;

    let state = fs::read_to_string("state/family-history.md")?;
    assert!(state.contains("source_type: self-reported"));
    let reloaded = list(false)?;
    let found = reloaded.iter().find(|e| e.id == sourced.id).unwrap();
    assert_eq!(found.provenance, sourced.provenance);
    let plain_found = reloaded.iter().find(|e| e.id == plain.id).unwrap();
    assert!(plain_found.provenance.is_none());
    Ok(())
}

/// The state file and its journal entry must land in the same commit, with
/// nothing left staged.
fn assert_committed_with_journal_entry(state_path: &str) -> Result<()> {
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", state_path, "journal"])
        .output()?;
    assert!(
        status.stdout.is_empty(),
        "uncommitted after mutation: {}",
        String::from_utf8_lossy(&status.stdout)
    );
    let head = std::process::Command::new("git")
        .args(["show", "--name-only", "--format=", "HEAD"])
        .output()?;
    let files = String::from_utf8_lossy(&head.stdout);
    assert!(files.lines().any(|line| line == state_path), "{files}");
    assert!(
        files.lines().any(|line| line.starts_with("journal/")),
        "{files}"
    );
    Ok(())
}

#[test]
#[serial]
fn family_history_mutations_commit_state_with_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let entry = add(family_history_input())?;
    assert_committed_with_journal_entry("state/family-history.md")?;
    entered_in_error(&entry.id, None)?;
    assert_committed_with_journal_entry("state/family-history.md")?;

    Ok(())
}
