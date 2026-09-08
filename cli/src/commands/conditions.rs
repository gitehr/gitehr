// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value as YamlValue;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{contributor, typed_state};

const STATE_FILE: &str = "conditions.md";
const PRISTINE_STATE_FILES: &[&[u8]] = &[
    b"",
    b"---\nconditions: []\n---\n",
    b"---\r\nconditions: []\r\n---\r\n",
];

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConditionCommands {
    #[command(about = "List conditions")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(
            long,
            help = "Include all statuses, including inactive, resolved, refuted, and entered-in-error"
        )]
        all: bool,
        #[arg(
            long,
            help = "Filter by problem-list-item category (combine with --all for history)"
        )]
        problems: bool,
    },
    #[command(about = "Add a condition or problem-list item")]
    Add {
        #[arg(long, help = "Condition display name")]
        name: String,
        #[arg(long, value_enum, help = "Clinical status; defaults to active")]
        status: Option<ClinicalStatus>,
        #[arg(
            long,
            value_enum,
            help = "Verification status; defaults to unconfirmed"
        )]
        verification: Option<VerificationStatus>,
        #[arg(long, value_enum, help = "Category; defaults to problem-list-item")]
        category: Option<Category>,
        #[arg(long, help = "Onset, e.g. ISO date, year, or \"childhood\"")]
        onset: Option<String>,
        #[arg(long, help = "Terminology code, e.g. snomed:73211009")]
        code: Option<String>,
        #[arg(long = "body-site", help = "Anatomical body site")]
        body_site: Option<String>,
        #[arg(long, value_enum, help = "Laterality of the body site")]
        laterality: Option<Laterality>,
        #[arg(long, help = "Severity")]
        severity: Option<String>,
        #[arg(long, help = "Optional clinical note")]
        note: Option<String>,
    },
    #[command(about = "Mark a condition resolved")]
    Resolve {
        #[arg(help = "Condition id")]
        id: String,
        #[arg(long, help = "Abatement date in YYYY-MM-DD format; defaults to today")]
        date: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    #[command(about = "Show a single condition")]
    Show {
        #[arg(help = "Condition id")]
        id: String,
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ClinicalStatus {
    #[default]
    Active,
    Recurrence,
    Relapse,
    Inactive,
    Remission,
    Resolved,
}

impl std::fmt::Display for ClinicalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ClinicalStatus::Active => "active",
            ClinicalStatus::Recurrence => "recurrence",
            ClinicalStatus::Relapse => "relapse",
            ClinicalStatus::Inactive => "inactive",
            ClinicalStatus::Remission => "remission",
            ClinicalStatus::Resolved => "resolved",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
pub enum VerificationStatus {
    #[serde(rename = "unconfirmed")]
    #[default]
    Unconfirmed,
    #[serde(rename = "provisional")]
    Provisional,
    #[serde(rename = "differential")]
    Differential,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "refuted")]
    Refuted,
    #[serde(rename = "entered-in-error")]
    EnteredInError,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VerificationStatus::Unconfirmed => "unconfirmed",
            VerificationStatus::Provisional => "provisional",
            VerificationStatus::Differential => "differential",
            VerificationStatus::Confirmed => "confirmed",
            VerificationStatus::Refuted => "refuted",
            VerificationStatus::EnteredInError => "entered-in-error",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
pub enum Category {
    #[serde(rename = "problem-list-item")]
    #[default]
    ProblemListItem,
    #[serde(rename = "encounter-diagnosis")]
    EncounterDiagnosis,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Category::ProblemListItem => "problem-list-item",
            Category::EncounterDiagnosis => "encounter-diagnosis",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Laterality {
    Left,
    Right,
    Bilateral,
    Midline,
}

impl std::fmt::Display for Laterality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Laterality::Left => "left",
            Laterality::Right => "right",
            Laterality::Bilateral => "bilateral",
            Laterality::Midline => "midline",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub name: String,
    pub clinical_status: ClinicalStatus,
    pub verification_status: VerificationStatus,
    pub category: Category,
    pub onset: Option<String>,
    pub abatement: Option<String>,
    pub abatement_reason: Option<String>,
    pub body_site: Option<String>,
    pub laterality: Option<Laterality>,
    pub code: Option<String>,
    pub severity: Option<String>,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionsState {
    pub conditions: Vec<Condition>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ConditionInput {
    pub name: String,
    pub status: ClinicalStatus,
    pub verification: VerificationStatus,
    pub category: Category,
    pub onset: Option<String>,
    pub code: Option<String>,
    pub body_site: Option<String>,
    pub laterality: Option<Laterality>,
    pub severity: Option<String>,
    pub note: Option<String>,
}

pub fn run(command: ConditionCommands) -> Result<()> {
    match command {
        ConditionCommands::List {
            json,
            all,
            problems,
        } => {
            typed_state::ensure_gitehr_repository()?;
            let conditions = list(all, problems)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&conditions)?);
            } else {
                print_human(&conditions);
            }
            Ok(())
        }
        ConditionCommands::Add {
            name,
            status,
            verification,
            category,
            onset,
            code,
            body_site,
            laterality,
            severity,
            note,
        } => {
            add(ConditionInput {
                name,
                status: status.unwrap_or_default(),
                verification: verification.unwrap_or_default(),
                category: category.unwrap_or_default(),
                onset,
                code,
                body_site,
                laterality,
                severity,
                note,
            })?;
            Ok(())
        }
        ConditionCommands::Resolve { id, date, reason } => {
            resolve(&id, date.as_deref(), reason.as_deref())?;
            Ok(())
        }
        ConditionCommands::Show { id, json } => {
            let condition = show(&id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&condition)?);
            } else {
                print_show(&condition);
            }
            Ok(())
        }
    }
}

