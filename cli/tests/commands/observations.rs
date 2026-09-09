// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

use gitehr::commands::journal::parsed_entries;
use gitehr::commands::observations::{
    Category, Observation, ObservationInput, ObservationStatus, ObservationsState, add, correct,
    list, load, show,
};

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

fn observation_input() -> ObservationInput {
    ObservationInput {
        name: "Blood pressure".to_string(),
        value: "128/82".to_string(),
        unit: Some("mmHg".to_string()),
        code: Some("loinc:85354-9".to_string()),
        category: Some(Category::VitalSigns),
        status: ObservationStatus::Final,
        effective: Some("2026-06-03".to_string()),
        interpretation: None,
        note: None,
    }
}

#[test]
#[serial]
fn observation_add_writes_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = observation_input();
    input.note = Some("Measured at annual review".to_string());
    let observation = add(input)?;
    assert!(observation.id.starts_with("OBS-"));

    let current = list(false, None)?;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].name, "Blood pressure");
    assert_eq!(current[0].value, "128/82");
    assert_eq!(current[0].unit.as_deref(), Some("mmHg"));
    assert_eq!(current[0].effective_at.as_deref(), Some("2026-06-03"));
    assert_eq!(current[0].status, ObservationStatus::Final);
    assert_eq!(current[0].category, Some(Category::VitalSigns));
    assert_eq!(
        current[0].note.as_deref(),
        Some("Measured at annual review")
    );

    let json = serde_json::to_value(&current[0])?;
    assert_eq!(json["code"], "loinc:85354-9");
    assert_eq!(json["status"], "final");
    assert_eq!(json["category"], "vital-signs");
    assert!(json.get("recorded_at").is_some());

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains(&format!(
        "Recorded observation: Blood pressure ({})",
        observation.id
    )));
    assert!(
        entries[0]
            .content
            .contains("Note: Measured at annual review")
    );

    Ok(())
}

#[test]
#[serial]
fn observation_add_defaults_effective_at_to_now_when_omitted() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = observation_input();
    input.effective = None;
    let observation = add(input)?;

    assert_eq!(
        observation.effective_at.as_deref(),
        Some(observation.recorded_at.as_str())
    );
    Ok(())
}

#[test]
#[serial]
fn observation_add_rejects_blank_effective_but_preserves_approximate_dates() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let error = add(ObservationInput {
        effective: Some(" \t ".to_string()),
        ..observation_input()
    })
    .unwrap_err();
    assert!(error.to_string().contains("--effective must not be empty"));
    assert!(!Path::new("state/observations.md").exists());
    assert_eq!(parsed_entries()?.len(), 0);

    let observation = add(ObservationInput {
        effective: Some("approximately June 2020".to_string()),
        ..observation_input()
    })?;
    assert_eq!(
        show(&observation.id)?.effective_at.as_deref(),
        Some("approximately June 2020")
    );
    Ok(())
}

#[test]
#[serial]
fn observation_correction_rejects_noop_and_blank_unit_without_consuming_correction() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    let before = fs::read("state/observations.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    let index = git(&["ls-files", "--stage"])?;
    for (value, unit, message) in [
        ("128/82", None, "must change"),
        (" 128/82 ", Some(" mmHg "), "must change"),
        ("134/88", Some(" \t "), "--unit must not be empty"),
    ] {
        let error = correct(&observation.id, value, unit, None).unwrap_err();
        assert!(error.to_string().contains(message));
        assert_eq!(fs::read("state/observations.md")?, before);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert_eq!(git(&["ls-files", "--stage"])?, index);
        assert_eq!(parsed_entries()?.len(), 1);
    }
    assert_eq!(
        correct(&observation.id, "134/88", None, None)?.status,
        ObservationStatus::Corrected
    );
    Ok(())
}

#[test]
#[serial]
fn observation_correction_preserves_history_even_if_status_was_changed_externally() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    correct(&observation.id, "134/88", None, Some("Transcription error"))?;
    let mut state = load()?;
    state.observations[0].status = ObservationStatus::Amended;
    let content = format!("---\n{}---\n", serde_yaml_ng::to_string(&state)?);
    fs::write("state/observations.md", &content)?;
    git(&["add", "state/observations.md"])?;
    git(&["commit", "-m", "External status update"])?;
    let head = git(&["rev-parse", "HEAD"])?;
    let error = correct(&observation.id, "140/90", None, None).unwrap_err();
    assert!(error.to_string().contains("already corrected"));
    assert_eq!(fs::read_to_string("state/observations.md")?, content);
    assert_eq!(git(&["rev-parse", "HEAD"])?, head);
    assert_eq!(parsed_entries()?.len(), 2);
    Ok(())
}

