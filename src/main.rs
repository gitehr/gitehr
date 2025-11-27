use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;

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
    /// Add a new clinical document
    Add {
        /// Content of the clinical entry
        #[arg(help = "The content to add to the journal")]
        content: String,
    },
    /// Journal-related commands
    Journal {
        #[command(subcommand)]
        command: JournalCommands,
    },
}

#[derive(Subcommand)]
enum JournalCommands {
    /// Verify the integrity of the journal chain
    Verify,
}

fn is_gitehr_repository() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no subcommand was provided, print the version and exit successfully
    if std::env::args().len() == 1 {
        let cmd = Cli::command();
        // clap already defines the version via Cargo.toml
        println!("{}", cmd.get_version().unwrap_or_default());
        return Ok(());
    }

    match cli.command {
        Commands::Init => {
            commands::initialise()?;
        }
        Commands::Add { content } => {
            // Check if we're in a GitEHR repository
            if !is_gitehr_repository() {
                anyhow::bail!(
                    "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
                );
            }

            // Get the latest entry's hash to use as parent
            let latest = commands::get_latest_journal_entry()?;
            let parent_hash = latest.map(|(_, hash)| hash);
            commands::create_journal_entry(&content, parent_hash)?;
        }
        Commands::Journal { command } => {
            // Check if we're in a GitEHR repository
            if !is_gitehr_repository() {
                anyhow::bail!(
                    "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
                );
            }

            match command {
                JournalCommands::Verify => {
                    commands::verify::verify_journal()?;
                }
            }
        }
    }

    Ok(())
}
