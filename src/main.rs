// SPDX-License-Identifier: AGPL-3.0-or-later

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
    Init,
    Journal {
        #[command(subcommand)]
        command: JournalCommands,
    },
    State {
        #[command(subcommand)]
        command: Option<StateCommands>,
    },
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommands>,
    },
    Encrypt {
        #[arg(long, help = "Key source (local or remote URL)")]
        key: Option<String>,
    },
    Decrypt {
        #[arg(long, help = "Key source (local or remote URL)")]
        key: Option<String>,
    },
    #[command(visible_alias = "st")]
    Status,
    Transport {
        #[command(subcommand)]
        command: Option<TransportCommands>,
    },
    #[command(visible_alias = "contributor")]
    User {
        #[command(subcommand)]
        command: Option<UserCommands>,
    },
    Gui,
    Upgrade,
    #[command(
        name = "upgrade-binary",
        about = "Update the bundled binary to the current CLI version"
    )]
    UpgradeBinary,
    #[command(visible_alias = "v")]
    Version,
    #[command(hide = true)]
    V,
}

#[derive(Subcommand)]
enum JournalCommands {
    Add {
        #[arg(help = "The content to add to the journal (use --file for file input)")]
        content: Option<String>,
        #[arg(short, long, help = "Read content from a file (use - for stdin)")]
        file: Option<String>,
    },
    Show {
        #[arg(
            short = 'n',
            long,
            default_value = "10",
            help = "Maximum number of entries to display"
        )]
        limit: usize,
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Number of entries to skip from the start"
        )]
        offset: usize,
        #[arg(short, long, help = "Show newest entries first")]
        reverse: bool,
        #[arg(short, long, help = "Show all entries (ignores --limit)")]
        all: bool,
    },
    Verify,
}

#[derive(Subcommand)]
enum StateCommands {
    List,
    Get {
        #[arg(help = "Name of the state file")]
        filename: String,
    },
    Set {
        #[arg(help = "Name of the state file")]
        filename: String,
        #[arg(help = "Content to write")]
        content: String,
    },
}

#[derive(Subcommand)]
enum RemoteCommands {
    Add {
        #[arg(help = "Name for the remote")]
        name: String,
        #[arg(help = "URL of the remote")]
        url: String,
    },
    #[command(visible_alias = "rm")]
    Remove {
        #[arg(help = "Name of the remote to remove")]
        name: String,
    },
    List,
}