#[test]
#[serial]
fn observation_cli_accepts_negative_numeric_values_for_add_and_correct() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let added = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args([
            "observations",
            "add",
            "--name",
            "Base excess",
            "--value",
            "-2.5",
            "--unit",
            "mmol/L",
        ])
        .output()?;
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let observation = list(false, None)?.remove(0);
    assert_eq!(observation.value, "-2.5");
    let corrected = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args([
            "observations",
            "correct",
            &observation.id,
            "--value",
            "-3.5",
        ])
        .output()?;
    assert!(
        corrected.status.success(),
        "{}",
        String::from_utf8_lossy(&corrected.stderr)
    );
    assert_eq!(show(&observation.id)?.value, "-3.5");
    Ok(())
}

#[test]
#[serial]
fn observation_add_rejects_empty_name_or_value() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut missing_name = observation_input();
    missing_name.name = "   ".to_string();
    assert!(add(missing_name).is_err());

    let mut missing_value = observation_input();
    missing_value.value = "".to_string();
    assert!(add(missing_value).is_err());

    assert!(list(true, None)?.is_empty());
    Ok(())
}

#[test]
#[serial]
fn observation_add_accepts_pristine_scaffold_templates() -> Result<()> {
    for template in [
        "---\nobservations: []\n---\n",
        "---\r\nobservations: []\r\n---\r\n",
    ] {
        let _temp_dir = setup_with_git()?;
        fs::write("state/observations.md", template)?;

        add(observation_input())?;

        assert_eq!(list(false, None)?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn observation_add_refuses_non_pristine_untracked_state_when_git_hides_untracked_files()
-> Result<()> {
    let _temp_dir = setup_with_git()?;
    let original = "---\nsource_system: untracked-import\nobservations: []\n---\n";
    fs::write("state/observations.md", original)?;
    let configured = Command::new("git")
        .args(["config", "status.showUntrackedFiles", "no"])
        .status()?;
    assert!(configured.success());
    let hidden = Command::new("git")
        .args(["status", "--porcelain", "--", "state/observations.md"])
        .output()?;
    assert!(hidden.status.success());
    assert!(hidden.stdout.is_empty());

    let result = add(observation_input());

    assert!(result.is_err());
    assert_eq!(fs::read_to_string("state/observations.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    Ok(())
}

#[test]
#[serial]
fn observation_correct_updates_value_and_keeps_history() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let observation = add(observation_input())?;
    correct(
        &observation.id,
        "134/88",
        Some("mmHg"),
        Some("Original reading was transcribed incorrectly"),
    )?;

    let current = list(false, None)?;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].value, "134/88");
    assert_eq!(current[0].status, ObservationStatus::Corrected);
    assert_eq!(current[0].previous_value.as_deref(), Some("128/82"));
    assert_eq!(current[0].previous_unit.as_deref(), Some("mmHg"));
    assert_eq!(
        current[0].correction_reason.as_deref(),
        Some("Original reading was transcribed incorrectly")
    );

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| {
        entry.content.contains(&format!(
            "Corrected observation: Blood pressure ({}): 128/82 mmHg -> 134/88 mmHg",
            observation.id
        )) && entry
            .content
            .contains("Reason: Original reading was transcribed incorrectly")
    }));

    Ok(())
}

#[test]
#[serial]
fn observation_correct_keeps_previous_unit_when_omitted() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;

    let corrected = correct(&observation.id, "130/85", None, None)?;

    assert_eq!(corrected.unit.as_deref(), Some("mmHg"));
    assert_eq!(corrected.previous_unit.as_deref(), Some("mmHg"));
    Ok(())
}

