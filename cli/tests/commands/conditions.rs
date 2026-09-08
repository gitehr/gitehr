// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

use gitehr::commands::conditions::{
    Category, ClinicalStatus, ConditionInput, ConditionsState, Laterality, VerificationStatus, add,
    list, load, resolve, show,
};
use gitehr::commands::journal::parsed_entries;

fn setup_with_git() -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir()?;
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir(".gitehr")?;
    fs::create_dir("journal")?;
    fs::create_dir("state")?;
    git(&["init"])?;
    git(&["config", "user.name", "Test User"])?;
    git(&["config", "user.email", "test@example.com"])?;
    git(&["config", "commit.gpgsign", "false"])?;
    Ok(temp_dir)
}

fn git(args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").args(args).output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
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
fn condition_add_preserves_free_text_onset() -> Result<()> {
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
        Some("Diagnosis recorded as resolved after review"),
    )?;

    assert!(list(false, false)?.is_empty());
    let all = list(true, false)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].clinical_status, ClinicalStatus::Resolved);
    assert_eq!(all[0].abatement.as_deref(), Some("2026-06-30"));
    assert_eq!(
        all[0].abatement_reason.as_deref(),
        Some("Diagnosis recorded as resolved after review")
    );

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| {
        entry.content.contains(&format!(
            "Resolved condition: Type 2 diabetes mellitus ({}) on 2026-06-30",
            condition.id
        )) && entry
            .content
            .contains("Reason: Diagnosis recorded as resolved after review")
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

#[test]
#[serial]
fn condition_list_filters_every_status_verification_and_category_combination() -> Result<()> {
    // spec/problem-condition-list.md: a problem is a possible current concern,
    // not a refuted assertion or a record entered in error.
    let _temp_dir = setup_with_git()?;
    let original = add(condition_input())?;
    let mut state = ConditionsState::default();
    let mut current_ids = Vec::new();
    let mut problem_ids = Vec::new();
    let mut all_problem_ids = Vec::new();
    for (status, current) in [
        (ClinicalStatus::Active, true),
        (ClinicalStatus::Recurrence, true),
        (ClinicalStatus::Relapse, true),
        (ClinicalStatus::Remission, true),
        (ClinicalStatus::Inactive, false),
        (ClinicalStatus::Resolved, false),
    ] {
        for (verification, plausible) in [
            (VerificationStatus::Unconfirmed, true),
            (VerificationStatus::Provisional, true),
            (VerificationStatus::Differential, true),
            (VerificationStatus::Confirmed, true),
            (VerificationStatus::Refuted, false),
            (VerificationStatus::EnteredInError, false),
        ] {
            for category in [Category::ProblemListItem, Category::EncounterDiagnosis] {
                let mut condition = original.clone();
                condition.id = format!("COND-{status}-{verification}-{category}");
                condition.clinical_status = status;
                condition.verification_status = verification;
                condition.category = category;
                if category == Category::ProblemListItem {
                    all_problem_ids.push(condition.id.clone());
                }
                if current && plausible {
                    current_ids.push(condition.id.clone());
                    if category == Category::ProblemListItem {
                        problem_ids.push(condition.id.clone());
                    }
                }
                state.conditions.push(condition);
            }
        }
    }
    fs::write("state/conditions.md", serde_yaml_ng::to_string(&state)?)?;
    for (all, problems, expected) in [
        (false, false, current_ids),
        (false, true, problem_ids),
        (true, true, all_problem_ids),
        (
            true,
            false,
            state.conditions.iter().map(|c| c.id.clone()).collect(),
        ),
    ] {
        let actual: Vec<_> = list(all, problems)?.into_iter().map(|c| c.id).collect();
        assert_eq!(actual, expected, "all={all}, problems={problems}");
    }
    Ok(())
}

#[test]
#[serial]
fn condition_journal_preserves_the_complete_original_assertion() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(ConditionInput {
        name: "Knee pain".to_string(),
        code: None,
        verification: VerificationStatus::Differential,
        onset: Some("childhood".to_string()),
        body_site: Some("knee".to_string()),
        laterality: Some(Laterality::Left),
        note: Some("Diagnosis under investigation".to_string()),
        ..condition_input()
    })?;
    let entries = parsed_entries()?;
    let yaml = entries[0].content.split("```yaml\n").nth(1).unwrap();
    let yaml = yaml.strip_suffix("```").unwrap();
    let recorded: gitehr::commands::conditions::Condition = serde_yaml_ng::from_str(yaml)?;
    assert_eq!(
        serde_json::to_value(recorded)?,
        serde_json::to_value(condition)?
    );
    Ok(())
}

