// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

mod commands;
mod utils;

use commands::document::DocumentCommands;
use commands::journal::JournalCommands;
use commands::mcp::McpCommands;
use commands::remote::RemoteCommands;
use commands::state::StateCommands;
use commands::store::StoreCommands;
use commands::transport::TransportCommands;
use commands::user::UserCommands;

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
    /// Manage contributors and the active author
    User {
        #[command(subcommand)]
        command: Option<UserCommands>,
    },
    /// Import journal entries or documents from a file or directory
    Import {
        #[arg(long, value_enum, help = "What kind of data to import")]
        mode: commands::import::ImportMode,
        #[arg(help = "File or directory to import")]
        path: std::path::PathBuf,
    },
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
    /// Manage a multi-patient store and its Main Patient Index (MPI)
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
    Gui,
    #[command(alias = "attach")]
    Document {
        #[command(subcommand)]
        command: DocumentCommands,
    },
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Clinical calculators (scores, screeners, risk tools)
    Calc(calc_cli::CalcCommand),
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
    /// List installed plugins (gitehr-<command> executables on PATH)
    Plugins,
    /// Run an installed `gitehr-<command>` plugin from PATH. Any subcommand
    /// that is not built in is dispatched here; built-ins always take priority.
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() -> Result<()> {
    // Built-in names (and their aliases) always shadow a same-named plugin.
    let builtins = commands::plugin::builtin_names(&Cli::command());

    // Build the command, injecting any discovered plugins into `--help`.
    let mut cmd = Cli::command();
    if let Some(section) = commands::plugin::plugins_help_section(&builtins) {
        cmd = cmd.after_help(section);
    }

    if std::env::args().len() == 1 {
        let version = cmd.get_version().unwrap_or_default();
        println!("GitEHR {}", version);
        println!();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    let cli = match Cli::from_arg_matches(&cmd.get_matches()) {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    match cli.command {
        Commands::Init => commands::init::run()?,
        Commands::User { command } => commands::user::run(command)?,
        Commands::Import { mode, path } => commands::import::run(mode, &path)?,
        Commands::Journal { command } => commands::journal::run(command)?,
        Commands::State { command } => commands::state::run(command)?,
        Commands::Remote { command } => commands::remote::run(command)?,
        Commands::Encrypt { key } => commands::encrypt::run(key.as_deref())?,
        Commands::Decrypt { key } => commands::decrypt::run(key.as_deref())?,
        Commands::Status => commands::status::run()?,
        Commands::Transport { command } => commands::transport::run(command)?,
        Commands::Store { command } => commands::store::run(command)?,
        Commands::Gui => commands::gui::run()?,
        Commands::Document { command } => commands::document::run(command)?,
        Commands::Mcp { command } => commands::mcp::run(command)?,
        Commands::Calc(command) => calc_cli::run(command)?,
        Commands::Upgrade => commands::upgrade::run()?,
        Commands::UpgradeBinary => commands::upgrade_binary::run()?,
        Commands::Version => commands::version::run(),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            commands::completions::run(shell, &mut cmd);
        }
        Commands::Plugins => commands::plugin::list(&builtins)?,
        Commands::External(args) => commands::plugin::run(args)?,
    }

    Ok(())
}