#[test]
#[serial]
fn observation_correction_does_not_reuse_interpretation_of_the_original_result() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    for (value, unit) in [("134/88", None), ("128/82", Some("kPa"))] {
        let observation = add(ObservationInput {
            interpretation: Some("high".to_string()),
            ..observation_input()
        })?;
        let corrected = correct(&observation.id, value, unit, Some("Transcription error"))?;
        let stored = show(&observation.id)?;
        assert_eq!(stored.interpretation, None);
        assert_eq!(stored.previous_interpretation.as_deref(), Some("high"));
        assert_eq!(stored.recorded_at, observation.recorded_at);
        assert_eq!(stored.recorded_by, observation.recorded_by);
        assert_eq!(stored.effective_at, observation.effective_at);
        let text = Command::new(env!("CARGO_BIN_EXE_gitehr"))
            .args(["observations", "show", &observation.id])
            .output()?;
        assert!(text.status.success());
        let text = String::from_utf8_lossy(&text.stdout);
        assert!(
            text.lines()
                .any(|line| line == "previous_interpretation: high")
        );
        assert!(!text.lines().any(|line| line == "interpretation: high"));
        let entries = parsed_entries()?;
        let entry = entries
            .iter()
            .find(|entry| {
                entry.content.starts_with(&format!(
                    "Corrected observation: Blood pressure ({})",
                    observation.id
                ))
            })
            .unwrap();
        let snapshots: Vec<Observation> = entry
            .content
            .split("```yaml\n")
            .skip(1)
            .map(|block| serde_yaml_ng::from_str(block.split("```").next().unwrap()))
            .collect::<std::result::Result<_, _>>()?;
        assert_eq!(
            serde_json::to_value(snapshots)?,
            serde_json::to_value([observation, corrected])?
        );
    }
    Ok(())
}

#[test]
#[serial]
fn observation_correct_rejects_empty_value() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;

    let result = correct(&observation.id, "   ", None, None);

    assert!(result.is_err());
    assert_eq!(show(&observation.id)?.value, "128/82");
    Ok(())
}

#[test]
#[serial]
fn observation_correct_rejects_repeated_correction() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;

    correct(&observation.id, "134/88", None, Some("First correction"))?;
    let repeated = correct(&observation.id, "140/90", None, Some("Second correction"));

    assert!(repeated.is_err());
    assert!(
        repeated
            .unwrap_err()
            .to_string()
            .contains("already corrected")
    );
    let stored = show(&observation.id)?;
    assert_eq!(stored.value, "134/88");
    Ok(())
}

#[test]
#[serial]
fn observation_correct_refuses_cancelled_and_erroneous_entries() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    for status in [
        ObservationStatus::Cancelled,
        ObservationStatus::EnteredInError,
    ] {
        let observation = add(ObservationInput {
            status,
            ..observation_input()
        })?;
        let before = fs::read("state/observations.md")?;
        let head = git(&["rev-parse", "HEAD"])?;

        let error = correct(&observation.id, "134/88", None, None).unwrap_err();

        assert!(error.to_string().contains("Cannot correct"));
        assert_eq!(fs::read("state/observations.md")?, before);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert_eq!(show(&observation.id)?.status, status);
    }
    Ok(())
}

#[test]
#[serial]
fn observation_list_filters_by_category_and_currency() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut vital = observation_input();
    vital.name = "Temperature".to_string();
    vital.category = Some(Category::VitalSigns);
    add(vital)?;

    let mut lab = observation_input();
    lab.name = "HbA1c".to_string();
    lab.category = Some(Category::Laboratory);
    add(lab)?;

    let mut void = observation_input();
    void.name = "Mistaken entry".to_string();
    void.status = ObservationStatus::EnteredInError;
    add(void)?;

    let vitals_only = list(false, Some(Category::VitalSigns))?;
    assert_eq!(vitals_only.len(), 1);
    assert_eq!(vitals_only[0].name, "Temperature");

    let current = list(false, None)?;
    assert_eq!(current.len(), 2);

    let all = list(true, None)?;
    assert_eq!(all.len(), 3);

    Ok(())
}

#[test]
#[serial]
fn observation_show_finds_observation_regardless_of_status() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    correct(&observation.id, "134/88", None, None)?;

    let found = show(&observation.id)?;
    assert_eq!(found.id, observation.id);
    assert_eq!(found.status, ObservationStatus::Corrected);

    let missing = show("OBS-does-not-exist");
    assert!(missing.is_err());
    assert!(missing.unwrap_err().to_string().contains("not found"));

    Ok(())
}

