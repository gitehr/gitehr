// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::NaiveDate;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use super::{git, journal, typed_state};

const STATE_FILE: &str = "demographics.md";

#[derive(Subcommand)]
pub enum DemographicsCommands {
    #[command(about = "Show current demographics")]
    Show {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
    },
    #[command(about = "Update current demographics")]
    Set {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        full_name: Option<String>,
        #[arg(long)]
        preferred_name: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long, help = "Date of birth in YYYY-MM-DD format")]
        date_of_birth: Option<String>,
        #[arg(long)]
        nhs_number: Option<String>,
        #[arg(
            long,
            help = "Additional identifier as type:value",
            value_name = "type:value"
        )]
        identifier: Vec<String>,
        #[arg(long, help = "Optional journal narrative for the state change")]
        note: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Demographics {
    pub title: Option<String>,
    pub full_name: Option<String>,
    pub preferred_name: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<String>,
    pub nhs_number: Option<String>,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub value: String,
}

#[derive(Debug, Default)]
pub struct DemographicsUpdate {
    pub title: Option<String>,
    pub full_name: Option<String>,
    pub preferred_name: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<String>,
    pub nhs_number: Option<String>,
    pub identifiers: Vec<Identifier>,
    pub note: Option<String>,
}

pub fn run(command: DemographicsCommands) -> Result<()> {
    match command {
        DemographicsCommands::Show { json } => {
            typed_state::ensure_gitehr_repository()?;
            let demographics = load()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&demographics)?);
            } else {
                print_human(&demographics);
            }
            Ok(())
        }
        DemographicsCommands::Set {
            title,
            full_name,
            preferred_name,
            address,
            date_of_birth,
            nhs_number,
            identifier,
            note,
        } => {
            let identifiers = identifier
                .iter()
                .map(|raw| parse_identifier(raw))
                .collect::<Result<Vec<_>>>()?;
            update(DemographicsUpdate {
                title,
                full_name,
                preferred_name,
                address,
                date_of_birth,
                nhs_number,
                identifiers,
                note,
            })?;
            Ok(())
        }
    }
}

pub fn load() -> Result<Demographics> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn update(update: DemographicsUpdate) -> Result<Demographics> {
    typed_state::ensure_gitehr_repository()?;
    let note = update.note.clone();
    if update.title.is_none()
        && update.full_name.is_none()
        && update.preferred_name.is_none()
        && update.address.is_none()
        && update.date_of_birth.is_none()
        && update.nhs_number.is_none()
        && update.identifiers.is_empty()
    {
        anyhow::bail!("No demographic fields supplied");
    }

    if let Some(date) = update.date_of_birth.as_deref() {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("--date-of-birth must use YYYY-MM-DD format"))?;
    }

    let mut demographics = load()?;
    apply(update, &mut demographics);
    let path = typed_state::write_front_matter(STATE_FILE, &demographics)?;
    git::git_add(&path.to_string_lossy())?;
    let body = note.unwrap_or_else(|| {
        state_change_body("Updated demographics", demographics.full_name.as_deref())
    });
    journal::create_journal_entry(&body)?;
    println!("Updated demographics");
    Ok(demographics)
}

fn apply(update: DemographicsUpdate, demographics: &mut Demographics) {
    if let Some(value) = cleaned(update.title) {
        demographics.title = Some(value);
    }
    if let Some(value) = cleaned(update.full_name) {
        demographics.full_name = Some(value);
    }
    if let Some(value) = cleaned(update.preferred_name) {
        demographics.preferred_name = Some(value);
    }
    if let Some(value) = cleaned(update.address) {
        demographics.address = Some(value);
    }
    if let Some(value) = cleaned(update.date_of_birth) {
        demographics.date_of_birth = Some(value);
    }
    if let Some(value) = cleaned(update.nhs_number) {
        demographics.nhs_number = Some(value.clone());
        upsert_identifier(
            &mut demographics.identifiers,
            Identifier {
                id_type: "NHS".to_string(),
                value,
            },
        );
    }
    for identifier in update.identifiers {
        upsert_identifier(&mut demographics.identifiers, identifier);
    }
}

fn parse_identifier(raw: &str) -> Result<Identifier> {
    let (id_type, value) = raw
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Identifier must be in type:value form"))?;
    let id_type = id_type.trim();
    let value = value.trim();
    if id_type.is_empty() || value.is_empty() {
        anyhow::bail!("Identifier type and value must not be empty");
    }
    Ok(Identifier {
        id_type: id_type.to_string(),
        value: value.to_string(),
    })
}

fn upsert_identifier(identifiers: &mut Vec<Identifier>, identifier: Identifier) {
    if let Some(existing) = identifiers
        .iter_mut()
        .find(|existing| existing.id_type.eq_ignore_ascii_case(&identifier.id_type))
    {
        *existing = identifier;
    } else {
        identifiers.push(identifier);
    }
}

fn cleaned(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn print_human(demographics: &Demographics) {
    println!(
        "Name: {}",
        demographics.full_name.as_deref().unwrap_or("Not recorded")
    );
    println!(
        "Preferred name: {}",
        demographics
            .preferred_name
            .as_deref()
            .unwrap_or("Not recorded")
    );
    println!(
        "Date of birth: {}",
        demographics
            .date_of_birth
            .as_deref()
            .unwrap_or("Not recorded")
    );
    println!(
        "NHS number: {}",
        demographics.nhs_number.as_deref().unwrap_or("Not recorded")
    );
    for identifier in &demographics.identifiers {
        println!("Identifier: {}:{}", identifier.id_type, identifier.value);
    }
}

fn state_change_body(action: &str, subject: Option<&str>) -> String {
    match subject {
        Some(subject) => format!("{action}: {subject}"),
        None => action.to_string(),
    }
}
