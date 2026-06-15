use anyhow::Result;
use std::fs;
use std::path::Path;

use super::{MpiIdentifier, MpiPatient};

/// Add a patient repository to the store
pub fn run(
    patient_id: String,
    repo_path: String,
    identifiers: Vec<(String, String)>,
) -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("Not a GitEHR store root (gitehr-mpi.json not found)");
    }

    // Read existing MPI
    let mpi_content = fs::read_to_string("gitehr-mpi.json")?;
    let mut mpi: super::MpiInfo = serde_json::from_str(&mpi_content)?;

    // Check if patient already exists
    if mpi.patients.iter().any(|p| p.patient_id == patient_id) {
        anyhow::bail!("Patient {} already exists in the MPI", patient_id);
    }

    // Verify the repository path exists and is a gitehr repo
    let full_repo_path = Path::new(&repo_path);
    if !full_repo_path.exists() {
        anyhow::bail!("Repository path does not exist: {}", repo_path);
    }

    let gitehr_path = full_repo_path.join(".gitehr");
    if !gitehr_path.exists() {
        anyhow::bail!(
            "Path is not a GitEHR repository (no .gitehr directory): {}",
            repo_path
        );
    }

    // Create patient entry
    let patient = MpiPatient {
        patient_id: patient_id.clone(),
        repo_path: repo_path.clone(),
        status: "active".to_string(),
        merged_into: None,
        updated_at: chrono::Utc::now().to_rfc3339(),
        identifiers: identifiers
            .into_iter()
            .map(|(id_type, value)| MpiIdentifier { id_type, value })
            .collect(),
    };

    // Add to MPI
    mpi.patients.push(patient);
    mpi.updated_at = chrono::Utc::now().to_rfc3339();

    // Write updated MPI
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    println!("✓ Added patient {} to MPI", patient_id);
    println!("  Repository: {}", repo_path);

    Ok(())
}
