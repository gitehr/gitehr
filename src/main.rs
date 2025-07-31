use anyhow::Result;
use clap::{Parser, Subcommand};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::initialise()?;
        }
        Commands::Add { content } => {
            // Get the latest entry's hash to use as parent
            let latest = commands::get_latest_journal_entry()?;
            let parent_hash = latest.map(|(_, hash)| hash);
            commands::create_journal_entry(&content, parent_hash)?;
        }
    }

    Ok(())
}
