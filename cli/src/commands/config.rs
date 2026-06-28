// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use crate::config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Print the config file path GitEHR will use
    Path,
    /// Print the current config values
    Show,
    /// Set the default Store root used outside a repo or Store
    SetStore {
        #[arg(help = "Existing Store root containing gitehr-mpi.json")]
        path: PathBuf,
    },
}

pub fn run(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Path => {
            println!("{}", config::config_path()?.display());
        }
        ConfigCommands::Show => {
            let config_path = config::config_path()?;
            let store_path = config::configured_store_path()?;
            println!("config_path: {}", config_path.display());
            match store_path {
                Some(path) => println!("store_path: {}", path.display()),
                None => println!("store_path: <unset>"),
            }
        }
        ConfigCommands::SetStore { path } => {
            let store_path = config::set_store_path(&path)?;
            println!("store_path: {}", store_path.display());
            println!("config_path: {}", config::config_path()?.display());
        }
    }
    Ok(())
}