#[test]
#[serial]
fn observation_mutation_preserves_unknown_fields_and_markdown_body() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write(
        "state/observations.md",
        r#"---
source_system: nhs-app
observations:
  - id: OBS-legacy
    name: Blood pressure
    code: "loinc:85354-9"
    category: vital-signs
    status: final
    value: "128/82"
    unit: mmHg
    interpretation: null
    effective_at: "2026-06-03"
    previous_value: null
    previous_unit: null
    correction_reason: null
    recorded_at: "2026-06-03T09:00:00Z"
    recorded_by: dr-example
    note: null
    source_id: external-123
---

# Observation notes

Imported verbatim context.
"#,
    )?;
    Command::new("git")
        .args(["add", "state/observations.md"])
        .output()?;
    Command::new("git")
        .args(["commit", "-m", "Seed observation state"])
        .output()?;

    let previous = show("OBS-legacy")?;
    let corrected = correct("OBS-legacy", "134/88", None, None)?;

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    let snapshots: Vec<Observation> = entries[0]
        .content
        .split("```yaml\n")
        .skip(1)
        .map(|block| serde_yaml_ng::from_str(block.split("```").next().unwrap()))
        .collect::<std::result::Result<_, _>>()?;
    assert_eq!(
        serde_json::to_value(snapshots)?,
        serde_json::to_value([previous, corrected])?
    );

    let content = fs::read_to_string("state/observations.md")?;
    assert!(content.contains("source_system: nhs-app"));
    assert!(content.contains("source_id: external-123"));
    assert!(content.contains("# Observation notes"));
    assert!(content.contains("Imported verbatim context."));
    Ok(())
}

#[test]
#[serial]
fn observation_commit_failure_restores_files_and_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    Command::new("git")
        .args(["config", "user.name", ""])
        .output()?;

    let result = add(observation_input());

    assert!(result.is_err());
    assert!(!Path::new("state/observations.md").exists());
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    assert!(status.stdout.is_empty());
    Ok(())
}

#[test]
#[serial]
fn observation_commit_leaves_unrelated_staged_changes_untouched() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    fs::write("unrelated.txt", "keep staged")?;
    Command::new("git")
        .args(["add", "unrelated.txt"])
        .output()?;

    add(observation_input())?;

    let committed = Command::new("git")
        .args(["show", "--name-only", "--format="])
        .output()?;
    let committed = String::from_utf8_lossy(&committed.stdout);
    assert!(committed.contains("state/observations.md"));
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
fn observation_mutation_preserves_state_file_mode_bits() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let _temp_dir = setup_with_git()?;
    fs::write("state/observations.md", "---\nobservations: []\n---\n")?;
    fs::set_permissions("state/observations.md", fs::Permissions::from_mode(0o640))?;

    add(observation_input())?;

    assert_eq!(
        fs::metadata("state/observations.md")?.permissions().mode() & 0o777,
        0o640
    );
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn observation_add_refuses_symlinked_state_file() -> Result<()> {
    use std::os::unix::fs::symlink;

    let _temp_dir = setup_with_git()?;
    fs::remove_dir("state")?;
    fs::create_dir("outside")?;
    let original = "---\nobservations: []\n---\n";
    fs::write("outside/observations.md", original)?;
    symlink("outside", "state")?;
    Command::new("git").args(["add", "state"]).output()?;
    Command::new("git")
        .args(["commit", "-m", "Track state symlink"])
        .output()?;

    let result = add(observation_input());

    assert!(result.is_err());
    assert!(fs::symlink_metadata("state")?.file_type().is_symlink());
    assert_eq!(fs::read_to_string("outside/observations.md")?, original);
    assert_eq!(fs::read_dir("journal")?.count(), 0);
    Ok(())
}

#[test]
#[serial]
fn observation_journal_preserves_the_complete_original_assertion() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(ObservationInput {
        interpretation: Some("high".to_string()),
        note: Some("Recheck in clinic".to_string()),
        ..observation_input()
    })?;
    let entries = parsed_entries()?;
    let yaml = entries[0].content.split("```yaml\n").nth(1).unwrap();
    let yaml = yaml.strip_suffix("```").unwrap();
    let recorded: Observation = serde_yaml_ng::from_str(yaml)?;
    assert_eq!(
        serde_json::to_value(recorded)?,
        serde_json::to_value(observation)?
    );
    Ok(())
}

