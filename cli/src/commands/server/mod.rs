// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;

pub mod store;
pub mod user;

#[derive(Subcommand)]
pub enum ServerCommands {
    Store {
        #[command(subcommand)]
        command: store::StoreCommands,
    },
    User {
        #[command(subcommand)]
        command: Option<user::UserCommands>,
    },
}

pub fn run(command: ServerCommands) -> Result<()> {
    match command {
        ServerCommands::User { command } => user::run(command),
        ServerCommands::Store { command } => store::run(command),
    }
}
