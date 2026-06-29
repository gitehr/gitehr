// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::Utc;
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{contributor, git, journal, typed_state};

const STATE_FILE: &str = "allergies.md";

#[derive(Subcommand)]
pub enum AllergyCommands {
    #[command(about = "List current allergies")]
    List {
        #[arg(long, help = "Emit JSON for GUI or automation callers")]
        json: bool,
        #[arg(long, help = "Include inactive allergies")]
        all: bool,
    },
    #[command(about = "Add an active allergy or adverse reaction")]
    Add {
        #[arg(long)]
        agent: String,
        #[arg(long)]
        reaction: String,
        #[arg(long, value_enum, default_value_t = AllergySeverity::Moderate)]
        severity: AllergySeverity,
        #[arg(long)]
        note: Option<String>,
    },
    #[command(about = "Mark an allergy inactive")]
    Inactive {
        #[arg(help = "Allergy id")]
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AllergySeverity {
    Low,
    Moderate,
    High,
    Critical,
}

impl std::fmt::Display for AllergySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AllergySeverity::Low => "low",
            AllergySeverity::Moderate => "moderate",
            AllergySeverity::High => "high",
            AllergySeverity::Critical => "critical",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllergyStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allergy {
    pub id: String,
    pub agent: String,
    pub reaction: String,
    pub severity: AllergySeverity,
    pub status: AllergyStatus,
    pub recorded_at: String,
    pub recorded_by: Option<String>,
    pub inactive_at: Option<String>,
    pub inactive_reason: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllergiesState {
    #[serde(default)]
    pub allergies: Vec<Allergy>,
}

pub fn run(command: AllergyCommands) -> Result<()> {
    match command {
        AllergyCommands::List { json, all } => {
            typed_state::ensure_gitehr_repository()?;
            let allergies = list(all)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&allergies)?);
            } else {
                print_human(&allergies);
            }
            Ok(())
        }
        AllergyCommands::Add {
            agent,
            reaction,
            severity,
            note,
        } => {
            add(&agent, &reaction, severity, note.as_deref())?;
            Ok(())
        }
        AllergyCommands::Inactive { id, reason } => {
            inactive(&id, reason.as_deref())?;
            Ok(())
        }
    }
}

pub fn load() -> Result<AllergiesState> {
    typed_state::read_front_matter(STATE_FILE)
}

pub fn list(include_inactive: bool) -> Result<Vec<Allergy>> {
    let state = load()?;
    Ok(state
        .allergies
        .into_iter()
        .filter(|allergy| include_inactive || allergy.status == AllergyStatus::Active)
        .collect())
}

pub fn add(
    agent: &str,
    reaction: &str,
    severity: AllergySeverity,
    note: Option<&str>,
) -> Result<Allergy> {
    typed_state::ensure_gitehr_repository()?;
    let agent = require_text(agent, "--agent")?;
    let reaction = require_text(reaction, "--reaction")?;
    let now = Utc::now();
    let allergy = Allergy {
        id: format!(
            "ALG-{}-{}",
            now.format("%Y%m%dT%H%M%SZ"),
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        ),
        agent: agent.to_string(),
        reaction: reaction.to_string(),
        severity,
        status: AllergyStatus::Active,
        recorded_at: now.to_rfc3339(),
        recorded_by: contributor::get_current_contributor(),
        inactive_at: None,
        inactive_reason: None,
        note: note.and_then(cleaned_str),
    };

    let mut state = load()?;
    state.allergies.push(allergy.clone());
    persist_with_journal(
        &state,
        note.unwrap_or(&format!(
            "Added allergy: {} - {} ({})",
            allergy.agent, allergy.reaction, allergy.severity
        )),
    )?;
    println!("Added allergy: {}", allergy.id);
    Ok(allergy)
}

pub fn inactive(id: &str, reason: Option<&str>) -> Result<Allergy> {
    typed_state::ensure_gitehr_repository()?;
    let mut state = load()?;
    let allergy = state
        .allergies
        .iter_mut()
        .find(|allergy| allergy.id == id)
        .ok_or_else(|| anyhow::anyhow!("Allergy not found: {}", id))?;

    allergy.status = AllergyStatus::Inactive;
    allergy.inactive_at = Some(Utc::now().to_rfc3339());
    allergy.inactive_reason = reason.and_then(cleaned_str);
    let changed = allergy.clone();

    persist_with_journal(
        &state,
        reason.unwrap_or(&format!("Marked allergy inactive: {}", changed.agent)),
    )?;
    println!("Marked allergy inactive: {}", changed.id);
    Ok(changed)
}

fn persist_with_journal(state: &AllergiesState, journal_body: &str) -> Result<()> {
    let path = typed_state::write_front_matter(STATE_FILE, state)?;
    git::git_add(&path.to_string_lossy())?;
    journal::create_journal_entry(journal_body)?;
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

fn print_human(allergies: &[Allergy]) {
    if allergies.is_empty() {
        println!("No active allergies recorded.");
        return;
    }

    for allergy in allergies {
        println!(
            "{}  {} - {} ({})",
            allergy.id, allergy.agent, allergy.reaction, allergy.severity
        );
    }
}