pub fn load() -> Result<ConditionsState> {
    typed_state::ensure_gitehr_repository()?;
    let state: ConditionsState = typed_state::read_front_matter(STATE_FILE)?;
    let mut ids = BTreeSet::new();
    for condition in &state.conditions {
        require_text(&condition.id, "Stored condition id")?;
        require_text(&condition.name, "Stored condition name")?;
        if !ids.insert(&condition.id) {
            anyhow::bail!(
                "Duplicate condition id in state/{STATE_FILE}: {}",
                condition.id
            );
        }
    }
    Ok(state)
}

fn is_current(condition: &Condition) -> bool {
    !matches!(
        condition.clinical_status,
        ClinicalStatus::Inactive | ClinicalStatus::Resolved
    ) && !matches!(
        condition.verification_status,
        VerificationStatus::Refuted | VerificationStatus::EnteredInError
    )
}

pub fn list(all: bool, problems_only: bool) -> Result<Vec<Condition>> {
    let state = load()?;
    Ok(state
        .conditions
        .into_iter()
        .filter(|condition| all || is_current(condition))
        .filter(|condition| !problems_only || condition.category == Category::ProblemListItem)
        .collect())
}

pub fn add(input: ConditionInput) -> Result<Condition> {
    typed_state::ensure_gitehr_repository()?;
    let name = require_text(&input.name, "--name")?;

    let now = Utc::now();
    let note = input.note.as_deref().and_then(cleaned_str);
    let condition = Condition {
        id: format!(
            "COND-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        name: name.to_string(),
        clinical_status: input.status,
        verification_status: input.verification,
        category: input.category,
        onset: input.onset.as_deref().and_then(cleaned_str),
        abatement: None,
        abatement_reason: None,
        body_site: input.body_site.as_deref().and_then(cleaned_str),
        laterality: input.laterality,
        code: input.code.as_deref().and_then(cleaned_str),
        severity: input.severity.as_deref().and_then(cleaned_str),
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        note: note.clone(),
        extra: BTreeMap::new(),
    };

    let mut state = load()?;
    state.conditions.push(condition.clone());
    let mut journal_body = format!("Recorded condition: {} ({})", condition.name, condition.id);
    if let Some(note) = note {
        journal_body.push_str(&format!("\n\nNote: {note}"));
    }
    // Keep the original assertion reconstructable from the immutable journal.
    journal_body.push_str(&format!(
        "\n\nRecorded condition state:\n\n```yaml\n{}```",
        serde_yaml_ng::to_string(&condition)?
    ));
    persist_with_journal(&state, &journal_body)?;
    println!("Recorded condition: {}", condition.id);
    Ok(condition)
}

pub fn resolve(id: &str, date: Option<&str>, reason: Option<&str>) -> Result<Condition> {
    typed_state::ensure_gitehr_repository()?;
    let abatement_date = match date {
        Some(date) => {
            validate_date(date, "--date")?;
            date.to_string()
        }
        None => Utc::now().format("%Y-%m-%d").to_string(),
    };

    let mut state = load()?;
    let condition = state
        .conditions
        .iter_mut()
        .find(|condition| condition.id == id)
        .ok_or_else(|| anyhow::anyhow!("Condition not found: {}", id))?;

    if condition.clinical_status == ClinicalStatus::Resolved {
        anyhow::bail!("Condition is already resolved: {id}");
    }
    if matches!(
        condition.verification_status,
        VerificationStatus::Refuted | VerificationStatus::EnteredInError
    ) {
        anyhow::bail!(
            "Cannot resolve a {} condition: {id}",
            condition.verification_status
        );
    }
    if let Some(onset) = condition.onset.as_deref()
        && let Ok(onset) = NaiveDate::parse_from_str(onset, "%Y-%m-%d")
    {
        let abatement = NaiveDate::parse_from_str(&abatement_date, "%Y-%m-%d")?;
        if abatement < onset {
            anyhow::bail!("--date must not be before the condition onset date");
        }
    }

    condition.clinical_status = ClinicalStatus::Resolved;
    condition.abatement = Some(abatement_date);
    condition.abatement_reason = reason.and_then(cleaned_str);
    let changed = condition.clone();

    let mut journal_body = format!(
        "Resolved condition: {} ({}) on {}",
        changed.name,
        changed.id,
        changed.abatement.as_deref().unwrap_or_default()
    );
    if let Some(reason) = changed.abatement_reason.as_deref() {
        journal_body.push_str(&format!("\n\nReason: {reason}"));
    }
    persist_with_journal(&state, &journal_body)?;
    println!("Resolved condition: {}", changed.id);
    Ok(changed)
}

pub fn show(id: &str) -> Result<Condition> {
    typed_state::ensure_gitehr_repository()?;
    let state = load()?;
    state
        .conditions
        .into_iter()
        .find(|condition| condition.id == id)
        .ok_or_else(|| anyhow::anyhow!("Condition not found: {}", id))
}

fn persist_with_journal(state: &ConditionsState, journal_body: &str) -> Result<()> {
    typed_state::write_with_journal(STATE_FILE, state, journal_body, PRISTINE_STATE_FILES)
}

fn validate_date(value: &str, label: &str) -> Result<()> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("{} must use YYYY-MM-DD format", label))?;
    if value.len() != 10 || date.format("%Y-%m-%d").to_string() != value {
        anyhow::bail!("{} must use YYYY-MM-DD format", label);
    }
    Ok(())
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

