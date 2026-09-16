// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::{Months, NaiveDate, Utc};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{contributor, git, journal, typed_state};

const STATE_FILE: &str = "acquisitions.md";
const DEFAULT_RIGHT_INVOKED: &str = "UK-GDPR-Art-15";

#[derive(Subcommand)]
pub enum AcquisitionCommands {
    #[command(about = "List record-acquisition requests (SARs, portal pulls, paper)")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(
            long,
            help = "Include resolved acquisitions (received, refused, nil-destroyed)"
        )]
        all: bool,
        #[arg(long, help = "Only unresolved acquisitions past their due date")]
        overdue: bool,
    },
    #[command(about = "Record a record-acquisition request as sent")]
    Add {
        #[arg(long, help = "Legal data controller the request was sent to")]
        controller: String,
        #[arg(long, help = "Treating site, if different from the controller")]
        site: Option<String>,
        #[arg(long, help = "Email/portal/postal address actually used")]
        contact: Option<String>,
        #[arg(long, help = "What this request is chasing")]
        context: Option<String>,
        #[arg(
            long,
            default_value = DEFAULT_RIGHT_INVOKED,
            help = "Right invoked, e.g. UK-GDPR-Art-15"
        )]
        right: String,
        #[arg(
            long = "identifier",
            help = "Identifier provided, e.g. NHS:1234567890; repeatable"
        )]
        identifiers: Vec<String>,
        #[arg(long, help = "Date the request was sent, YYYY-MM-DD")]
        date_sent: String,
        #[arg(long, help = "What ID was attached to the request")]
        id_provided: Option<String>,
        #[arg(long, help = "Optional note")]
        notes: Option<String>,
    },
    #[command(about = "Update the status and details of an acquisition request")]
    Update {
        #[arg(help = "Acquisition id")]
        id: String,
        #[arg(long, value_enum, help = "New status")]
        status: Option<AcquisitionStatus>,
        #[arg(
            long,
            help = "Date the controller acknowledged the request, YYYY-MM-DD"
        )]
        ack_date: Option<String>,
        #[arg(
            long,
            help = "Response due date, YYYY-MM-DD; overrides the date computed from --date-sent"
        )]
        due_date: Option<String>,
        #[arg(long, help = "What came back")]
        outcome: Option<String>,
        #[arg(
            long = "filed-to",
            help = "Journal entry or document reference the result was filed to; repeatable"
        )]
        filed_to: Vec<String>,
        #[arg(long, help = "Optional note")]
        notes: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum AcquisitionStatus {
    Drafted,
    Sent,
    Acknowledged,
    Received,
    Partial,
    NilDestroyed,
    Refused,
}

impl std::fmt::Display for AcquisitionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AcquisitionStatus::Drafted => "drafted",
            AcquisitionStatus::Sent => "sent",
            AcquisitionStatus::Acknowledged => "acknowledged",
            AcquisitionStatus::Received => "received",
            AcquisitionStatus::Partial => "partial",
            AcquisitionStatus::NilDestroyed => "nil-destroyed",
            AcquisitionStatus::Refused => "refused",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acquisition {
    pub id: String,
    pub controller: String,
    pub site: Option<String>,
    pub contact_used: Option<String>,
    pub care_context: Option<String>,
    pub right_invoked: String,
    #[serde(default)]
    pub identifiers_provided: Vec<String>,
    pub date_sent: String,
    pub id_provided: Option<String>,
    pub ack_date: Option<String>,
    pub due_date: Option<String>,
    pub status: AcquisitionStatus,
    pub outcome: Option<String>,
    #[serde(default)]
    pub filed_to: Vec<String>,
    pub notes: Option<String>,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcquisitionsState {
    #[serde(default)]
    pub acquisitions: Vec<Acquisition>,
}

#[derive(Debug, Clone)]
pub struct AcquisitionInput {
    pub controller: String,
    pub site: Option<String>,
    pub contact_used: Option<String>,
    pub care_context: Option<String>,
    pub right_invoked: String,
    pub identifiers_provided: Vec<String>,
    pub date_sent: String,
    pub id_provided: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AcquisitionUpdateInput {
    pub id: String,
    pub status: Option<AcquisitionStatus>,
    pub ack_date: Option<String>,
    pub due_date: Option<String>,
    pub outcome: Option<String>,
    pub filed_to: Vec<String>,
    pub notes: Option<String>,
}

pub fn run(command: AcquisitionCommands) -> Result<()> {
    match command {
        AcquisitionCommands::List { json, all, overdue } => {
            typed_state::ensure_gitehr_repository()?;
            let acquisitions = list(all, overdue)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&acquisitions)?);
            } else {
                print_human(&acquisitions);
            }
            Ok(())
        }
        AcquisitionCommands::Add {
            controller,
            site,
            contact,
            context,
            right,
            identifiers,
            date_sent,
            id_provided,
            notes,
        } => {
            add(AcquisitionInput {
                controller,
                site,
                contact_used: contact,
                care_context: context,
                right_invoked: right,
                identifiers_provided: identifiers,
                date_sent,
                id_provided,
                notes,
            })?;
            Ok(())
        }
        AcquisitionCommands::Update {
            id,
            status,
            ack_date,
            due_date,
            outcome,
            filed_to,
            notes,
        } => {
            update(AcquisitionUpdateInput {
                id,
                status,
                ack_date,
                due_date,
                outcome,
                filed_to,
                notes,
            })?;
            Ok(())
        }
    }
}

