// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod add;
pub mod init;
pub mod list;
pub mod remove;

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
}

pub fn run(command: StoreCommands) -> Result<()> {
    match command {
        StoreCommands::Init { name } => init::run(name.as_deref()),
        StoreCommands::Add { name, identifier } => {
            add::run(name.as_deref(), parse_identifiers(&identifier)?)
        }
        StoreCommands::Remove { subject } => remove::run(&subject),
        StoreCommands::List => list::run(),
    }
}

/// Parse `type:value` identifier strings (e.g. `NHS:1234567890`).
fn parse_identifiers(raw: &[String]) -> Result<Vec<(String, String)>> {
    raw.iter()
        .map(|s| {
            s.split_once(':')
                .map(|(t, v)| (t.to_string(), v.to_string()))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid identifier '{}'. Use type:value (e.g. NHS:1234567890)",
                        s
                    )
                })
        })
        .collect()
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub value: String,
}
