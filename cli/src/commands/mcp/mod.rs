// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

pub mod serve;
mod server_impl;

const MCP_VERSION: &str = "2024-11-05";

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let _ = fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}

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
