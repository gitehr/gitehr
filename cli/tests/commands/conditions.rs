// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

use gitehr::commands::conditions::{
    Category, ClinicalStatus, ConditionInput, VerificationStatus, add, list, resolve, show,
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

fn condition_input() -> ConditionInput {
    ConditionInput {
        name: "Type 2 diabetes mellitus".to_string(),
        status: ClinicalStatus::Active,
        verification: VerificationStatus::Confirmed,
        category: Category::ProblemListItem,
        onset: Some("2020-03-01".to_string()),
        code: Some("snomed:44054006".to_string()),
        body_site: None,
        laterality: None,
        severity: Some("moderate".to_string()),
        note: None,
    }
}

#[test]
#[serial]
fn condition_add_writes_active_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = condition_input();
    input.note = Some("Diagnosed at annual review".to_string());
    let condition = add(input)?;
    assert!(condition.id.starts_with("COND-"));

    let current = list(false, false)?;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].name, "Type 2 diabetes mellitus");
    assert_eq!(current[0].onset.as_deref(), Some("2020-03-01"));
    assert_eq!(current[0].clinical_status, ClinicalStatus::Active);
    assert_eq!(current[0].category, Category::ProblemListItem);
    assert_eq!(
        current[0].verification_status,
        VerificationStatus::Confirmed
    );
    assert_eq!(
        current[0].note.as_deref(),
        Some("Diagnosed at annual review")
    );

    let json = serde_json::to_value(&current[0])?;
    assert_eq!(json["code"], "snomed:44054006");
    assert_eq!(json["clinical_status"], "active");
    assert_eq!(json["verification_status"], "confirmed");
    assert_eq!(json["category"], "problem-list-item");
    assert!(json.get("recorded_at").is_some());

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains(&format!(
        "Recorded condition: Type 2 diabetes mellitus ({})",
        condition.id
    )));
    assert!(
        entries[0]
            .content
            .contains("Note: Diagnosed at annual review")
    );

    Ok(())
}

#[test]
#[serial]
fn condition_add_rejects_malformed_onset_is_stored_as_free_text() -> Result<()> {
    // onset is free text and is not strictly validated as a date.
    let _temp_dir = setup_with_git()?;

    let mut input = condition_input();
    input.onset = Some("childhood".to_string());
    let condition = add(input)?;

    assert_eq!(condition.onset.as_deref(), Some("childhood"));
    Ok(())
}

#[test]
#[serial]
fn condition_add_rejects_empty_name() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = condition_input();
    input.name = "   ".to_string();
    let result = add(input);

    assert!(result.is_err());
    assert!(list(true, false)?.is_empty());
    Ok(())
}

#[test]
#[serial]
fn condition_add_accepts_pristine_scaffold_templates() -> Result<()> {
    for template in [
        "---\nconditions: []\n---\n",
        "---\r\nconditions: []\r\n---\r\n",
    ] {
        let _temp_dir = setup_with_git()?;
        fs::write("state/conditions.md", template)?;

        add(condition_input())?;

        assert_eq!(list(false, false)?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn condition_add_refuses_non_pristine_untracked_state_when_git_hides_untracked_files() -> Result<()>
{
    let _temp_dir = setup_with_git()?;
    let original = "---\nsource_system: untracked-import\nconditions: []\n---\n";
    fs::write("state/conditions.md", original)?;
    let configured = Command::new("git")
        .args(["config", "status.showUntrackedFiles", "no"])
        .status()?;
    assert!(configured.success());
    let hidden = Command::new("git")
        .args(["status", "--porcelain", "--", "state/conditions.md"])
        .output()?;
    assert!(hidden.status.success());
    assert!(hidden.stdout.is_empty());

    let result = add(condition_input());

    assert!(result.is_err());
    assert_eq!(fs::read_to_string("state/conditions.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    Ok(())
}

#[test]
#[serial]
fn condition_resolve_hides_from_default_list_but_keeps_history() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let condition = add(condition_input())?;
    resolve(
        &condition.id,
        Some("2026-06-30"),
        Some("Achieved remission via lifestyle change"),
    )?;

    assert!(list(false, false)?.is_empty());
    let all = list(true, false)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].clinical_status, ClinicalStatus::Resolved);
    assert_eq!(all[0].abatement.as_deref(), Some("2026-06-30"));
    assert_eq!(
        all[0].abatement_reason.as_deref(),
        Some("Achieved remission via lifestyle change")
    );

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| {
        entry.content.contains(&format!(
            "Resolved condition: Type 2 diabetes mellitus ({}) on 2026-06-30",
            condition.id
        )) && entry
            .content
            .contains("Reason: Achieved remission via lifestyle change")
    }));

    Ok(())
}

#[test]
#[serial]
fn condition_resolve_defaults_date_to_today_when_omitted() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let condition = add(condition_input())?;
    let resolved = resolve(&condition.id, None, None)?;

    assert!(resolved.abatement.is_some());

    Ok(())
}

#[test]
#[serial]
fn condition_resolve_rejects_malformed_date() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;

    let result = resolve(&condition.id, Some("30-06-2026"), None);

    assert!(result.is_err());
    assert_eq!(list(false, false)?.len(), 1);
    Ok(())
}

