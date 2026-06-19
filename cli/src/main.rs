// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod commands;
mod utils;

use commands::document::DocumentCommands;
use commands::journal::JournalCommands;
use commands::mcp::McpCommands;
use commands::remote::RemoteCommands;
#[cfg(feature = "server")]
use commands::server::ServerCommands;
use commands::state::StateCommands;
use commands::transport::TransportCommands;

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
        Commands::Init => commands::init::run()?,
        Commands::Journal { command } => commands::journal::run(command)?,
        Commands::State { command } => commands::state::run(command)?,
        Commands::Remote { command } => commands::remote::run(command)?,
        Commands::Encrypt { key } => commands::encrypt::run(key.as_deref())?,
        Commands::Decrypt { key } => commands::decrypt::run(key.as_deref())?,
        Commands::Status => commands::status::run()?,
        Commands::Transport { command } => commands::transport::run(command)?,
        Commands::Gui => commands::gui::run()?,
        #[cfg(feature = "server")]
        Commands::Server { command } => commands::server::run(command)?,
        Commands::Document { command } => commands::document::run(command)?,
        Commands::Mcp { command } => commands::mcp::run(command)?,
        Commands::Upgrade => commands::upgrade::run()?,
        Commands::UpgradeBinary => commands::upgrade_binary::run()?,
        Commands::Version => commands::version::run(),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            commands::completions::run(shell, &mut cmd);
        }
    }

    Ok(())
}
