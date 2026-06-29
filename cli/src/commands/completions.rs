// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, anyhow};
use clap::{Command, Subcommand};
use clap_complete::{Shell, generate};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Subcommand)]
pub enum CompletionCommand {
    /// Install completions for the current user.
    Install {
        /// Shell to install completions for. Detected from $SHELL when omitted.
        #[arg(long)]
        shell: Option<Shell>,

        /// Completion directory to write to.
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
}

pub fn run(
    command: Option<CompletionCommand>,
    shell: Option<Shell>,
    dir: Option<&Path>,
    cmd: &mut Command,
) -> Result<()> {
    match command {
        Some(CompletionCommand::Install { shell, dir }) => {
            let shell = shell.or_else(detect_shell).ok_or_else(|| {
                anyhow!("could not detect shell; pass --shell bash|zsh|fish|powershell|elvish")
            })?;
            let dir = dir
                .map(Ok)
                .unwrap_or_else(|| default_completion_dir(shell))?;
            write_completion(shell, cmd, &dir)?;
            print_install_note(shell, &dir);
        }
        None => {
            let shell =
                shell.ok_or_else(|| anyhow!("missing shell; try `gitehr completions install`"))?;
            if let Some(dir) = dir {
                write_completion(shell, cmd, dir)?;
            } else {
                generate(shell, cmd, "gitehr", &mut std::io::stdout());
            }
        }
    }
    Ok(())
}

fn write_completion(shell: Shell, cmd: &mut Command, dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dir)
        .with_context(|| format!("creating completion directory {}", dir.display()))?;
    let path = dir.join(completion_filename(shell));
    let mut file = fs::File::create(&path)
        .with_context(|| format!("creating completion file {}", path.display()))?;
    generate(shell, cmd, "gitehr", &mut file);
    println!("Completion script written to: {}", path.display());
    Ok(path)
}

fn completion_filename(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "gitehr",
        Shell::Zsh => "_gitehr",
        Shell::Fish => "gitehr.fish",
        Shell::PowerShell => "gitehr.ps1",
        Shell::Elvish => "gitehr.elv",
        _ => "gitehr.completion",
    }
}

fn detect_shell() -> Option<Shell> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell).file_name()?.to_string_lossy();
    match name.as_ref() {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

fn default_completion_dir(shell: Shell) -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow!("could not determine home directory"))?;
    Ok(match shell {
        Shell::Bash => env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("bash-completion/completions"),
        Shell::Zsh => home.join(".zfunc"),
        Shell::Fish => env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
            .join("fish/completions"),
        Shell::PowerShell => home.join(".config/powershell/completions"),
        Shell::Elvish => home.join(".elvish/lib"),
        _ => home.join(".local/share/gitehr/completions"),
    })
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn print_install_note(shell: Shell, dir: &Path) {
    match shell {
        Shell::Zsh => {
            println!("Add this before `compinit` in ~/.zshrc if it is not already there:");
            println!("  fpath=({} $fpath)", dir.display());
            println!("Then restart zsh or run `autoload -Uz compinit && compinit`.");
        }
        Shell::PowerShell => {
            println!("Add this to your PowerShell profile if it is not already there:");
            println!("  . {}/gitehr.ps1", dir.display());
        }
        _ => println!("Restart your shell to load the updated completions."),
    }
}
