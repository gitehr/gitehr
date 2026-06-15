use anyhow::Result;
use std::fs;
use std::path::Path;

use super::MpiInfo;

/// List all patient repositories in the store
pub fn run() -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("Not a GitEHR store root (gitehr-mpi.json not found)");
    }

    let mpi_content = fs::read_to_string("gitehr-mpi.json")?;
    let mpi: MpiInfo = serde_json::from_str(&mpi_content)?;

    if mpi.patients.is_empty() {
        println!("No patients registered in this store root.");
        println!("Add patient repositories in the patients/ directory and register them in gitehr-mpi.json");
        return Ok(());
    }

    println!("GitEHR Store Root");
    println!("MPI Version: {}", mpi.version);
    println!("Last Updated: {}", mpi.updated_at);
    println!();
    println!("Patients ({}):", mpi.patients.len());
    println!();

    for patient in &mpi.patients {
        println!("  Patient ID: {}", patient.patient_id);
        println!("  Repository: {}", patient.repo_path);
        println!("  Status:     {}", patient.status);
        if !patient.identifiers.is_empty() {
            println!("  Identifiers:");
            for id in &patient.identifiers {
                println!("    {}: {}", id.id_type, id.value);
            }
        }
        if let Some(merged) = &patient.merged_into {
            println!("  Merged into: {}", merged);
        }
        println!();
    }

    Ok(())
}
