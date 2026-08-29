// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub mod add;
pub mod init;
pub mod link;
pub mod list;
pub mod merge;
pub mod path;
pub mod remove;
pub mod search;
pub mod unlink;

#[derive(Subcommand)]
pub enum StoreCommands {
    /// Initialise a new Store: creates the Store, its MPI, and the first subject repo
    Init {
        #[arg(
            help = "Friendly name for the first subject (a person or pet). Omit for an auto-generated id."
        )]
        name: Option<String>,
    },
    /// Add a new subject: creates a repo and registers it in the MPI
    Add {
        #[arg(help = "Friendly name for the subject. Omit for an auto-generated id.")]
        name: Option<String>,
        #[arg(
            long,
            help = "Identifier as type:value (e.g. NHS:1234567890). Repeatable."
        )]
        identifier: Vec<String>,
    },
    /// Remove a subject from the MPI (does not delete files)
    Remove {
        #[arg(help = "Subject to remove, by canonical id or friendly name")]
        subject: String,
    },
    /// List the subjects in the Store
    List,
    /// Find subjects by identifier, id, or name (substring or exact type:value)
    Search {
        #[arg(help = "Identifier (type:value), canonical id, or name to search for")]
        query: String,
    },
    /// Link an identifier (type:value) to a subject
    Link {
        #[arg(help = "Subject to link the identifier to, by canonical id or friendly name")]
        subject: String,
        #[arg(help = "Identifier as type:value (e.g. NHS:1234567890)")]
        identifier: String,
    },
    /// Remove an identifier link (type:value) from whichever subject holds it
    Unlink {
        #[arg(help = "Identifier to remove, as type:value (e.g. NHS:1234567890)")]
        identifier: String,
    },
    /// Merge one subject into another, moving its identifiers
    Merge {
        #[arg(help = "Subject to merge away, by canonical id or friendly name")]
        from: String,
        #[arg(help = "Subject to merge into, by canonical id or friendly name")]
        into: String,
    },
    /// Print the repository path for a subject
    Path {
        #[arg(help = "Subject to locate, by canonical id or friendly name")]
        subject: String,
    },
}

pub fn run(command: StoreCommands) -> Result<()> {
    match command {
        StoreCommands::Init { name } => init::run(name.as_deref()),
        StoreCommands::Add { name, identifier } => {
            add::run(name.as_deref(), parse_identifiers(&identifier)?)
        }
        StoreCommands::Remove { subject } => remove::run(&subject),
        StoreCommands::List => list::run(),
        StoreCommands::Search { query } => search::run(&query),
        StoreCommands::Link {
            subject,
            identifier,
        } => {
            let (id_type, value) = parse_identifier(&identifier)?;
            link::run(&subject, id_type, &value)
        }
        StoreCommands::Unlink { identifier } => {
            let (id_type, value) = parse_identifier(&identifier)?;
            unlink::run(&id_type, &value)
        }
        StoreCommands::Merge { from, into } => merge::run(&from, &into),
        StoreCommands::Path { subject } => path::run(&subject),
    }
}

/// Parse a single `type:value` identifier string (e.g. `NHS:1234567890`).
fn parse_identifier(raw: &str) -> Result<(String, String)> {
    raw.split_once(':')
        .map(|(t, v)| (t.to_string(), v.to_string()))
        .ok_or_else(|| {
            anyhow::anyhow!("Invalid identifier '{raw}'. Use type:value (e.g. NHS:1234567890)")
        })
}

/// Parse `type:value` identifier strings (e.g. `NHS:1234567890`).
fn parse_identifiers(raw: &[String]) -> Result<Vec<(String, String)>> {
    raw.iter().map(|s| parse_identifier(s)).collect()
}

// ── MPI location, load, and save ─────────────────────────────────────────────

/// Locate the MPI file. `GITEHR_MPI_PATH` overrides the default of
/// `gitehr-mpi.json` in the current directory (which is the Store root).
pub(crate) fn mpi_path() -> Result<std::path::PathBuf> {
    match std::env::var("GITEHR_MPI_PATH") {
        Ok(p) if !p.trim().is_empty() => Ok(PathBuf::from(p)),
        Ok(_) => Ok(PathBuf::from("gitehr-mpi.json")),
        Err(std::env::VarError::NotPresent) => Ok(PathBuf::from("gitehr-mpi.json")),
        Err(e) => Err(anyhow::anyhow!("GITEHR_MPI_PATH is invalid: {e}")),
    }
}

pub(crate) fn load_mpi() -> Result<MpiInfo> {
    let path = mpi_path()?;
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Not a GitEHR Store root ({} not found)", path.display()))?;
    let mpi: MpiInfo = serde_json::from_str(&text)
        .with_context(|| format!("{} is not a valid MPI file", path.display()))?;
    Ok(mpi)
}

pub(crate) fn save_mpi(mpi: &MpiInfo) -> Result<()> {
    let path = mpi_path()?;
    fs::write(&path, serde_json::to_string_pretty(mpi)?)?;
    Ok(())
}

/// Resolve a subject by canonical id or friendly (repo) name.
pub(crate) fn find_subject<'a>(mpi: &'a MpiInfo, id_or_name: &str) -> Option<&'a MpiPatient> {
    mpi.patients
        .iter()
        .find(|p| p.patient_id == id_or_name || p.repo_path == id_or_name)
}

// ── Shared data structures (the MPI - gitehr-mpi.json at the Store root) ───────

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiInfo {
    pub version: u32,
    pub updated_at: String,
    pub patients: Vec<MpiPatient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiPatient {
    /// Canonical, stable id (a UUIDv7 in Crockford base32). Never changes.
    pub patient_id: String,
    /// On-disk directory for the subject's repo (friendly slug or the id).
    pub repo_path: String,
    pub status: String,
    pub merged_into: Option<String>,
    pub updated_at: String,
    pub identifiers: Vec<MpiIdentifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpiIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub value: String,
}
