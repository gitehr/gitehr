// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod add_patient;
pub mod init;
pub mod list;
pub mod remove_patient;

#[derive(Subcommand)]
pub enum StoreCommands {
    #[command(about = "Initialize a new store root (multi-patient)")]
    Init,
    #[command(about = "List all patients in the store")]
    List,
    #[command(about = "Add a patient repository to the store")]
    AddPatient {
        #[arg(help = "Patient ID (UUID or other unique identifier)")]
        patient_id: String,
        #[arg(help = "Path to patient repository (e.g., patients/patient-uuid)")]
        repo_path: String,
        #[arg(
            long,
            help = "Identifiers in format type:value (e.g., NHS:1234567890). Can be specified multiple times."
        )]
        identifier: Vec<String>,
    },
    #[command(about = "Remove a patient repository from the store")]
    RemovePatient {
        #[arg(help = "Patient ID to remove")]
        patient_id: String,
    },
}

pub fn run(command: StoreCommands) -> Result<()> {
    match command {
        StoreCommands::Init => init::run(),
        StoreCommands::List => list::run(),
        StoreCommands::AddPatient {
            patient_id,
            repo_path,
            identifier,
        } => {
            // Parse identifier strings of the form "type:value"
            let identifiers: Result<Vec<(String, String)>> = identifier
                .iter()
                .map(|id_str| {
                    let parts: Vec<&str> = id_str.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        anyhow::bail!(
                            "Invalid identifier format '{}'. Use type:value (e.g., NHS:1234567890)",
                            id_str
                        );
                    }
                    Ok((parts[0].to_string(), parts[1].to_string()))
                })
                .collect();

            add_patient::run(patient_id, repo_path, identifiers?)
        }
        StoreCommands::RemovePatient { patient_id } => remove_patient::run(patient_id),
    }
}

// ── Shared data structures ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiInfo {
    pub version: u32,
    pub updated_at: String,
    pub patients: Vec<MpiPatient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiPatient {
    pub patient_id: String,
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