#[test]
#[serial]
fn condition_resolve_rejects_noncanonical_and_impossible_dates_without_mutation() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    let before = fs::read("state/conditions.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    for date in [
        "2026-6-3",
        "2026-06-3",
        "2026-6-03",
        "2026-02-29",
        "2026-13-01",
        " 2026-06-03",
        "+2026-06-03",
        "2026-06-03 ",
        "2026-06-03T00:00:00Z",
    ] {
        let error = resolve(&condition.id, Some(date), None).unwrap_err();
        assert!(error.to_string().contains("YYYY-MM-DD"), "{date}: {error}");
        assert_eq!(fs::read("state/conditions.md")?, before);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert!(git(&["status", "--porcelain"])?.is_empty());
        assert_eq!(parsed_entries()?.len(), 1);
    }
    resolve(&condition.id, Some("2024-02-29"), None)?;
    Ok(())
}

#[test]
#[serial]
fn condition_resolve_refuses_refuted_and_erroneous_assertions() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    for verification in [
        VerificationStatus::Refuted,
        VerificationStatus::EnteredInError,
    ] {
        let condition = add(ConditionInput {
            verification,
            ..condition_input()
        })?;
        let before = fs::read("state/conditions.md")?;
        let head = git(&["rev-parse", "HEAD"])?;
        let error = resolve(&condition.id, Some("2026-06-30"), None).unwrap_err();
        assert!(error.to_string().contains("Cannot resolve"));
        assert_eq!(fs::read("state/conditions.md")?, before);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert_eq!(show(&condition.id)?.verification_status, verification);
    }
    Ok(())
}

