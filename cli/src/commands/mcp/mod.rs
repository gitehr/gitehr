// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

pub mod serve;

#[derive(Subcommand)]
pub enum McpCommands {
    #[command(about = "Start MCP server")]
    Serve {
        #[arg(long, help = "Use stdio transport (default)")]
        stdio: bool,
        #[arg(long, help = "Repository path (default: current directory)")]
        repo_path: Option<PathBuf>,
    },
}

pub fn run(command: McpCommands) -> Result<()> {
    match command {
        McpCommands::Serve {
            stdio: _,
            repo_path,
        } => serve::run(repo_path),
    }
}
