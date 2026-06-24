pub mod completions;
pub mod contributor;
pub mod decrypt;
pub mod document;
pub mod encrypt;
mod git;
pub mod gui;
pub mod import;
pub mod init;
pub mod journal;
pub mod mcp;
pub mod remote;
#[cfg(feature = "server")]
pub mod server;
pub mod state;
pub mod status;
pub mod transport;
pub mod upgrade;
pub mod upgrade_binary;
pub mod version;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

use document::DocumentCommands;
use journal::JournalCommands;
use mcp::McpCommands;
use remote::RemoteCommands;
#[cfg(feature = "server")]
use server::ServerCommands;
use state::StateCommands;
use transport::TransportCommands;

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
    Import,
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
    Gui,
    #[cfg(feature = "server")]
    Server {
        #[command(subcommand)]
        command: ServerCommands,
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
        shell: clap_complete::Shell,
    },
}

pub fn run() -> Result<()> {
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
        Commands::Init => init::run()?,
        Commands::Import => import::run()?,
        Commands::Journal { command } => journal::run(command)?,
        Commands::State { command } => state::run(command)?,
        Commands::Remote { command } => remote::run(command)?,
        Commands::Encrypt { key } => encrypt::run(key.as_deref())?,
        Commands::Decrypt { key } => decrypt::run(key.as_deref())?,
        Commands::Status => status::run()?,
        Commands::Transport { command } => transport::run(command)?,
        Commands::Gui => gui::run()?,
        #[cfg(feature = "server")]
        Commands::Server { command } => server::run(command)?,
        Commands::Document { command } => document::run(command)?,
        Commands::Mcp { command } => mcp::run(command)?,
        Commands::Upgrade => upgrade::run()?,
        Commands::UpgradeBinary => upgrade_binary::run()?,
        Commands::Version => version::run(),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            completions::run(shell, &mut cmd);
        }
    }

    Ok(())
}