fn print_human(conditions: &[Condition]) {
    if conditions.is_empty() {
        println!("No conditions match this view.");
        return;
    }

    for condition in conditions {
        let mut location = String::new();
        if let Some(site) = condition.body_site.as_deref() {
            location.push_str(&format!(" [{}", site));
            if let Some(laterality) = condition.laterality {
                location.push_str(&format!(" ({})", laterality));
            }
            location.push(']');
        } else if let Some(laterality) = condition.laterality {
            location.push_str(&format!(" [{}]", laterality));
        }
        println!(
            "{}  {} ({}, {}, {}){}",
            condition.id,
            condition.name,
            condition.clinical_status,
            condition.verification_status,
            condition.category,
            location
        );
    }
}

fn print_show(condition: &Condition) {
    println!("id: {}", condition.id);
    println!("name: {}", condition.name);
    println!("clinical_status: {}", condition.clinical_status);
    println!("verification_status: {}", condition.verification_status);
    println!("category: {}", condition.category);
    if let Some(onset) = condition.onset.as_deref() {
        println!("onset: {onset}");
    }
    if let Some(abatement) = condition.abatement.as_deref() {
        println!("abatement: {abatement}");
    }
    if let Some(reason) = condition.abatement_reason.as_deref() {
        println!("abatement_reason: {reason}");
    }
    if let Some(body_site) = condition.body_site.as_deref() {
        println!("body_site: {body_site}");
    }
    if let Some(laterality) = condition.laterality {
        println!("laterality: {laterality}");
    }
    if let Some(code) = condition.code.as_deref() {
        println!("code: {code}");
    }
    if let Some(severity) = condition.severity.as_deref() {
        println!("severity: {severity}");
    }
    println!("recorded_at: {}", condition.recorded_at);
    if let Some(recorded_by) = condition.recorded_by.as_deref() {
        println!("recorded_by: {recorded_by}");
    }
    if let Some(note) = condition.note.as_deref() {
        println!("note: {note}");
    }
}
