// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use uuid::Uuid;

use super::typed_state::{Confidence, EvidenceLevel, Provenance, SourceType};
use super::{contributor, typed_state};

const STATE_FILE: &str = "family-history.md";
const PRISTINE_STATE_FILES: &[&[u8]] = &[
    b"",
    b"---\nfamily_history: []\n---\n",
    b"---\r\nfamily_history: []\r\n---\r\n",
];
/// Upper bound for `--onset-age`, to catch typos such as a year of birth.
const MAX_ONSET_AGE: u32 = 130;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum FamilyHistoryCommands {
    #[command(about = "List recorded family history")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(long, help = "Include entries marked entered-in-error")]
        all: bool,
    },
    #[command(about = "Record a condition in a blood relative")]
    Add {
        #[arg(
            long,
            help = "Relationship to the patient, e.g. mother, father, brother, maternal grandmother"
        )]
        relationship: String,
        #[arg(long, help = "Condition display name, e.g. myocardial infarction")]
        condition: String,
        #[arg(long, help = "Relative's age in whole years when the condition began")]
        onset_age: Option<u32>,
        #[arg(long, help = "The condition contributed to the relative's death")]
        contributed_to_death: bool,
        #[arg(
            long,
            value_name = "PATH",
            help = "FHIR R4 FamilyMemberHistory JSON file"
        )]
        fhir_json: Option<PathBuf>,
        #[arg(long, help = "Optional clinical note")]
        note: Option<String>,
        #[arg(long, value_enum, help = "Provenance: where this assertion came from")]
        source_type: Option<SourceType>,
        #[arg(long, help = "Provenance: controller, portal, device or citation")]
        source_detail: Option<String>,
        #[arg(long, help = "Provenance: acquisition id (see `gitehr acquisitions`)")]
        acquired_via: Option<String>,
        #[arg(
            long,
            help = "Provenance: existing Document under documents/ or imaging/ that substantiates this (its sha256 is recorded)"
        )]
        document_ref: Option<String>,
        #[arg(long, value_enum, help = "Provenance: strength of evidence")]
        evidence_level: Option<EvidenceLevel>,
        #[arg(long, value_enum, help = "Provenance: confidence in the assertion")]
        confidence: Option<Confidence>,
    },
    #[command(about = "Mark a family-history entry as entered in error")]
    EnteredInError {
        #[arg(help = "Family-history id")]
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FamilyHistoryStatus {
    Completed,
    EnteredInError,
}

impl std::fmt::Display for FamilyHistoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FamilyHistoryStatus::Completed => "completed",
            FamilyHistoryStatus::EnteredInError => "entered-in-error",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyHistoryEntry {
    pub id: String,
    pub status: FamilyHistoryStatus,
    pub relationship: String,
    pub condition: String,
    pub onset_age: Option<u32>,
    #[serde(default)]
    pub contributed_to_death: bool,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub entered_in_error_at: Option<String>,
    pub entered_in_error_reason: Option<String>,
    pub note: Option<String>,
    pub fhir_r4: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FamilyHistoryState {
    #[serde(default)]
    pub family_history: Vec<FamilyHistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct FamilyHistoryInput {
    pub relationship: String,
    pub condition: String,
    pub onset_age: Option<u32>,
    pub contributed_to_death: bool,
    pub fhir_json: Option<PathBuf>,
    pub note: Option<String>,
    pub provenance: Option<Provenance>,
}

pub fn run(command: FamilyHistoryCommands) -> Result<()> {
    match command {
        FamilyHistoryCommands::List { json, all } => {
            typed_state::ensure_gitehr_repository()?;
            let entries = list(all)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                print_human(&entries);
            }
            Ok(())
        }
        FamilyHistoryCommands::Add {
            relationship,
            condition,
            onset_age,
            contributed_to_death,
            fhir_json,
            note,
            source_type,
            source_detail,
            acquired_via,
            document_ref,
            evidence_level,
            confidence,
        } => {
            let provenance = Provenance::from_parts(
                source_type,
                source_detail.as_deref(),
                acquired_via.as_deref(),
                document_ref.as_deref(),
                evidence_level,
                confidence,
            )?;
            add(FamilyHistoryInput {
                relationship,
                condition,
                onset_age,
                contributed_to_death,
                fhir_json,
                note,
                provenance,
            })?;
            Ok(())
        }
        FamilyHistoryCommands::EnteredInError { id, reason } => {
            entered_in_error(&id, reason.as_deref())?;
            Ok(())
        }
    }
}

pub fn load() -> Result<FamilyHistoryState> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn list(include_entered_in_error: bool) -> Result<Vec<FamilyHistoryEntry>> {
    let state = load()?;
    Ok(state
        .family_history
        .into_iter()
        .filter(|entry| include_entered_in_error || entry.status == FamilyHistoryStatus::Completed)
        .collect())
}

pub fn add(input: FamilyHistoryInput) -> Result<FamilyHistoryEntry> {
    typed_state::ensure_gitehr_repository()?;
    let relationship = require_text(&input.relationship, "--relationship")?;
    let condition = require_text(&input.condition, "--condition")?;
    if let Some(age) = input.onset_age
        && age > MAX_ONSET_AGE
    {
        anyhow::bail!(
            "--onset-age must be an age in years (0-{}), not {}",
            MAX_ONSET_AGE,
            age
        );
    }
    let fhir_r4 = match input.fhir_json.as_ref() {
        Some(path) => Some(read_fhir_json(path)?),
        None => None,
    };

    let now = Utc::now();
    let entry = FamilyHistoryEntry {
        id: format!(
            "FH-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        status: FamilyHistoryStatus::Completed,
        relationship: relationship.to_string(),
        condition: condition.to_string(),
        onset_age: input.onset_age,
        contributed_to_death: input.contributed_to_death,
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        entered_in_error_at: None,
        entered_in_error_reason: None,
        note: input.note.as_deref().and_then(cleaned_str),
        fhir_r4,
        provenance: input.provenance,
    };

    let mut state = load()?;
    state.family_history.push(entry.clone());
    persist_with_journal(
        &state,
        input.note.as_deref().unwrap_or(&format!(
            "Recorded family history: {} - {}",
            entry.relationship, entry.condition
        )),
    )?;
    println!("Recorded family history: {}", entry.id);
    Ok(entry)
}

pub fn entered_in_error(id: &str, reason: Option<&str>) -> Result<FamilyHistoryEntry> {
    typed_state::ensure_gitehr_repository()?;
    let mut state = load()?;
    let entry = state
        .family_history
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| anyhow::anyhow!("Family-history entry not found: {}", id))?;

    entry.status = FamilyHistoryStatus::EnteredInError;
    entry.entered_in_error_at = Some(Utc::now().to_rfc3339());
    entry.entered_in_error_reason = reason.and_then(cleaned_str);
    let changed = entry.clone();

    persist_with_journal(
        &state,
        reason.unwrap_or(&format!(
            "Marked family history entered in error: {} - {}",
            changed.relationship, changed.condition
        )),
    )?;
    println!("Marked family history entered in error: {}", changed.id);
    Ok(changed)
}

fn persist_with_journal(state: &FamilyHistoryState, journal_body: &str) -> Result<()> {
    typed_state::write_with_journal(STATE_FILE, state, journal_body, PRISTINE_STATE_FILES)
}

fn read_fhir_json(path: &PathBuf) -> Result<JsonValue> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read FHIR JSON {}", path.display()))?;
    let value: JsonValue = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse FHIR JSON {}", path.display()))?;
    let resource_type = value
        .get("resourceType")
        .and_then(|resource_type| resource_type.as_str());
    if resource_type != Some("FamilyMemberHistory") {
        anyhow::bail!("--fhir-json must contain a FHIR R4 FamilyMemberHistory resource");
    }
    Ok(value)
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

fn print_human(entries: &[FamilyHistoryEntry]) {
    if entries.is_empty() {
        println!("No family history recorded.");
        return;
    }

    for entry in entries {
        let onset = entry
            .onset_age
            .map(|age| format!(", onset age {}", age))
            .unwrap_or_default();
        let death = if entry.contributed_to_death {
            ", contributed to death"
        } else {
            ""
        };
        println!(
            "{}  {}: {}{}{} ({})",
            entry.id, entry.relationship, entry.condition, onset, death, entry.status
        );
    }
}
