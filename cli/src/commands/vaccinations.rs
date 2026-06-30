// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use uuid::Uuid;

use super::{contributor, git, journal, typed_state};

const STATE_FILE: &str = "vaccinations.md";

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum VaccinationCommands {
    #[command(about = "List recorded vaccinations")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(long, help = "Include entries marked entered-in-error")]
        all: bool,
    },
    #[command(about = "Record a vaccination or immunisation")]
    Add {
        #[arg(long, help = "Vaccine or immunisation display name")]
        vaccine: String,
        #[arg(long, help = "Administration date in YYYY-MM-DD format")]
        date: String,
        #[arg(long, help = "Dose sequence number, e.g. 1, 2, 3")]
        dose_sequence: Option<u32>,
        #[arg(long, help = "Target disease; repeatable")]
        target_disease: Vec<String>,
        #[arg(long, help = "Anatomical administration site")]
        site: Option<String>,
        #[arg(long, help = "Administration route")]
        route: Option<String>,
        #[arg(long, help = "Exact product administered")]
        product: Option<String>,
        #[arg(long)]
        manufacturer: Option<String>,
        #[arg(long)]
        batch_number: Option<String>,
        #[arg(long)]
        performer: Option<String>,
        #[arg(long, value_name = "PATH", help = "FHIR R4 Immunization JSON file")]
        fhir_json: Option<PathBuf>,
        #[arg(long, help = "Optional clinical note")]
        note: Option<String>,
    },
    #[command(about = "Mark a vaccination entry as entered in error")]
    EnteredInError {
        #[arg(help = "Vaccination id")]
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VaccinationStatus {
    Completed,
    EnteredInError,
}

impl std::fmt::Display for VaccinationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VaccinationStatus::Completed => "completed",
            VaccinationStatus::EnteredInError => "entered-in-error",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vaccination {
    pub id: String,
    pub status: VaccinationStatus,
    pub vaccine: String,
    pub date: String,
    pub dose_sequence: Option<u32>,
    #[serde(default)]
    pub target_disease: Vec<String>,
    pub anatomical_site: Option<String>,
    pub route: Option<String>,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
    pub batch_number: Option<String>,
    pub performer: Option<String>,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub entered_in_error_at: Option<String>,
    pub entered_in_error_reason: Option<String>,
    pub note: Option<String>,
    pub fhir_r4: Option<JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaccinationsState {
    #[serde(default)]
    pub vaccinations: Vec<Vaccination>,
}

#[derive(Debug, Clone)]
pub struct VaccinationInput {
    pub vaccine: String,
    pub date: String,
    pub dose_sequence: Option<u32>,
    pub target_disease: Vec<String>,
    pub anatomical_site: Option<String>,
    pub route: Option<String>,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
    pub batch_number: Option<String>,
    pub performer: Option<String>,
    pub fhir_json: Option<PathBuf>,
    pub note: Option<String>,
}

pub fn run(command: VaccinationCommands) -> Result<()> {
    match command {
        VaccinationCommands::List { json, all } => {
            typed_state::ensure_gitehr_repository()?;
            let vaccinations = list(all)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&vaccinations)?);
            } else {
                print_human(&vaccinations);
            }
            Ok(())
        }
        VaccinationCommands::Add {
            vaccine,
            date,
            dose_sequence,
            target_disease,
            site,
            route,
            product,
            manufacturer,
            batch_number,
            performer,
            fhir_json,
            note,
        } => {
            add(VaccinationInput {
                vaccine,
                date,
                dose_sequence,
                target_disease,
                anatomical_site: site,
                route,
                product,
                manufacturer,
                batch_number,
                performer,
                fhir_json,
                note,
            })?;
            Ok(())
        }
        VaccinationCommands::EnteredInError { id, reason } => {
            entered_in_error(&id, reason.as_deref())?;
            Ok(())
        }
    }
}

pub fn load() -> Result<VaccinationsState> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn list(include_entered_in_error: bool) -> Result<Vec<Vaccination>> {
    let state = load()?;
    Ok(state
        .vaccinations
        .into_iter()
        .filter(|vaccination| {
            include_entered_in_error || vaccination.status == VaccinationStatus::Completed
        })
        .collect())
}