pub fn load() -> Result<AcquisitionsState> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn list(include_resolved: bool, overdue_only: bool) -> Result<Vec<Acquisition>> {
    let state = load()?;
    let today = Utc::now().date_naive();
    Ok(state
        .acquisitions
        .into_iter()
        .filter(|acquisition| include_resolved || !is_resolved(acquisition.status))
        .filter(|acquisition| {
            if !overdue_only {
                return true;
            }
            !is_resolved(acquisition.status)
                && acquisition
                    .due_date
                    .as_deref()
                    .and_then(|due| NaiveDate::parse_from_str(due, "%Y-%m-%d").ok())
                    .is_some_and(|due| due < today)
        })
        .collect())
}

pub fn add(input: AcquisitionInput) -> Result<Acquisition> {
    typed_state::ensure_gitehr_repository()?;
    let controller = require_text(&input.controller, "--controller")?;
    let right_invoked = require_text(&input.right_invoked, "--right")?;
    parse_date(&input.date_sent, "--date-sent")?;
    let identifiers_provided = input
        .identifiers_provided
        .iter()
        .filter_map(|value| cleaned_str(value))
        .collect::<Vec<_>>();

    let now = Utc::now();
    let acquisition = Acquisition {
        id: format!(
            "ACQ-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        controller: controller.to_string(),
        site: input.site.as_deref().and_then(cleaned_str),
        contact_used: input.contact_used.as_deref().and_then(cleaned_str),
        care_context: input.care_context.as_deref().and_then(cleaned_str),
        right_invoked: right_invoked.to_string(),
        identifiers_provided,
        // Set at the point of sending, not on acknowledgement: a controller
        // that never replies is exactly the one worth chasing, and a request
        // with no due date would never reach `list --overdue`.
        due_date: add_one_calendar_month(&input.date_sent),
        date_sent: input.date_sent,
        id_provided: input.id_provided.as_deref().and_then(cleaned_str),
        ack_date: None,
        status: AcquisitionStatus::Sent,
        outcome: None,
        filed_to: Vec::new(),
        notes: input.notes.as_deref().and_then(cleaned_str),
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
    };

    let mut state = load()?;
    state.acquisitions.push(acquisition.clone());
    persist_with_journal(
        &state,
        &format!(
            "Sent record request to {} ({})",
            acquisition.controller, acquisition.right_invoked
        ),
    )?;
    println!("Recorded acquisition: {}", acquisition.id);
    Ok(acquisition)
}

pub fn update(input: AcquisitionUpdateInput) -> Result<Acquisition> {
    typed_state::ensure_gitehr_repository()?;
    if input.status.is_none()
        && input.ack_date.is_none()
        && input.due_date.is_none()
        && input.outcome.is_none()
        && input.filed_to.is_empty()
        && input.notes.is_none()
    {
        anyhow::bail!(
            "No changes specified: pass --status, --ack-date, --due-date, --outcome, --filed-to, or --notes"
        );
    }
    if let Some(ack_date) = input.ack_date.as_deref() {
        parse_date(ack_date, "--ack-date")?;
    }
    if let Some(due_date) = input.due_date.as_deref() {
        parse_date(due_date, "--due-date")?;
    }

    let mut state = load()?;
    let acquisition = state
        .acquisitions
        .iter_mut()
        .find(|acquisition| acquisition.id == input.id)
        .ok_or_else(|| anyhow::anyhow!("Acquisition not found: {}", input.id))?;

    let mut changes = Vec::new();
    if let Some(status) = input.status {
        changes.push(format!("status -> {status}"));
        acquisition.status = status;
    }
    if let Some(ack_date) = input.ack_date.as_deref().and_then(cleaned_str) {
        changes.push(format!("acknowledged {ack_date}"));
        acquisition.ack_date = Some(ack_date);
    }
    if let Some(due_date) = input.due_date.as_deref().and_then(cleaned_str) {
        changes.push(format!("due {due_date}"));
        acquisition.due_date = Some(due_date);
    } else if acquisition.due_date.is_none() {
        // Only for registers written before `add` computed this. An
        // acknowledgement does not restart the clock, so it is never the
        // basis for the deadline.
        acquisition.due_date = add_one_calendar_month(&acquisition.date_sent);
    }
    if let Some(outcome) = input.outcome.as_deref().and_then(cleaned_str) {
        changes.push("outcome recorded".to_string());
        acquisition.outcome = Some(outcome);
    }
    for reference in input.filed_to.iter().filter_map(|value| cleaned_str(value)) {
        if !acquisition.filed_to.contains(&reference) {
            acquisition.filed_to.push(reference);
        }
    }
    if let Some(notes) = input.notes.as_deref().and_then(cleaned_str) {
        acquisition.notes = Some(notes);
    }

    let changed = acquisition.clone();
    let summary = if changes.is_empty() {
        format!(
            "Updated acquisition {} ({})",
            changed.id, changed.controller
        )
    } else {
        format!(
            "Updated acquisition {} ({}): {}",
            changed.id,
            changed.controller,
            changes.join(", ")
        )
    };

    persist_with_journal(&state, &summary)?;
    println!("Updated acquisition: {}", changed.id);
    Ok(changed)
}

fn is_resolved(status: AcquisitionStatus) -> bool {
    matches!(
        status,
        AcquisitionStatus::Received | AcquisitionStatus::NilDestroyed | AcquisitionStatus::Refused
    )
}

/// The corresponding date in the following month, clamped to the month's
/// length (31 January yields 28 or 29 February).
///
/// Under UK GDPR Article 12(3) a controller must respond within one month of
/// *receiving* the request. A patient cannot know the receipt date, so this
/// counts from the date the request was sent: the earliest the response can
/// be due, which is the right way to err for a register whose purpose is
/// chasing. A controller may extend by up to two further months for complex
/// or numerous requests, and may state a different date on acknowledging;
/// `--due-date` records that instead.
fn add_one_calendar_month(date: &str) -> Option<String> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    parsed
        .checked_add_months(Months::new(1))
        .map(|due| due.format("%Y-%m-%d").to_string())
}

fn persist_with_journal(state: &AcquisitionsState, journal_body: &str) -> Result<()> {
    let path = typed_state::write_front_matter(STATE_FILE, state)?;
    git::git_add(&path.to_string_lossy())?;
    journal::create_journal_entry(journal_body)?;
    Ok(())
}

fn parse_date(value: &str, label: &str) -> Result<()> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| anyhow::anyhow!("{} must use YYYY-MM-DD format", label))
}

fn require_text<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{} must not be empty", label);
    }
    Ok(trimmed)
}

fn cleaned_str(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn print_human(acquisitions: &[Acquisition]) {
    if acquisitions.is_empty() {
        println!("No acquisitions recorded.");
        return;
    }

    for acquisition in acquisitions {
        let due = acquisition
            .due_date
            .as_deref()
            .map(|due| format!(", due {due}"))
            .unwrap_or_default();
        println!(
            "{}  {} - {} (sent {}{})",
            acquisition.id, acquisition.controller, acquisition.status, acquisition.date_sent, due
        );
    }
}
