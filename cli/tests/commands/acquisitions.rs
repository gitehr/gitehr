// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;

use gitehr::commands::acquisitions::{
    Acquisition, AcquisitionInput, AcquisitionStatus, AcquisitionUpdateInput, add, list, update,
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

fn sample_input() -> AcquisitionInput {
    AcquisitionInput {
        controller: "York Teaching Hospitals NHS Trust".to_string(),
        site: Some("York Hospital".to_string()),
        contact_used: Some("dpo@york.nhs.uk".to_string()),
        care_context: Some("chasing 2019 discharge summary".to_string()),
        right_invoked: "UK-GDPR-Art-15".to_string(),
        identifiers_provided: vec!["NHS:1234567890".to_string()],
        date_sent: "2026-07-10".to_string(),
        id_provided: Some("passport copy".to_string()),
        notes: None,
    }
}

#[test]
#[serial]
fn acquisition_add_writes_sent_state_and_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let acquisition = add(sample_input())?;
    assert!(acquisition.id.starts_with("ACQ-"));
    assert_eq!(acquisition.status, AcquisitionStatus::Sent);
    assert_eq!(acquisition.controller, "York Teaching Hospitals NHS Trust");
    assert_eq!(acquisition.identifiers_provided, vec!["NHS:1234567890"]);

    let open = list(false, false)?;
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].id, acquisition.id);

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 1);
    assert!(
        entries[0]
            .content
            .contains("Sent record request to York Teaching Hospitals NHS Trust")
    );

    Ok(())
}

#[test]
#[serial]
fn acquisition_add_rejects_blank_controller() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let mut input = sample_input();
    input.controller = "   ".to_string();
    assert!(add(input).is_err());

    Ok(())
}

#[test]
#[serial]
fn acquisition_add_sets_the_due_date_from_the_date_sent() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    // The statutory clock runs from the controller receiving the request, not
    // from any reply, so a request is chaseable the moment it is recorded.
    let acquisition = add(sample_input())?;
    assert_eq!(acquisition.date_sent, "2026-07-10");
    assert_eq!(acquisition.due_date.as_deref(), Some("2026-08-10"));

    Ok(())
}

#[test]
#[serial]
fn acquisition_acknowledgement_does_not_move_the_due_date() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let acquisition = add(sample_input())?;
    let updated = update(AcquisitionUpdateInput {
        id: acquisition.id.clone(),
        status: Some(AcquisitionStatus::Acknowledged),
        ack_date: Some("2026-07-29".to_string()),
        ..Default::default()
    })?;

    assert_eq!(updated.status, AcquisitionStatus::Acknowledged);
    assert_eq!(updated.ack_date.as_deref(), Some("2026-07-29"));
    // A late acknowledgement would otherwise buy the controller three weeks.
    assert_eq!(updated.due_date.as_deref(), Some("2026-08-10"));

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn acquisition_ignored_request_becomes_overdue_without_any_reply() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    // A controller that never acknowledges is the one most worth chasing.
    let mut input = sample_input();
    input.date_sent = "2020-01-31".to_string();
    let ignored = add(input)?;

    // 31 January has no corresponding date in February, so it clamps.
    assert_eq!(ignored.due_date.as_deref(), Some("2020-02-29"));

    let overdue = list(false, true)?;
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue[0].id, ignored.id);
    assert!(overdue[0].ack_date.is_none());

    Ok(())
}

#[test]
#[serial]
fn acquisition_update_respects_explicit_due_date() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let acquisition = add(sample_input())?;
    let updated = update(AcquisitionUpdateInput {
        id: acquisition.id.clone(),
        ack_date: Some("2026-07-12".to_string()),
        due_date: Some("2026-09-01".to_string()),
        ..Default::default()
    })?;

    assert_eq!(updated.due_date.as_deref(), Some("2026-09-01"));

    Ok(())
}

#[test]
#[serial]
fn acquisition_update_rejects_no_changes() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let acquisition = add(sample_input())?;
    let result = update(AcquisitionUpdateInput {
        id: acquisition.id,
        ..Default::default()
    });

    assert!(result.is_err());

    Ok(())
}

#[test]
#[serial]
fn acquisition_received_is_hidden_from_default_list_but_kept_in_all() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let acquisition = add(sample_input())?;
    update(AcquisitionUpdateInput {
        id: acquisition.id.clone(),
        status: Some(AcquisitionStatus::Received),
        outcome: Some("14 documents received".to_string()),
        filed_to: vec!["documents/discharge.pdf".to_string()],
        ..Default::default()
    })?;

    assert!(list(false, false)?.is_empty());
    let all: Vec<Acquisition> = list(true, false)?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, AcquisitionStatus::Received);
    assert_eq!(all[0].outcome.as_deref(), Some("14 documents received"));
    assert_eq!(all[0].filed_to, vec!["documents/discharge.pdf"]);

    let entries = parsed_entries()?;
    assert_eq!(entries.len(), 2);

    Ok(())
}

#[test]
#[serial]
fn acquisition_overdue_filters_by_due_date_and_status() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let overdue = add(sample_input())?;
    update(AcquisitionUpdateInput {
        id: overdue.id.clone(),
        status: Some(AcquisitionStatus::Acknowledged),
        ack_date: Some("2020-01-01".to_string()),
        ..Default::default()
    })?;

    let mut future_input = sample_input();
    future_input.controller = "A Different Trust".to_string();
    let not_overdue = add(future_input)?;
    update(AcquisitionUpdateInput {
        id: not_overdue.id.clone(),
        status: Some(AcquisitionStatus::Acknowledged),
        due_date: Some("2999-01-01".to_string()),
        ..Default::default()
    })?;

    let overdue_only = list(false, true)?;
    assert_eq!(overdue_only.len(), 1);
    assert_eq!(overdue_only[0].id, overdue.id);

    Ok(())
}
