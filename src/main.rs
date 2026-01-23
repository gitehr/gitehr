use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "gitehr")]
#[command(about = "The Git-based Electronic Health Record", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new GitEHR repository
    Init,
    /// Journal-related commands
    Journal {
        #[command(subcommand)]
        command: JournalCommands,
    },
    /// Manage mutable clinical state
    State {
        /// Additional state command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Manage remote GitEHR repositories
    Remote {
        /// Additional remote command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Encrypt a repository or file
    Encrypt {
        /// Additional encrypt command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Decrypt a repository or file
    Decrypt {
        /// Additional decrypt command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Display repository status
    Status {
        /// Additional status command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Convert repository for transport
    Transport {
        /// Additional transport command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Manage contributors
    Contributor {
        /// Additional contributor command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Open the GitEHR GUI
    Gui,
    /// Upgrade the GitEHR repository
    Upgrade {
        /// Additional upgrade command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Print the GitEHR CLI and GUI versions
    #[command(visible_alias = "v")]
    Version,
}

#[derive(Subcommand)]
enum JournalCommands {
    /// Add a new clinical document
    Add {
        /// Content of the clinical entry
        #[arg(help = "The content to add to the journal")]
        content: String,
    },
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
        let mut cmd = Cli::command();
        // clap already defines the version via Cargo.toml
        let version = cmd.get_version().unwrap_or_default();
        println!("GitEHR {}", version);
        println!();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    match cli.command {
        Commands::Init => {
            commands::initialise()?;
        }
        Commands::Journal { command } => {
            // Check if we're in a GitEHR repository
            if !is_gitehr_repository() {
                anyhow::bail!(
                    "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
                );
            }

            match command {
                JournalCommands::Add { content } => {
                    // Get the latest entry's hash to use as parent
                    let latest = commands::get_latest_journal_entry()?;
                    let parent_hash = latest.map(|(_, hash)| hash);
                    commands::create_journal_entry(&content, parent_hash)?;
                }
                JournalCommands::Verify => {
                    commands::verify::verify_journal()?;
                }
            }
        }
        Commands::State { args } => {
            println!(
                "The 'state' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Remote { args } => {
            println!(
                "The 'remote' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Encrypt { args } => {
            println!(
                "The 'encrypt' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Decrypt { args } => {
            println!(
                "The 'decrypt' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Status { args } => {
            println!(
                "The 'status' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Transport { args } => {
            println!(
                "The 'transport' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Contributor { args } => {
            println!(
                "The 'contributor' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Gui => {
            println!("The 'gui' command is not implemented yet.");
        }
        Commands::Upgrade { args } => {
            println!(
                "The 'upgrade' command is not implemented yet. Args: {}",
                args.join(" ")
            );
        }
        Commands::Version => {
            let cmd = Cli::command();
            let version = cmd.get_version().unwrap_or_default();
            println!("GitEHR {}", version);
        }
    }

    Ok(())
}
