use anyhow::Result;
use clap::{Parser, Subcommand};
use fs_extra::dir::{self, CopyOptions};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gitehr")]
#[command(about = "A Git-based Electronic Health Record system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new GitEHR repository
    Init,
}

fn init_repository() -> Result<()> {
    // Check if .gitehr directory already exists
    let gitehr_dir = PathBuf::from(".gitehr");
    if gitehr_dir.exists() {
        anyhow::bail!("GitEHR repository already exists in this directory");
    }

    // Get the path to the folder structure template
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gitehr-folder-structure");

    if !template_path.exists() {
        anyhow::bail!("Could not find template directory");
    }

    // Create initial directories
    std::fs::create_dir(".gitehr")?;

    // Read and copy contents from template directory
    for entry in std::fs::read_dir(&template_path)? {
        let entry = entry?;
        let target_name = entry.file_name();

        if entry.file_type()?.is_dir() {
            let dir_options = CopyOptions::new();
            dir::copy(entry.path(), ".", &dir_options)?;
        } else {
            let file_options = fs_extra::file::CopyOptions::new();
            fs_extra::file::copy(entry.path(), target_name, &file_options)?;
        }
    }

    println!("Initialized empty GitEHR repository");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_repository()?,
    }

    Ok(())
}