pub fn add(input: VaccinationInput) -> Result<Vaccination> {
    typed_state::ensure_gitehr_repository()?;
    let vaccine = require_text(&input.vaccine, "--vaccine")?;
    NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("--date must use YYYY-MM-DD format"))?;
    let target_disease = input
        .target_disease
        .iter()
        .filter_map(|value| cleaned_str(value))
        .collect::<Vec<_>>();
    let fhir_r4 = match input.fhir_json.as_ref() {
        Some(path) => Some(read_fhir_json(path)?),
        None => None,
    };

    let now = Utc::now();
    let vaccination = Vaccination {
        id: format!(
            "VAC-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        status: VaccinationStatus::Completed,
        vaccine: vaccine.to_string(),
        date: input.date,
        dose_sequence: input.dose_sequence,
        target_disease,
        anatomical_site: input.anatomical_site.as_deref().and_then(cleaned_str),
        route: input.route.as_deref().and_then(cleaned_str),
        product: input.product.as_deref().and_then(cleaned_str),
        manufacturer: input.manufacturer.as_deref().and_then(cleaned_str),
        batch_number: input.batch_number.as_deref().and_then(cleaned_str),
        performer: input.performer.as_deref().and_then(cleaned_str),
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        entered_in_error_at: None,
        entered_in_error_reason: None,
        note: input.note.as_deref().and_then(cleaned_str),
        fhir_r4,
    };

    let mut state = load()?;
    state.vaccinations.push(vaccination.clone());
    persist_with_journal(
        &state,
        input.note.as_deref().unwrap_or(&format!(
            "Recorded vaccination: {} on {}",
            vaccination.vaccine, vaccination.date
        )),
    )?;
    println!("Recorded vaccination: {}", vaccination.id);
    Ok(vaccination)
}

pub fn entered_in_error(id: &str, reason: Option<&str>) -> Result<Vaccination> {
    typed_state::ensure_gitehr_repository()?;
    let mut state = load()?;
    let vaccination = state
        .vaccinations
        .iter_mut()
        .find(|vaccination| vaccination.id == id)
        .ok_or_else(|| anyhow::anyhow!("Vaccination not found: {}", id))?;

    vaccination.status = VaccinationStatus::EnteredInError;
    vaccination.entered_in_error_at = Some(Utc::now().to_rfc3339());
    vaccination.entered_in_error_reason = reason.and_then(cleaned_str);
    let changed = vaccination.clone();

    persist_with_journal(
        &state,
        reason.unwrap_or(&format!(
            "Marked vaccination entered in error: {}",
            changed.vaccine
        )),
    )?;
    println!("Marked vaccination entered in error: {}", changed.id);
    Ok(changed)
}

fn persist_with_journal(state: &VaccinationsState, journal_body: &str) -> Result<()> {
    let path = typed_state::write_front_matter(STATE_FILE, state)?;
    git::git_add(&path.to_string_lossy())?;
    journal::create_journal_entry(journal_body)?;
    Ok(())
}

fn read_fhir_json(path: &PathBuf) -> Result<JsonValue> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read FHIR JSON {}", path.display()))?;
    let value: JsonValue = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse FHIR JSON {}", path.display()))?;
    let resource_type = value
        .get("resourceType")
        .and_then(|resource_type| resource_type.as_str());
    if resource_type != Some("Immunization") {
        anyhow::bail!("--fhir-json must contain a FHIR R4 Immunization resource");
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

fn print_human(vaccinations: &[Vaccination]) {
    if vaccinations.is_empty() {
        println!("No vaccinations recorded.");
        return;
    }

    for vaccination in vaccinations {
        let dose = vaccination
            .dose_sequence
            .map(|dose| format!(" dose {}", dose))
            .unwrap_or_default();
        println!(
            "{}  {}{} on {} ({})",
            vaccination.id, vaccination.vaccine, dose, vaccination.date, vaccination.status
        );
    }
}
