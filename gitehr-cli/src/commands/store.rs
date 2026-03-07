use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct MpiInfo {
    version: u32,
    updated_at: String,
    patients: Vec<MpiPatient>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpiPatient {
    patient_id: String,
    repo_path: String,
    status: String,
    merged_into: Option<String>,
    updated_at: String,
    identifiers: Vec<MpiIdentifier>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpiIdentifier {
    #[serde(rename = "type")]
    id_type: String,
    value: String,
}

/// Initialize a new GitEHR store root (multi-patient store)
pub fn init() -> Result<()> {
    // Check if we're already in a store root
    if Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("This directory is already a GitEHR store root (gitehr-mpi.json exists)");
    }

    // Check if we're in a patient repository
    if Path::new(".gitehr").exists() {
        anyhow::bail!("This directory is already a GitEHR patient repository. Store roots should be created in a separate location.");
    }

    // Create empty MPI
    let mpi = MpiInfo {
        version: 1,
        updated_at: chrono::Utc::now().to_rfc3339(),
        patients: vec![],
    };

    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    // Create patients directory
    fs::create_dir_all("patients")?;

    // Create README
    let readme = r#"# GitEHR Store Root

This directory is a GitEHR store root containing multiple patient repositories.

## Structure

```
.
├── gitehr-mpi.json          # Master Patient Index
├── patients/                # Individual patient repositories
│   └── <patient-uuid>/      # Each patient has their own Git repository
└── README.md                # This file
```

## Usage

### Add a new patient repository

```bash
cd patients
mkdir <patient-uuid>
cd <patient-uuid>
gitehr init
```

Then register the patient in the MPI by editing `gitehr-mpi.json`.

### Open in GUI

```bash
gitehr gui
# Or: Open the store root folder in the GitEHR GUI
```

The GUI will detect the MPI and allow you to select and manage individual patient repositories.

## Master Patient Index (MPI)

The `gitehr-mpi.json` file contains:
- List of all patient repositories in this store
- Patient identifiers (NHS number, local ID, etc.)
- Repository status (active, merged, etc.)
- Cross-references between merged repositories

This enables:
- Multi-patient management
- Patient identity management
- Repository merging/linking
- Practice/organization-wide views

## Security Note

Each patient repository is a separate Git repository with its own encryption.
The MPI itself does not contain clinical data, only metadata and paths.
"#;

    fs::write("README.md", readme)?;

    println!("✓ Initialized GitEHR store root");
    println!("  Created: gitehr-mpi.json (empty MPI)");
    println!("  Created: patients/ directory");
    println!("  Created: README.md");
    println!();
    println!("Next steps:");
    println!("  1. Create patient repositories in patients/ directory");
    println!("  2. Register patients in gitehr-mpi.json");
    println!("  3. Open this directory in the GitEHR GUI");

    Ok(())
}

/// List all patient repositories in the store
pub fn list() -> Result<()> {
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

/// Add a patient repository to the store
pub fn add_patient(
    patient_id: String,
    repo_path: String,
    identifiers: Vec<(String, String)>,
) -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("Not a GitEHR store root (gitehr-mpi.json not found)");
    }

    // Read existing MPI
    let mpi_content = fs::read_to_string("gitehr-mpi.json")?;
    let mut mpi: MpiInfo = serde_json::from_str(&mpi_content)?;

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

/// Remove a patient repository from the store (does not delete files)
pub fn remove_patient(patient_id: String) -> Result<()> {
    if !Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("Not a GitEHR store root (gitehr-mpi.json not found)");
    }

    // Read existing MPI
    let mpi_content = fs::read_to_string("gitehr-mpi.json")?;
    let mut mpi: MpiInfo = serde_json::from_str(&mpi_content)?;

    // Find and remove patient
    let initial_count = mpi.patients.len();
    mpi.patients.retain(|p| p.patient_id != patient_id);

    if mpi.patients.len() == initial_count {
        anyhow::bail!("Patient {} not found in MPI", patient_id);
    }

    mpi.updated_at = chrono::Utc::now().to_rfc3339();

    // Write updated MPI
    fs::write("gitehr-mpi.json", serde_json::to_string_pretty(&mpi)?)?;

    println!("✓ Removed patient {} from MPI", patient_id);
    println!("  Note: Patient repository files were not deleted");

    Ok(())
}

/// Check if current directory is a store root
pub fn is_store_root() -> Result<bool> {
    Ok(Path::new("gitehr-mpi.json").exists())
}
