// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::Utc;
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value as YamlValue;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{contributor, typed_state};

const STATE_FILE: &str = "observations.md";
const PRISTINE_STATE_FILES: &[&[u8]] = &[
    b"",
    b"---\nobservations: []\n---\n",
    b"---\r\nobservations: []\r\n---\r\n",
];

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ObservationCommands {
    #[command(about = "List observations")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(long, help = "Include cancelled and entered-in-error observations")]
        all: bool,
        #[arg(long, value_enum, help = "Filter by category")]
        category: Option<Category>,
    },
    #[command(about = "Add an observation, such as a vital sign or lab result")]
    Add {
        #[arg(long, help = "Observation display name, e.g. \"Blood pressure\"")]
        name: String,
        #[arg(long, help = "Recorded value, e.g. \"128/82\" or \"37.1\"")]
        value: String,
        #[arg(long, help = "Unit of measurement, e.g. mmHg")]
        unit: Option<String>,
        #[arg(long, help = "Terminology code, e.g. loinc:85354-9")]
        code: Option<String>,
        #[arg(long, value_enum, help = "Category, e.g. vital-signs")]
        category: Option<Category>,
        #[arg(long, value_enum, help = "Status; defaults to final")]
        status: Option<ObservationStatus>,
        #[arg(
            long,
            help = "When the observation was made (ISO date or date-time); defaults to now"
        )]
        effective: Option<String>,
        #[arg(long, help = "Interpretation, e.g. high, low, normal, critical")]
        interpretation: Option<String>,
        #[arg(long, help = "Optional clinical note")]
        note: Option<String>,
    },
    #[command(about = "Correct a previously recorded observation's value")]
    Correct {
        #[arg(help = "Observation id")]
        id: String,
        #[arg(long, help = "Corrected value")]
        value: String,
        #[arg(long, help = "Corrected unit; keeps the previous unit if omitted")]
        unit: Option<String>,
        #[arg(long, help = "Reason for the correction")]
        reason: Option<String>,
    },
    #[command(about = "Show a single observation")]
    Show {
        #[arg(help = "Observation id")]
        id: String,
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum)]
pub enum ObservationStatus {
    #[serde(rename = "registered")]
    Registered,
    #[serde(rename = "preliminary")]
    Preliminary,
    #[serde(rename = "final")]
    #[default]
    Final,
    #[serde(rename = "amended")]
    Amended,
    #[serde(rename = "corrected")]
    Corrected,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "entered-in-error")]
    EnteredInError,
    #[serde(rename = "unknown")]
    Unknown,
}

impl std::fmt::Display for ObservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ObservationStatus::Registered => "registered",
            ObservationStatus::Preliminary => "preliminary",
            ObservationStatus::Final => "final",
            ObservationStatus::Amended => "amended",
            ObservationStatus::Corrected => "corrected",
            ObservationStatus::Cancelled => "cancelled",
            ObservationStatus::EnteredInError => "entered-in-error",
            ObservationStatus::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Category {
    #[serde(rename = "vital-signs")]
    VitalSigns,
    #[serde(rename = "laboratory")]
    Laboratory,
    #[serde(rename = "social-history")]
    SocialHistory,
    #[serde(rename = "imaging")]
    Imaging,
    #[serde(rename = "procedure")]
    Procedure,
    #[serde(rename = "survey")]
    Survey,
    #[serde(rename = "exam")]
    Exam,
    #[serde(rename = "therapy")]
    Therapy,
    #[serde(rename = "activity")]
    Activity,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Category::VitalSigns => "vital-signs",
            Category::Laboratory => "laboratory",
            Category::SocialHistory => "social-history",
            Category::Imaging => "imaging",
            Category::Procedure => "procedure",
            Category::Survey => "survey",
            Category::Exam => "exam",
            Category::Therapy => "therapy",
            Category::Activity => "activity",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub category: Option<Category>,
    pub status: ObservationStatus,
    pub value: String,
    pub unit: Option<String>,
    pub interpretation: Option<String>,
    pub effective_at: Option<String>,
    pub previous_value: Option<String>,
    pub previous_unit: Option<String>,
    pub correction_reason: Option<String>,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservationsState {
    pub observations: Vec<Observation>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ObservationInput {
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub code: Option<String>,
    pub category: Option<Category>,
    pub status: ObservationStatus,
    pub effective: Option<String>,
    pub interpretation: Option<String>,
    pub note: Option<String>,
}

pub fn run(command: ObservationCommands) -> Result<()> {
    match command {
        ObservationCommands::List {
            json,
            all,
            category,
        } => {
            typed_state::ensure_gitehr_repository()?;
            let observations = list(all, category)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&observations)?);
            } else {
                print_human(&observations);
            }
            Ok(())
        }
        ObservationCommands::Add {
            name,
            value,
            unit,
            code,
            category,
            status,
            effective,
            interpretation,
            note,
        } => {
            add(ObservationInput {
                name,
                value,
                unit,
                code,
                category,
                status: status.unwrap_or_default(),
                effective,
                interpretation,
                note,
            })?;
            Ok(())
        }
        ObservationCommands::Correct {
            id,
            value,
            unit,
            reason,
        } => {
            correct(&id, &value, unit.as_deref(), reason.as_deref())?;
            Ok(())
        }
        ObservationCommands::Show { id, json } => {
            let observation = show(&id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&observation)?);
            } else {
                print_show(&observation);
            }
            Ok(())
        }
    }
}