#[test]
#[serial]
fn condition_resolve_rejects_invalid_lifecycle_changes() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;

    let before_onset = resolve(&condition.id, Some("2019-01-01"), None);
    assert!(before_onset.is_err());
    assert_eq!(list(false, false)?.len(), 1);

    resolve(&condition.id, Some("2026-06-30"), Some("Completed"))?;
    let repeated = resolve(&condition.id, Some("2026-07-01"), Some("Replacement"));
    assert!(repeated.is_err());
    assert!(
        repeated
            .unwrap_err()
            .to_string()
            .contains("already resolved")
    );
    let stored = list(true, false)?.remove(0);
    assert_eq!(stored.abatement.as_deref(), Some("2026-06-30"));
    assert_eq!(stored.abatement_reason.as_deref(), Some("Completed"));

    Ok(())
}

#[test]
#[serial]
fn condition_list_problems_filters_by_category_and_currency() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut problem = condition_input();
    problem.name = "Hypertension".to_string();
    problem.category = Category::ProblemListItem;
    add(problem)?;

    let mut encounter = condition_input();
    encounter.name = "Ankle sprain".to_string();
    encounter.category = Category::EncounterDiagnosis;
    add(encounter)?;

    let mut resolved_problem = condition_input();
    resolved_problem.name = "Old fracture".to_string();
    resolved_problem.category = Category::ProblemListItem;
    let resolved_problem = add(resolved_problem)?;
    resolve(&resolved_problem.id, Some("2026-06-30"), None)?;

    let problems = list(false, true)?;
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].name, "Hypertension");

    let all_problems = list(true, true)?;
    assert_eq!(all_problems.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn condition_show_finds_condition_regardless_of_status() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    resolve(&condition.id, Some("2026-06-30"), None)?;

    let found = show(&condition.id)?;
    assert_eq!(found.id, condition.id);
    assert_eq!(found.clinical_status, ClinicalStatus::Resolved);

    let missing = show("COND-does-not-exist");
    assert!(missing.is_err());
    assert!(missing.unwrap_err().to_string().contains("not found"));

    Ok(())
}

#[test]
#[serial]
fn condition_mutation_preserves_unknown_fields_and_markdown_body() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write(
        "state/conditions.md",
        r#"---
source_system: nhs-app
conditions:
  - id: COND-legacy
    name: Type 2 diabetes mellitus
    clinical_status: active
    verification_status: confirmed
    category: problem-list-item
    onset: "2020-03-01"
    abatement: null
    abatement_reason: null
    body_site: null
    laterality: null
    code: "snomed:44054006"
    severity: moderate
    recorded_at: "2020-03-01T09:00:00Z"
    recorded_by: dr-example
    note: null
    source_id: external-123
---

# Condition notes

Imported verbatim context.
"#,
    )?;
    Command::new("git")
        .args(["add", "state/conditions.md"])
        .output()?;
    Command::new("git")
        .args(["commit", "-m", "Seed condition state"])
        .output()?;

    resolve("COND-legacy", Some("2026-06-30"), None)?;

    let content = fs::read_to_string("state/conditions.md")?;
    assert!(content.contains("source_system: nhs-app"));
    assert!(content.contains("source_id: external-123"));
    assert!(content.contains("# Condition notes"));
    assert!(content.contains("Imported verbatim context."));
    Ok(())
}

#[test]
#[serial]
fn condition_commit_failure_restores_files_and_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    Command::new("git")
        .args(["config", "user.name", ""])
        .output()?;

    let result = add(condition_input());

    assert!(result.is_err());
    assert!(!Path::new("state/conditions.md").exists());
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    assert!(status.stdout.is_empty());
    Ok(())
}

#[test]
#[serial]
fn condition_commit_leaves_unrelated_staged_changes_untouched() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write("unrelated.txt", "keep staged")?;
    Command::new("git")
        .args(["add", "unrelated.txt"])
        .output()?;

    add(condition_input())?;

    let committed = Command::new("git")
        .args(["show", "--name-only", "--format="])
        .output()?;
    let committed = String::from_utf8_lossy(&committed.stdout);
    assert!(committed.contains("state/conditions.md"));
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
fn condition_mutation_preserves_state_file_mode_bits() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let _temp_dir = setup_with_git()?;
    fs::write("state/conditions.md", "---\nconditions: []\n---\n")?;
    fs::set_permissions("state/conditions.md", fs::Permissions::from_mode(0o640))?;

    add(condition_input())?;

    assert_eq!(
        fs::metadata("state/conditions.md")?.permissions().mode() & 0o777,
        0o640
    );
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn condition_add_refuses_symlinked_state_file() -> Result<()> {
    use std::os::unix::fs::symlink;

    let _temp_dir = setup_with_git()?;
    fs::remove_dir("state")?;
    fs::create_dir("outside")?;
    let original = "---\nconditions: []\n---\n";
    fs::write("outside/conditions.md", original)?;
    symlink("outside", "state")?;
    Command::new("git").args(["add", "state"]).output()?;
    Command::new("git")
        .args(["commit", "-m", "Track state symlink"])
        .output()?;

    let result = add(condition_input());

    assert!(result.is_err());
    assert!(fs::symlink_metadata("state")?.file_type().is_symlink());
    assert_eq!(fs::read_to_string("outside/conditions.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    Ok(())
}
