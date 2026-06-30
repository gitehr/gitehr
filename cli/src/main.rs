// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::path::{Path, PathBuf};

mod commands;
mod config;
mod utils;

use commands::allergies::AllergyCommands;
use commands::config::ConfigCommands;
use commands::demographics::DemographicsCommands;
use commands::document::DocumentCommands;
use commands::journal::JournalCommands;
use commands::mcp::McpCommands;
use commands::remote::RemoteCommands;
use commands::state::StateCommands;
use commands::store::StoreCommands;
use commands::transport::TransportCommands;
use commands::user::UserCommands;
use commands::vaccinations::VaccinationCommands;

#[derive(Parser)]
#[command(name = "gitehr")]
#[command(about = "The Git-based Electronic Health Record", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage typed allergy and adverse-reaction state
    Allergies {
        #[command(subcommand)]
        command: AllergyCommands,
    },
    #[command(
        about = "Generate shell completions",
        long_about = r#"Generate shell completions for gitehr.

Examples:
  gitehr completions install
  gitehr completions zsh --dir ~/.zfunc
  gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr

Restart your shell after installing completions."#
    )]
    Completions {
        #[command(subcommand)]
        command: Option<commands::completions::CompletionCommand>,
        #[arg(help = "Shell type (bash, zsh, fish, powershell)")]
        shell: Option<clap_complete::Shell>,
        #[arg(long, short = 'd', help = "Output directory")]
        dir: Option<PathBuf>,
    },
    /// Manage local GitEHR configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Decrypt {
        #[arg(long, help = "Key source (local or remote URL)")]
        key: Option<String>,
    },
    /// Manage typed patient demographics state
    Demographics {
        #[command(subcommand)]
        command: DemographicsCommands,
    },
    #[command(alias = "attach")]
    Document {
        #[command(subcommand)]
        command: DocumentCommands,
    },
    Encrypt {
        #[arg(long, help = "Key source (local or remote URL)")]
        key: Option<String>,
    },
    Gui,
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
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// List installed plugins (gitehr-<command> executables on PATH)
    Plugins,
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommands>,
    },
    State {
        #[command(subcommand)]
        command: Option<StateCommands>,
    },
    #[command(visible_alias = "st")]
    Status,
    /// Manage a multi-patient store and its Main Patient Index (MPI)
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
    Transport {
        #[command(subcommand)]
        command: Option<TransportCommands>,
    },
    // Clinical calculators are temporarily dormant while pacharanero/calc is
    // pre-crates.io; keep GitEHR's release pipeline free of git-only
    // dependencies. Restore `Calc(calc_cli::CalcCommand)` once calc-cli is
    // published.
    Upgrade,
    #[command(
        name = "upgrade-binary",
        about = "Update the bundled binary to the current CLI version"
    )]
    UpgradeBinary,
    /// Manage contributors and the active author
    #[command(visible_alias = "contributor")]
    User {
        #[command(subcommand)]
        command: Option<UserCommands>,
    },
    /// Manage typed vaccination and immunisation state
    #[command(visible_aliases = ["immunisations", "immunizations"])]
    Vaccinations {
        #[command(subcommand)]
        command: VaccinationCommands,
    },
    #[command(visible_alias = "v")]
    Version,
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
        Commands::Allergies { command } => commands::allergies::run(command)?,
        Commands::Completions {
            command,
            shell,
            dir,
        } => {
            let mut cmd = Cli::command();
            commands::completions::run(command, shell, dir.as_deref(), &mut cmd)?;
        }
        Commands::Config { command } => commands::config::run(command)?,
        Commands::Decrypt { key } => commands::decrypt::run(key.as_deref())?,
        Commands::Demographics { command } => commands::demographics::run(command)?,
        Commands::Document { command } => commands::document::run(command)?,
        Commands::Encrypt { key } => commands::encrypt::run(key.as_deref())?,
        Commands::Gui => commands::gui::run()?,
        Commands::Import { mode, path } => commands::import::run(mode, &path)?,
        Commands::Journal { command } => commands::journal::run(command)?,
        Commands::Mcp { command } => commands::mcp::run(command)?,
        Commands::Plugins => commands::plugin::list(&builtins)?,
        Commands::Remote { command } => commands::remote::run(command)?,
        Commands::State { command } => commands::state::run(command)?,
        Commands::Status => commands::status::run()?,
        Commands::Store { command } => commands::store::run(command)?,
        Commands::Transport { command } => commands::transport::run(command)?,
        Commands::Upgrade => commands::upgrade::run()?,
        Commands::UpgradeBinary => commands::upgrade_binary::run()?,
        Commands::User { command } => commands::user::run(command)?,
        Commands::Vaccinations { command } => commands::vaccinations::run(command)?,
        Commands::Version => commands::version::run(),
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
            command: DocumentCommands::Add { paths, .. },
        } => {
            for path in paths {
                fix_pb(path, base);
            }
        }
        Commands::Vaccinations {
            command:
                VaccinationCommands::Add {
                    fhir_json: Some(path),
                    ..
                },
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
        | Commands::Demographics { .. }
        | Commands::Allergies { .. }
        | Commands::Vaccinations { .. }
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