pub fn load() -> Result<ObservationsState> {
    typed_state::ensure_gitehr_repository()?;
    let state: ObservationsState = typed_state::read_front_matter(STATE_FILE)?;
    let mut ids = BTreeSet::new();
    for observation in &state.observations {
        require_text(&observation.id, "Stored observation id")?;
        require_text(&observation.name, "Stored observation name")?;
        require_text(&observation.value, "Stored observation value")?;
        if !ids.insert(&observation.id) {
            anyhow::bail!(
                "Duplicate observation id in state/{STATE_FILE}: {}",
                observation.id
            );
        }
    }
    Ok(state)
}

fn is_current(observation: &Observation) -> bool {
    !matches!(
        observation.status,
        ObservationStatus::Cancelled | ObservationStatus::EnteredInError
    )
}

pub fn list(all: bool, category: Option<Category>) -> Result<Vec<Observation>> {
    let state = load()?;
    Ok(state
        .observations
        .into_iter()
        .filter(|observation| all || is_current(observation))
        .filter(|observation| {
            category.is_none_or(|category| observation.category == Some(category))
        })
        .collect())
}

pub fn add(input: ObservationInput) -> Result<Observation> {
    typed_state::ensure_gitehr_repository()?;
    let name = require_text(&input.name, "--name")?;
    let value = require_text(&input.value, "--value")?;

    let now = Utc::now();
    let note = input.note.as_deref().and_then(cleaned_str);
    let observation = Observation {
        id: format!(
            "OBS-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        name: name.to_string(),
        code: input.code.as_deref().and_then(cleaned_str),
        category: input.category,
        status: input.status,
        value: value.to_string(),
        unit: input.unit.as_deref().and_then(cleaned_str),
        interpretation: input.interpretation.as_deref().and_then(cleaned_str),
        effective_at: Some(
            input
                .effective
                .as_deref()
                .and_then(cleaned_str)
                .unwrap_or_else(|| now.to_rfc3339()),
        ),
        previous_value: None,
        previous_unit: None,
        correction_reason: None,
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        note: note.clone(),
        extra: BTreeMap::new(),
    };

    let mut state = load()?;
    state.observations.push(observation.clone());
    let mut journal_body = format!(
        "Recorded observation: {} ({})",
        observation.name, observation.id
    );
    if let Some(note) = note {
        journal_body.push_str(&format!("\n\nNote: {note}"));
    }
    // Keep the original assertion reconstructable from the immutable journal.
    journal_body.push_str(&format!(
        "\n\nRecorded observation state:\n\n```yaml\n{}```",
        serde_yaml_ng::to_string(&observation)?
    ));
    persist_with_journal(&state, &journal_body)?;
    println!("Recorded observation: {}", observation.id);
    Ok(observation)
}

pub fn correct(
    id: &str,
    value: &str,
    unit: Option<&str>,
    reason: Option<&str>,
) -> Result<Observation> {
    typed_state::ensure_gitehr_repository()?;
    let new_value = require_text(value, "--value")?.to_string();

    let mut state = load()?;
    let observation = state
        .observations
        .iter_mut()
        .find(|observation| observation.id == id)
        .ok_or_else(|| anyhow::anyhow!("Observation not found: {}", id))?;

    if observation.status == ObservationStatus::Corrected {
        anyhow::bail!("Observation is already corrected: {id}");
    }
    if matches!(
        observation.status,
        ObservationStatus::Cancelled | ObservationStatus::EnteredInError
    ) {
        anyhow::bail!("Cannot correct a {} observation: {id}", observation.status);
    }

    let previous_value = observation.value.clone();
    let previous_unit = observation.unit.clone();
    observation.previous_value = Some(previous_value.clone());
    observation.previous_unit = previous_unit.clone();
    observation.value = new_value.clone();
    if let Some(unit) = unit {
        observation.unit = cleaned_str(unit);
    }
    observation.correction_reason = reason.and_then(cleaned_str);
    observation.status = ObservationStatus::Corrected;
    let changed = observation.clone();

    let mut journal_body = format!(
        "Corrected observation: {} ({}): {}{} -> {}{}",
        changed.name,
        changed.id,
        previous_value,
        previous_unit
            .as_deref()
            .map(|unit| format!(" {unit}"))
            .unwrap_or_default(),
        changed.value,
        changed
            .unit
            .as_deref()
            .map(|unit| format!(" {unit}"))
            .unwrap_or_default(),
    );
    if let Some(reason) = changed.correction_reason.as_deref() {
        journal_body.push_str(&format!("\n\nReason: {reason}"));
    }
    persist_with_journal(&state, &journal_body)?;
    println!("Corrected observation: {}", changed.id);
    Ok(changed)
}

pub fn show(id: &str) -> Result<Observation> {
    typed_state::ensure_gitehr_repository()?;
    let state = load()?;
    state
        .observations
        .into_iter()
        .find(|observation| observation.id == id)
        .ok_or_else(|| anyhow::anyhow!("Observation not found: {}", id))
}

fn persist_with_journal(state: &ObservationsState, journal_body: &str) -> Result<()> {
    typed_state::write_with_journal(STATE_FILE, state, journal_body, PRISTINE_STATE_FILES)
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

fn print_human(observations: &[Observation]) {
    if observations.is_empty() {
        println!("No observations match this view.");
        return;
    }

    for observation in observations {
        let unit = observation
            .unit
            .as_deref()
            .map(|unit| format!(" {unit}"))
            .unwrap_or_default();
        let category = observation
            .category
            .map(|category| format!(", {category}"))
            .unwrap_or_default();
        println!(
            "{}  {}: {}{} ({}{})",
            observation.id, observation.name, observation.value, unit, observation.status, category
        );
    }
}

fn print_show(observation: &Observation) {
    println!("id: {}", observation.id);
    println!("name: {}", observation.name);
    if let Some(code) = observation.code.as_deref() {
        println!("code: {code}");
    }
    if let Some(category) = observation.category {
        println!("category: {category}");
    }
    println!("status: {}", observation.status);
    println!("value: {}", observation.value);
    if let Some(unit) = observation.unit.as_deref() {
        println!("unit: {unit}");
    }
    if let Some(interpretation) = observation.interpretation.as_deref() {
        println!("interpretation: {interpretation}");
    }
    if let Some(effective_at) = observation.effective_at.as_deref() {
        println!("effective_at: {effective_at}");
    }
    if let Some(previous_value) = observation.previous_value.as_deref() {
        println!("previous_value: {previous_value}");
    }
    if let Some(previous_unit) = observation.previous_unit.as_deref() {
        println!("previous_unit: {previous_unit}");
    }
    if let Some(reason) = observation.correction_reason.as_deref() {
        println!("correction_reason: {reason}");
    }
    println!("recorded_at: {}", observation.recorded_at);
    if let Some(recorded_by) = observation.recorded_by.as_deref() {
        println!("recorded_by: {recorded_by}");
    }
    if let Some(note) = observation.note.as_deref() {
        println!("note: {note}");
    }
}
