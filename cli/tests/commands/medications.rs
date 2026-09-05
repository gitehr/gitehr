// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

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

    let mut input = medication_input();
    input.note = Some("Medication reconciled with patient".to_string());
    let medication = add(input)?;
    assert!(medication.id.starts_with("MED-"));

    let active = list(false)?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "Atorvastatin");
    assert_eq!(active[0].dose.as_deref(), Some("20mg"));
    assert_eq!(active[0].started.as_deref(), Some("2026-01-15"));
    assert_eq!(active[0].status, MedicationStatus::Active);
    assert!(!active[0].supplement);
    assert_eq!(
        active[0].note.as_deref(),
        Some("Medication reconciled with patient")
    );

    let json = serde_json::to_value(&active[0])?;
    assert_eq!(json["prescriber"], "Dr Example");
    assert_eq!(json["status"], "active");
    assert_eq!(json["supplement"], false);
    assert!(json.get("recorded_at").is_some());

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains(&format!(
        "Added medication: Atorvastatin ({})",
        medication.id
    )));
    assert!(
        entries[0]
            .content
            .contains("Note: Medication reconciled with patient")
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
fn medication_add_accepts_pristine_scaffold_templates() -> Result<()> {
    for template in [
        "---\nmedications: []\n---\n",
        "---\r\nmedications: []\r\n---\r\n",
    ] {
        let _temp_dir = setup_with_git()?;
        fs::write("state/medications.md", template)?;

        add(medication_input())?;

        assert_eq!(list(false)?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn medication_add_refuses_non_pristine_untracked_state_when_git_hides_untracked_files() -> Result<()>
{
    let _temp_dir = setup_with_git()?;
    let original = "---\nsource_system: untracked-import\nmedications: []\n---\n";
    fs::write("state/medications.md", original)?;
    let configured = Command::new("git")
        .args(["config", "status.showUntrackedFiles", "no"])
        .status()?;
    assert!(configured.success());
    let hidden = Command::new("git")
        .args(["status", "--porcelain", "--", "state/medications.md"])
        .output()?;
    assert!(hidden.status.success());
    assert!(hidden.stdout.is_empty());

    let result = add(medication_input());

    assert!(result.is_err());
    assert_eq!(fs::read_to_string("state/medications.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
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
    assert!(entries.iter().any(|entry| {
        entry.content.contains(&format!(
            "Stopped medication: Atorvastatin ({}) on 2026-06-30",
            medication.id
        )) && entry.content.contains("Reason: Statin intolerance")
    }));

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

#[test]
#[serial]
fn medication_stop_rejects_invalid_lifecycle_changes() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let medication = add(medication_input())?;

    let before_start = stop(&medication.id, Some("2025-12-31"), None);
    assert!(before_start.is_err());
    assert_eq!(list(false)?.len(), 1);

    stop(&medication.id, Some("2026-06-30"), Some("Completed"))?;
    let repeated = stop(&medication.id, Some("2026-07-01"), Some("Replacement"));
    assert!(repeated.is_err());
    let stored = list(true)?.remove(0);
    assert_eq!(stored.stopped.as_deref(), Some("2026-06-30"));
    assert_eq!(stored.stopped_reason.as_deref(), Some("Completed"));

    Ok(())
}

#[test]
#[serial]
fn medication_mutation_preserves_unknown_fields_and_markdown_body() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write(
        "state/medications.md",
        r#"---
source_system: nhs-app
medications:
  - id: MED-legacy
    name: Atorvastatin
    dose: 20mg
    route: oral
    frequency: once daily
    indication: Hypercholesterolaemia
    prescriber: Dr Example
    supplement: false
    status: active
    started: "2026-01-15"
    stopped: null
    stopped_reason: null
    recorded_at: "2026-01-15T09:00:00Z"
    recorded_by: dr-example
    note: null
    source_id: external-123
---

# Medication notes

Imported verbatim context.
"#,
    )?;
    Command::new("git")
        .args(["add", "state/medications.md"])
        .output()?;
    Command::new("git")
        .args(["commit", "-m", "Seed medication state"])
        .output()?;

    stop("MED-legacy", Some("2026-06-30"), None)?;

    let content = fs::read_to_string("state/medications.md")?;
    assert!(content.contains("source_system: nhs-app"));
    assert!(content.contains("source_id: external-123"));
    assert!(content.contains("# Medication notes"));
    assert!(content.contains("Imported verbatim context."));
    Ok(())
}

#[test]
#[serial]
fn medication_commit_failure_restores_files_and_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    Command::new("git")
        .args(["config", "user.name", ""])
        .output()?;

    let result = add(medication_input());

    assert!(result.is_err());
    assert!(!Path::new("state/medications.md").exists());
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    assert!(status.stdout.is_empty());
    Ok(())
}

#[test]
#[serial]
fn medication_commit_leaves_unrelated_staged_changes_untouched() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write("unrelated.txt", "keep staged")?;
    Command::new("git")
        .args(["add", "unrelated.txt"])
        .output()?;

    add(medication_input())?;

    let committed = Command::new("git")
        .args(["show", "--name-only", "--format="])
        .output()?;
    let committed = String::from_utf8_lossy(&committed.stdout);
    assert!(committed.contains("state/medications.md"));
    assert!(committed.contains("journal/"));
    assert!(!committed.contains("unrelated.txt"));
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    assert!(String::from_utf8_lossy(&status.stdout).contains("A  unrelated.txt"));
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn medication_mutation_preserves_state_file_mode_bits() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let _temp_dir = setup_with_git()?;
    fs::write("state/medications.md", "---\nmedications: []\n---\n")?;
    fs::set_permissions("state/medications.md", fs::Permissions::from_mode(0o640))?;

    add(medication_input())?;

    assert_eq!(
        fs::metadata("state/medications.md")?.permissions().mode() & 0o777,
        0o640
    );
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn medication_add_refuses_symlinked_state_file() -> Result<()> {
    use std::os::unix::fs::symlink;

    let _temp_dir = setup_with_git()?;
    fs::remove_dir("state")?;
    fs::create_dir("outside")?;
    let original = "---\nmedications: []\n---\n";
    fs::write("outside/medications.md", original)?;
    symlink("outside", "state")?;
    Command::new("git").args(["add", "state"]).output()?;
    Command::new("git")
        .args(["commit", "-m", "Track state symlink"])
        .output()?;

    let result = add(medication_input());

    assert!(result.is_err());
    assert!(fs::symlink_metadata("state")?.file_type().is_symlink());
    assert_eq!(fs::read_to_string("outside/medications.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    Ok(())
}