#[derive(Subcommand)]
enum TransportCommands {
    Create {
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
        #[arg(long, help = "Apply additional encryption")]
        encrypt: bool,
    },
    Extract {
        #[arg(help = "Path to the transport archive")]
        archive: String,
        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum UserCommands {
    Add {
        #[arg(help = "Unique identifier for the user")]
        id: String,
        #[arg(help = "Display name")]
        name: String,
        #[arg(long, help = "Role or title")]
        role: Option<String>,
        #[arg(long, help = "Email address")]
        email: Option<String>,
    },
    Enable {
        #[arg(help = "User ID")]
        id: String,
    },
    Disable {
        #[arg(help = "User ID")]
        id: String,
    },
    Activate {
        #[arg(help = "User ID to set as current author")]
        id: String,
    },
    Deactivate,
    List,
}

fn is_gitehr_repository() -> bool {
    PathBuf::from(".gitehr").exists()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if std::env::args().len() == 1 {
        let mut cmd = Cli::command();
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
            if !is_gitehr_repository() {
                anyhow::bail!(
                    "Not a GitEHR repository (or not in the repository root). Run 'gitehr init' to create a new repository."
                );
            }

            match command {
                JournalCommands::Add { content, file } => {
                    let entry_content = match (content, file) {
                        (Some(text), None) => text,
                        (None, Some(path)) => {
                            if path == "-" {
                                use std::io::Read;
                                let mut buffer = String::new();
                                std::io::stdin().read_to_string(&mut buffer)?;
                                buffer
                            } else {
                                std::fs::read_to_string(&path).map_err(|e| {
                                    anyhow::anyhow!("Failed to read file '{}': {}", path, e)
                                })?
                            }
                        }
                        (Some(_), Some(_)) => {
                            anyhow::bail!("Cannot specify both content and --file");
                        }
                        (None, None) => {
                            anyhow::bail!("Must provide content or use --file <path>");
                        }
                    };

                    let latest = commands::get_latest_journal_entry()?;
                    let parent_hash = latest.map(|(_, hash)| hash);
                    commands::create_journal_entry(&entry_content, parent_hash)?;
                }
                JournalCommands::Show {
                    limit,
                    offset,
                    reverse,
                    all,
                } => {
                    commands::journal::show_journal_entries(limit, offset, reverse, all)?;
                }
                JournalCommands::Verify => {
                    commands::verify::verify_journal()?;
                }
            }
        }
        Commands::State { command } => match command {
            Some(StateCommands::List) => {
                commands::state::run_state_list()?;
            }
            Some(StateCommands::Get { filename }) => {
                commands::state::run_state_get(&filename)?;
            }
            Some(StateCommands::Set { filename, content }) => {
                commands::state::run_state_set(&filename, &content)?;
            }
            None => {
                commands::state::run_state_list()?;
            }
        },
        Commands::Remote { command } => match command {
            Some(RemoteCommands::Add { name, url }) => {
                commands::remote::add_remote(&name, &url)?;
            }
            Some(RemoteCommands::Remove { name }) => {
                commands::remote::remove_remote(&name)?;
            }
            Some(RemoteCommands::List) | None => {
                commands::remote::list_remotes()?;
            }
        },
        Commands::Encrypt { key } => {
            commands::encrypt::encrypt_repository(key.as_deref())?;
        }
        Commands::Decrypt { key } => {
            commands::decrypt::decrypt_repository(key.as_deref())?;
        }
        Commands::Status => {
            commands::status::run_status()?;
        }
        Commands::Transport { command } => match command {
            Some(TransportCommands::Create { output, encrypt }) => {
                commands::transport::create_transport_archive(output.as_deref(), encrypt)?;
            }
            Some(TransportCommands::Extract { archive, output }) => {
                commands::transport::extract_transport_archive(&archive, output.as_deref())?;
            }
            None => {
                println!("Usage: gitehr transport <create|extract>");
                println!();
                println!("Subcommands:");
                println!("  create    Create a transport archive from this repository");
                println!("  extract   Extract a transport archive");
            }
        },
        Commands::User { command } => match command {
            Some(UserCommands::Add {
                id,
                name,
                role,
                email,
            }) => {
                commands::contributor::add_contributor(
                    &id,
                    &name,
                    role.as_deref(),
                    email.as_deref(),
                )?;
            }
            Some(UserCommands::Enable { id }) => {
                commands::contributor::enable_contributor(&id)?;
            }
            Some(UserCommands::Disable { id }) => {
                commands::contributor::disable_contributor(&id)?;
            }
            Some(UserCommands::Activate { id }) => {
                commands::contributor::activate_contributor(&id)?;
            }
            Some(UserCommands::Deactivate) => {
                commands::contributor::deactivate_contributor()?;
            }
            Some(UserCommands::List) | None => {
                commands::contributor::list_contributors()?;
            }
        },
        Commands::Gui => {
            commands::gui::launch_gui()?;
        }
        Commands::Upgrade => {
            commands::upgrade::upgrade_repository()?;
        }
        Commands::UpgradeBinary => {
            commands::upgrade::upgrade_binary()?;
        }
        Commands::Version | Commands::V => {
            // Use the crate version from Cargo.toml
            let gitehr_version = env!("CARGO_PKG_VERSION");
            println!("GitEHR: {}", gitehr_version);
            if let Some(git_version) = commands::get_git_version() {
                // Format: Git <version>
                let git_version_str = git_version.replace("git version ", "");
                println!("Git {}", git_version_str);
            } else {
                println!("Git (not found or not installed)");
            }
        }
    }

    Ok(())
}
