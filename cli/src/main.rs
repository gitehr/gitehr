// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;
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
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
    #[command(alias = "attach")]
    Document {
        #[command(subcommand)]
        command: DocumentCommands,
    },
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    Upgrade,
    #[command(
        name = "upgrade-binary",
        about = "Update the bundled binary to the current CLI version"
    )]
    UpgradeBinary,
    #[command(visible_alias = "v")]
    Version,
    #[command(
        about = "Generate shell completions",
        long_about = r#"Generate shell completions for gitehr.

Examples:
  gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr
  gitehr completions zsh > "${fpath[1]}/_gitehr"
  gitehr completions fish > ~/.config/fish/completions/gitehr.fish
  gitehr completions powershell | Out-File -Append $PROFILE

Restart your shell after installing completions."#
    )]
    Completions {
        #[arg(help = "Shell type (bash, zsh, fish, powershell)")]
        shell: Shell,
    },
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
    #[command(
        about = "Play back the full journal, oldest first",
        long_about = "Print the full body of every journal entry in chronological order. \
                      Use this to read the patient's record end to end."
    )]
    Cat {
        #[arg(short, long, help = "Show newest entries first")]
        reverse: bool,
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
enum StoreCommands {
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
        #[arg(long, help = "Identifiers in format type:value (e.g., NHS:1234567890). Can be specified multiple times.")]
        identifier: Vec<String>,
    },
    #[command(about = "Remove a patient repository from the store")]
    RemovePatient {
        #[arg(help = "Patient ID to remove")]
        patient_id: String,
    },
}

#[derive(Subcommand)]
enum UserCommands {
    #[command(about = "Create a user interactively")]
    Create,
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

#[derive(Subcommand)]
enum DocumentCommands {
    #[command(about = "Add a Document to the record and link it from a new journal entry")]
    Add {
        #[arg(help = "Path to the file (or directory, e.g. a DICOM study) to add")]
        path: PathBuf,
        #[arg(long, help = "Store under imaging/ instead of documents/")]
        imaging: bool,
        #[arg(
            short,
            long,
            help = "Title used to build the stored filename slug (defaults to the original filename)"
        )]
        title: Option<String>,
        #[arg(short, long, help = "Journal entry text describing the Document")]
        message: Option<String>,
    },
    #[command(about = "List Documents referenced by journal entries")]
    List,
    #[command(about = "Show which journal entries reference a Document")]
    Info {
        #[arg(help = "Path of the Document within the record (e.g. documents/2026-06-12-...pdf)")]
        path: String,
    },
    #[command(about = "Verify Document integrity against the hashes recorded in journal entries")]
    Verify {
        #[arg(help = "Verify a single Document path (default: all)")]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    #[command(about = "Start MCP server")]
    Serve {
        #[arg(long, help = "Use stdio transport (default)")]
        stdio: bool,
        #[arg(long, help = "Repository path (default: current directory)")]
        repo_path: Option<PathBuf>,
    },
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
                JournalCommands::Cat { reverse } => {
                    commands::journal::cat_journal_entries(reverse)?;
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
            Some(UserCommands::Create) => {
                commands::contributor::create_user_interactive()?;
            }
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
                    None,
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
        Commands::Store { command } => match command {
            StoreCommands::Init => {
                commands::store::init()?;
            }
            StoreCommands::List => {
                commands::store::list()?;
            }
            StoreCommands::AddPatient { patient_id, repo_path, identifier } => {
                // Parse identifiers from type:value format
                let identifiers: Result<Vec<(String, String)>> = identifier
                    .iter()
                    .map(|id_str| {
                        let parts: Vec<&str> = id_str.splitn(2, ':').collect();
                        if parts.len() != 2 {
                            anyhow::bail!("Invalid identifier format '{}'. Use type:value (e.g., NHS:1234567890)", id_str);
                        }
                        Ok((parts[0].to_string(), parts[1].to_string()))
                    })
                    .collect();
                
                commands::store::add_patient(patient_id.clone(), repo_path.clone(), identifiers?)?;
            }
            StoreCommands::RemovePatient { patient_id } => {
                commands::store::remove_patient(patient_id.clone())?;
            }
        },
        Commands::Document { command } => match command {
            DocumentCommands::Add {
                path,
                imaging,
                title,
                message,
            } => {
                commands::document::add_document(
                    path.as_path(),
                    title.as_deref(),
                    imaging,
                    message.as_deref(),
                )?;
            }
            DocumentCommands::List => {
                commands::document::list_documents()?;
            }
            DocumentCommands::Info { path } => {
                commands::document::document_info(&path)?;
            }
            DocumentCommands::Verify { path } => {
                let ok = commands::document::verify_documents(path.as_deref())?;
                if !ok {
                    std::process::exit(1);
                }
            }
        },
        Commands::Mcp { command } => match command {
            McpCommands::Serve { stdio: _, repo_path } => {
                commands::mcp::serve_mcp_stdio(repo_path.clone())?;
            }
        },
        Commands::Upgrade => {
            commands::upgrade::upgrade_repository()?;
        }
        Commands::UpgradeBinary => {
            commands::upgrade::upgrade_binary()?;
        }
        Commands::Version => {
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
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "gitehr", &mut io::stdout());
        }
    }

    Ok(())
}
