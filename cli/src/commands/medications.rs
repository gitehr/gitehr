// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value as YamlValue;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{contributor, typed_state};

const STATE_FILE: &str = "medications.md";
const PRISTINE_STATE_FILES: &[&[u8]] = &[
    b"",
    b"---\nmedications: []\n---\n",
    b"---\r\nmedications: []\r\n---\r\n",
];

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum MedicationCommands {
    #[command(about = "List medications")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(long, help = "Include stopped medications")]
        all: bool,
    },
    #[command(about = "Add a medication or supplement")]
    Add {
        #[arg(long, help = "Medication or supplement display name")]
        name: String,
        #[arg(long, help = "Dose, e.g. 500mg")]
        dose: Option<String>,
        #[arg(long, help = "Route, e.g. oral")]
        route: Option<String>,
        #[arg(long, help = "Frequency, e.g. twice daily")]
        frequency: Option<String>,
        #[arg(long, help = "Clinical indication")]
        indication: Option<String>,
        #[arg(long, help = "Prescriber name")]
        prescriber: Option<String>,
        #[arg(long, help = "Start date in YYYY-MM-DD format")]
        started: Option<String>,
        #[arg(
            long,
            help = "Mark as a supplement rather than a prescribed medication"
        )]
        supplement: bool,
        #[arg(long, help = "Optional clinical note")]
        note: Option<String>,
    },
    #[command(about = "Mark a medication stopped")]
    Stop {
        #[arg(help = "Medication id")]
        id: String,
        #[arg(long, help = "Stop date in YYYY-MM-DD format; defaults to today")]
        date: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MedicationStatus {
    Active,
    Stopped,
}

impl std::fmt::Display for MedicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MedicationStatus::Active => "active",
            MedicationStatus::Stopped => "stopped",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub id: String,
    pub name: String,
    pub dose: Option<String>,
    pub route: Option<String>,
    pub frequency: Option<String>,
    pub indication: Option<String>,
    pub prescriber: Option<String>,
    #[serde(default)]
    pub supplement: bool,
    pub status: MedicationStatus,
    pub started: Option<String>,
    pub stopped: Option<String>,
    pub stopped_reason: Option<String>,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MedicationsState {
    #[serde(default)]
    pub medications: Vec<Medication>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default)]
pub struct MedicationInput {
    pub name: String,
    pub dose: Option<String>,
    pub route: Option<String>,
    pub frequency: Option<String>,
    pub indication: Option<String>,
    pub prescriber: Option<String>,
    pub started: Option<String>,
    pub supplement: bool,
    pub note: Option<String>,
}

pub fn run(command: MedicationCommands) -> Result<()> {
    match command {
        MedicationCommands::List { json, all } => {
            typed_state::ensure_gitehr_repository()?;
            let medications = list(all)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&medications)?);
            } else {
                print_human(&medications);
            }
            Ok(())
        }
        MedicationCommands::Add {
            name,
            dose,
            route,
            frequency,
            indication,
            prescriber,
            started,
            supplement,
            note,
        } => {
            add(MedicationInput {
                name,
                dose,
                route,
                frequency,
                indication,
                prescriber,
                started,
                supplement,
                note,
            })?;
            Ok(())
        }
        MedicationCommands::Stop { id, date, reason } => {
            stop(&id, date.as_deref(), reason.as_deref())?;
            Ok(())
        }
    }
}

pub fn load() -> Result<MedicationsState> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn list(include_stopped: bool) -> Result<Vec<Medication>> {
    let state = load()?;
    Ok(state
        .medications
        .into_iter()
        .filter(|medication| include_stopped || medication.status == MedicationStatus::Active)
        .collect())
}

pub fn add(input: MedicationInput) -> Result<Medication> {
    typed_state::ensure_gitehr_repository()?;
    let name = require_text(&input.name, "--name")?;
    if let Some(started) = input.started.as_deref() {
        validate_date(started, "--started")?;
    }

    let now = Utc::now();
    let note = input.note.as_deref().and_then(cleaned_str);
    let medication = Medication {
        id: format!(
            "MED-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        name: name.to_string(),
        dose: input.dose.as_deref().and_then(cleaned_str),
        route: input.route.as_deref().and_then(cleaned_str),
        frequency: input.frequency.as_deref().and_then(cleaned_str),
        indication: input.indication.as_deref().and_then(cleaned_str),
        prescriber: input.prescriber.as_deref().and_then(cleaned_str),
        supplement: input.supplement,
        status: MedicationStatus::Active,
        started: input.started,
        stopped: None,
        stopped_reason: None,
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        note: note.clone(),
        extra: BTreeMap::new(),
    };

    let mut state = load()?;
    state.medications.push(medication.clone());
    let mut journal_body = format!("Added medication: {} ({})", medication.name, medication.id);
    if let Some(note) = note {
        journal_body.push_str(&format!("\n\nNote: {note}"));
    }
    persist_with_journal(&state, &journal_body)?;
    println!("Added medication: {}", medication.id);
    Ok(medication)
}

pub fn stop(id: &str, date: Option<&str>, reason: Option<&str>) -> Result<Medication> {
    typed_state::ensure_gitehr_repository()?;
    let stopped_date = match date {
        Some(date) => {
            validate_date(date, "--date")?;
            date.to_string()
        }
        None => Utc::now().format("%Y-%m-%d").to_string(),
    };

    let mut state = load()?;
    let medication = state
        .medications
        .iter_mut()
        .find(|medication| medication.id == id)
        .ok_or_else(|| anyhow::anyhow!("Medication not found: {}", id))?;

    if medication.status == MedicationStatus::Stopped {
        anyhow::bail!("Medication is already stopped: {id}");
    }
    if let Some(started) = medication.started.as_deref() {
        let started = NaiveDate::parse_from_str(started, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Stored medication start date is invalid: {started}"))?;
        let stopped = NaiveDate::parse_from_str(&stopped_date, "%Y-%m-%d")?;
        if stopped < started {
            anyhow::bail!("--date must not be before the medication start date");
        }
    }

    medication.status = MedicationStatus::Stopped;
    medication.stopped = Some(stopped_date);
    medication.stopped_reason = reason.and_then(cleaned_str);
    let changed = medication.clone();

    let mut journal_body = format!(
        "Stopped medication: {} ({}) on {}",
        changed.name,
        changed.id,
        changed.stopped.as_deref().unwrap_or_default()
    );
    if let Some(reason) = changed.stopped_reason.as_deref() {
        journal_body.push_str(&format!("\n\nReason: {reason}"));
    }
    persist_with_journal(&state, &journal_body)?;
    println!("Stopped medication: {}", changed.id);
    Ok(changed)
}

fn persist_with_journal(state: &MedicationsState, journal_body: &str) -> Result<()> {
    typed_state::write_with_journal(STATE_FILE, state, journal_body, PRISTINE_STATE_FILES)
}

fn validate_date(value: &str, label: &str) -> Result<()> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("{} must use YYYY-MM-DD format", label))?;
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

fn print_human(medications: &[Medication]) {
    if medications.is_empty() {
        println!("No active medications recorded.");
        return;
    }

    for medication in medications {
        let dose = medication
            .dose
            .as_deref()
            .map(|dose| format!(" {}", dose))
            .unwrap_or_default();
        let kind = if medication.supplement {
            " (supplement)"
        } else {
            ""
        };
        println!(
            "{}  {}{}{} ({})",
            medication.id, medication.name, dose, kind, medication.status
        );
    }
}