#[test]
#[serial]
fn condition_load_rejects_missing_array_invalid_yaml_and_ambiguous_records() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    let duplicate = ConditionsState {
        conditions: vec![condition.clone(), condition.clone()],
        ..Default::default()
    };
    let mut unnamed = condition.clone();
    unnamed.name = " \t ".to_string();
    let mut no_id = condition.clone();
    no_id.id = " ".to_string();
    let invalid = [
        "---\ncondition: []\n---\n".to_string(),
        "---\nconditions: [\n---\n".to_string(),
        "---\nconditions: null\n---\n".to_string(),
        serde_yaml_ng::to_string(&duplicate)?,
        serde_yaml_ng::to_string(&ConditionsState {
            conditions: vec![unnamed],
            ..Default::default()
        })?,
        serde_yaml_ng::to_string(&ConditionsState {
            conditions: vec![no_id],
            ..Default::default()
        })?,
    ];
    for content in invalid {
        fs::write("state/conditions.md", &content)?;
        assert!(load().is_err(), "{content}");
        assert!(list(true, false).is_err());
        assert!(show(&condition.id).is_err());
        assert!(add(condition_input()).is_err());
        assert!(resolve(&condition.id, None, None).is_err());
        assert_eq!(fs::read_to_string("state/conditions.md")?, content);
        assert_eq!(parsed_entries()?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn condition_resolve_failure_restores_existing_state_and_unrelated_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    let before = fs::read("state/conditions.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    fs::write("unrelated.txt", "keep staged")?;
    git(&["add", "unrelated.txt"])?;
    let index = git(&["ls-files", "--stage"])?;
    git(&["config", "user.name", ""])?;
    assert!(resolve(&condition.id, Some("2026-06-30"), None).is_err());
    assert_eq!(fs::read("state/conditions.md")?, before);
    assert_eq!(git(&["rev-parse", "HEAD"])?, head);
    assert_eq!(git(&["ls-files", "--stage"])?, index);
    assert_eq!(parsed_entries()?.len(), 1);
    Ok(())
}

#[test]
#[serial]
fn condition_mutation_refuses_dirty_state_without_changing_files_or_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    let original = fs::read_to_string("state/conditions.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    for staged in [false, true] {
        let dirty = format!("{original}\nUncommitted clinical context\n");
        fs::write("state/conditions.md", &dirty)?;
        if staged {
            git(&["add", "state/conditions.md"])?;
        }
        let index = git(&["ls-files", "--stage"])?;
        for result in [add(condition_input()), resolve(&condition.id, None, None)] {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("uncommitted changes")
            );
        }
        assert_eq!(fs::read_to_string("state/conditions.md")?, dirty);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert_eq!(git(&["ls-files", "--stage"])?, index);
        assert_eq!(parsed_entries()?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn condition_cli_preserves_verification_in_text_and_emits_clean_json() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let condition = add(ConditionInput {
        verification: VerificationStatus::Provisional,
        ..condition_input()
    })?;
    let text = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["conditions", "list"])
        .output()?;
    assert!(text.status.success());
    assert!(
        String::from_utf8_lossy(&text.stdout).contains("active, provisional, problem-list-item")
    );
    let json = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["conditions", "list", "--json"])
        .output()?;
    assert!(json.status.success());
    let rows: serde_json::Value = serde_json::from_slice(&json.stdout)?;
    assert_eq!(rows[0], serde_json::to_value(&condition)?);
    let shown = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["conditions", "show", &condition.id, "--json"])
        .output()?;
    assert!(shown.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&shown.stdout)?,
        rows[0]
    );
    let help = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["conditions", "list", "--help"])
        .output()?;
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("refuted, and entered-in-error"));
    assert!(help.contains("Filter by problem-list-item category"));
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn condition_reads_and_mutations_refuse_live_and_dangling_state_symlinks() -> Result<()> {
    use std::os::unix::fs::symlink;
    for directory in [false, true] {
        for dangling in [false, true] {
            let _temp_dir = setup_with_git()?;
            fs::create_dir("outside")?;
            let original = "---\nconditions: []\n---\n";
            if !dangling {
                fs::write("outside/conditions.md", original)?;
            }
            if directory {
                fs::remove_dir("state")?;
                symlink(if dangling { "missing" } else { "outside" }, "state")?;
            } else {
                symlink("../outside/conditions.md", "state/conditions.md")?;
            }
            assert!(load().unwrap_err().to_string().contains("symlink"));
            assert!(
                list(true, false)
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                show("COND-test")
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                add(condition_input())
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                resolve("COND-test", None, None)
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert_eq!(fs::read_dir("journal")?.count(), 0);
            if !dangling {
                assert_eq!(fs::read_to_string("outside/conditions.md")?, original);
            } else {
                assert!(!Path::new("outside/conditions.md").exists());
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn condition_journal_symlink_failure_restores_state_and_index() -> Result<()> {
    use std::os::unix::fs::symlink;
    let _temp_dir = setup_with_git()?;
    let condition = add(condition_input())?;
    let original = fs::read("state/conditions.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    fs::write("unrelated.txt", "keep staged")?;
    git(&["add", "unrelated.txt"])?;
    let index = git(&["ls-files", "--stage"])?;
    fs::rename("journal", "original-journal")?;
    fs::create_dir("outside")?;
    symlink("outside", "journal")?;
    for result in [add(condition_input()), resolve(&condition.id, None, None)] {
        assert!(result.unwrap_err().to_string().contains("symlink"));
    }
    assert_eq!(fs::read("state/conditions.md")?, original);
    assert_eq!(git(&["rev-parse", "HEAD"])?, head);
    assert_eq!(git(&["ls-files", "--stage"])?, index);
    assert_eq!(fs::read_dir("outside")?.count(), 0);
    assert_eq!(fs::read_dir("original-journal")?.count(), 1);
    Ok(())
}

#[test]
#[serial]
fn condition_front_matter_preserves_delimiter_prefixed_yaml_keys_and_exact_body() -> Result<()> {
    for newline in ["\n", "\r\n"] {
        for closing_suffix in ["", " \t"] {
            let _temp_dir = setup_with_git()?;
            let body = format!(
                "{closing_suffix}{newline}{newline}# Clinical context{newline}---not a delimiter{newline}"
            );
            let original =
                format!("---{newline}conditions: []{newline}---source: imported{newline}---{body}");
            fs::write("state/conditions.md", &original)?;
            git(&["add", "state/conditions.md"])?;
            git(&["commit", "-m", "Seed condition state"])?;
            assert_eq!(load()?.extra["---source"].as_str(), Some("imported"));
            add(condition_input())?;
            let content = fs::read_to_string("state/conditions.md")?;
            assert!(content.ends_with(&format!("---{body}")));
            assert_eq!(load()?.extra["---source"].as_str(), Some("imported"));
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn condition_partial_journal_write_failure_rolls_back_state_and_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    add(condition_input())?;
    let original = fs::read("state/conditions.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    fs::write("unrelated.txt", "keep staged")?;
    git(&["add", "unrelated.txt"])?;
    let index = git(&["ls-files", "--stage"])?;
    // State contains the note once (<8 KiB); the journal includes both the
    // narrative and the snapshot (>8 KiB), so only the journal write fails.
    let output = Command::new("bash")
        .args([
            "-c",
            "trap '' XFSZ; ulimit -f 8; exec \"$@\"",
            "condition-write-test",
        ])
        .args([
            env!("CARGO_BIN_EXE_gitehr"),
            "conditions",
            "add",
            "--name",
            "Test condition",
            "--note",
        ])
        .arg("n".repeat(6000))
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to write journal entry"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("File too large"));
    assert_eq!(fs::read("state/conditions.md")?, original);
    assert_eq!(git(&["rev-parse", "HEAD"])?, head);
    assert_eq!(git(&["ls-files", "--stage"])?, index);
    assert_eq!(fs::read_dir("journal")?.count(), 1);
    assert_eq!(fs::read_dir("state")?.count(), 1);
    Ok(())
}
