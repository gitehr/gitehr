// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::path::{Path, PathBuf};

mod commands;
mod config;
mod utils;

use commands::config::ConfigCommands;
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
    /// Manage contributors and the active author
    #[command(visible_alias = "contributor")]
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
    /// Manage local GitEHR configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
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
    // Clinical calculators are temporarily dormant while pacharanero/calc is
    // pre-crates.io; release-plz package verification cannot package git-only
    // dependencies. Restore `Calc(calc_cli::CalcCommand)` once calc-cli is
    // published.
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

    let mut cli = match Cli::from_arg_matches(&cmd.get_matches()) {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    // Resolve the Store/repo working context before dispatch (ADR-0005).
    apply_context(&mut cli.command)?;

    match cli.command {
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
        Commands::Config { command } => commands::config::run(command)?,
        Commands::Gui => commands::gui::run()?,
        Commands::Document { command } => commands::document::run(command)?,
        Commands::Mcp { command } => commands::mcp::run(command)?,
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

/// Make external (user-cwd-relative) path arguments absolute before context
/// resolution changes the working directory. Repo-relative paths (journal
/// drafts, Document paths to verify) are intentionally left alone.
fn absolutize_external_paths(command: &mut Commands, base: &Path) {
    fn fix_pb(p: &mut PathBuf, base: &Path) {
        if p.is_relative() {
            *p = base.join(&*p);
        }
    }
    fn fix_str(s: &mut String, base: &Path) {
        if Path::new(s.as_str()).is_relative() {
            *s = base.join(&*s).to_string_lossy().into_owned();
        }
    }
    match command {
        Commands::Import { path, .. } => fix_pb(path, base),
        Commands::Journal {
            command: JournalCommands::Add { file: Some(f), .. },
        } if f != "-" => fix_str(f, base),
        Commands::Document {
            command: DocumentCommands::Add { path, .. },
        } => fix_pb(path, base),
        Commands::Transport {
            command: Some(transport),
        } => match transport {
            TransportCommands::Create { output, .. } => {
                if let Some(o) = output {
                    fix_str(o, base);
                }
            }
            TransportCommands::Extract { archive, output } => {
                fix_str(archive, base);
                if let Some(o) = output {
                    fix_str(o, base);
                }
            }
        },
        _ => {}
    }
}

/// Resolve the Store/repo working context for the command and change into it
/// (ADR-0005). Repo-level commands resolve a subject repo (with single-subject
/// auto-target); store commands resolve the Store root. Global commands - and
/// `store init`, which creates a Store - return without ever reading the cwd,
/// which may be invalid (e.g. a deleted directory inherited from a parent).
fn apply_context(command: &mut Commands) -> Result<()> {
    enum Ctx {
        None,
        Store,
        Repo,
    }
    let ctx = match command {
        Commands::Store {
            command: StoreCommands::Init { .. },
        } => Ctx::None,
        Commands::Config { .. } => Ctx::None,
        Commands::Store { .. } => Ctx::Store,
        Commands::Journal { .. }
        | Commands::State { .. }
        | Commands::Remote { .. }
        | Commands::Encrypt { .. }
        | Commands::Decrypt { .. }
        | Commands::Status
        | Commands::Transport { .. }
        | Commands::Document { .. }
        | Commands::Import { .. }
        | Commands::User { .. } => Ctx::Repo,
        _ => Ctx::None,
    };
    let target = match ctx {
        Ctx::None => return Ok(()),
        Ctx::Store => commands::context::resolve_store_root()?,
        Ctx::Repo => {
            // External path args are made absolute against the cwd before the cd.
            let cwd = std::env::current_dir()?;
            absolutize_external_paths(command, &cwd);
            commands::context::resolve_repo_root()?
        }
    };
    std::env::set_current_dir(target)?;
    Ok(())
}
