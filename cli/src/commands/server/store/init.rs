use anyhow::Result;
use std::fs;
use std::path::Path;

use super::MpiInfo;

/// Initialize a new GitEHR store root (multi-patient store)
pub fn run() -> Result<()> {
    // Check if we're already in a store root
    if Path::new("gitehr-mpi.json").exists() {
        anyhow::bail!("This directory is already a GitEHR store root (gitehr-mpi.json exists)");
    }

    // Check if we're in a patient repository
    if Path::new(".gitehr").exists() {
        anyhow::bail!(
            "This directory is already a GitEHR patient repository. Store roots should be created in a separate location."
        );
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
├── gitehr-mpi.json          # Main Patient Index
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

## Main Patient Index (MPI)

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