#[test]
#[serial]
fn observation_load_rejects_missing_array_invalid_yaml_and_ambiguous_records() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    let duplicate = ObservationsState {
        observations: vec![observation.clone(), observation.clone()],
        ..Default::default()
    };
    let mut unnamed = observation.clone();
    unnamed.name = " \t ".to_string();
    let mut no_id = observation.clone();
    no_id.id = " ".to_string();
    let mut no_value = observation.clone();
    no_value.value = "".to_string();
    let mut no_recorded_at = observation.clone();
    no_recorded_at.recorded_at = " \t ".to_string();
    let mut blank_previous_value = observation.clone();
    blank_previous_value.previous_value = Some(" ".to_string());
    let invalid = [
        "---\nobservation: []\n---\n".to_string(),
        "---\nobservations: [\n---\n".to_string(),
        "---\nobservations: null\n---\n".to_string(),
        serde_yaml_ng::to_string(&duplicate)?,
        serde_yaml_ng::to_string(&ObservationsState {
            observations: vec![unnamed],
            ..Default::default()
        })?,
        serde_yaml_ng::to_string(&ObservationsState {
            observations: vec![no_id],
            ..Default::default()
        })?,
        serde_yaml_ng::to_string(&ObservationsState {
            observations: vec![no_value],
            ..Default::default()
        })?,
        serde_yaml_ng::to_string(&ObservationsState {
            observations: vec![no_recorded_at],
            ..Default::default()
        })?,
        serde_yaml_ng::to_string(&ObservationsState {
            observations: vec![blank_previous_value],
            ..Default::default()
        })?,
    ];
    for content in invalid {
        fs::write("state/observations.md", &content)?;
        assert!(load().is_err(), "{content}");
        assert!(list(true, None).is_err());
        assert!(show(&observation.id).is_err());
        assert!(add(observation_input()).is_err());
        assert!(correct(&observation.id, "134/88", None, None).is_err());
        assert_eq!(fs::read_to_string("state/observations.md")?, content);
        assert_eq!(parsed_entries()?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn observation_mutation_refuses_dirty_state_without_changing_files_or_index() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    let original = fs::read_to_string("state/observations.md")?;
    let head = git(&["rev-parse", "HEAD"])?;
    for staged in [false, true] {
        let dirty = format!("{original}\nUncommitted clinical context\n");
        fs::write("state/observations.md", &dirty)?;
        if staged {
            git(&["add", "state/observations.md"])?;
        }
        let index = git(&["ls-files", "--stage"])?;
        for result in [
            add(observation_input()),
            correct(&observation.id, "134/88", None, None),
        ] {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("uncommitted changes")
            );
        }
        assert_eq!(fs::read_to_string("state/observations.md")?, dirty);
        assert_eq!(git(&["rev-parse", "HEAD"])?, head);
        assert_eq!(git(&["ls-files", "--stage"])?, index);
        assert_eq!(parsed_entries()?.len(), 1);
    }
    Ok(())
}

#[test]
#[serial]
fn observation_cli_preserves_status_in_text_and_emits_clean_json() -> Result<()> {
    let _temp_dir = setup_with_git()?;
    let observation = add(observation_input())?;
    let text = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["observations", "list"])
        .output()?;
    assert!(text.status.success());
    assert!(String::from_utf8_lossy(&text.stdout).contains("final, vital-signs"));
    assert!(String::from_utf8_lossy(&text.stdout).contains("effective: 2026-06-03"));
    let json = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["observations", "list", "--json"])
        .output()?;
    assert!(json.status.success());
    let rows: serde_json::Value = serde_json::from_slice(&json.stdout)?;
    assert_eq!(rows[0], serde_json::to_value(&observation)?);
    let shown = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["observations", "show", &observation.id, "--json"])
        .output()?;
    assert!(shown.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&shown.stdout)?,
        rows[0]
    );
    let help = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args(["observations", "list", "--help"])
        .output()?;
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("Filter by category"));
    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn observation_reads_and_mutations_refuse_live_and_dangling_state_symlinks() -> Result<()> {
    use std::os::unix::fs::symlink;
    for directory in [false, true] {
        for dangling in [false, true] {
            let _temp_dir = setup_with_git()?;
            fs::create_dir("outside")?;
            let original = "---\nobservations: []\n---\n";
            if !dangling {
                fs::write("outside/observations.md", original)?;
            }
            if directory {
                fs::remove_dir("state")?;
                symlink(if dangling { "missing" } else { "outside" }, "state")?;
            } else {
                symlink("../outside/observations.md", "state/observations.md")?;
            }
            assert!(load().unwrap_err().to_string().contains("symlink"));
            assert!(
                list(true, None)
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                show("OBS-test")
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                add(observation_input())
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert!(
                correct("OBS-test", "134/88", None, None)
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            assert_eq!(fs::read_dir("journal")?.count(), 0);
            if !dangling {
                assert_eq!(fs::read_to_string("outside/observations.md")?, original);
            } else {
                assert!(!Path::new("outside/observations.md").exists());
            }
        }
    }
    Ok(())
}
