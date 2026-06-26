// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;

pub mod activate;
pub mod add;
pub mod create;
pub mod deactivate;
pub mod disable;
pub mod enable;
pub mod list;

#[derive(Subcommand)]
pub enum UserCommands {
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

pub fn run(command: Option<UserCommands>) -> Result<()> {
    match command {
        Some(UserCommands::Create) => create::run(),
        Some(UserCommands::Add {
            id,
            name,
            role,
            email,
        }) => add::run(&id, &name, role.as_deref(), email.as_deref()),
        Some(UserCommands::Enable { id }) => enable::run(&id),
        Some(UserCommands::Disable { id }) => disable::run(&id),
        Some(UserCommands::Activate { id }) => activate::run(&id),
        Some(UserCommands::Deactivate) => deactivate::run(),
        Some(UserCommands::List) | None => list::run(),
    }
}
