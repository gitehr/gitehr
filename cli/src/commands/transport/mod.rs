// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use clap::Subcommand;

pub mod create;
pub mod extract;

#[derive(Subcommand)]
pub enum TransportCommands {
    /// Create a transport archive from this repository
    Create {
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
        #[arg(long, help = "Apply additional encryption")]
        encrypt: bool,
    },
    /// Extract a transport archive
    Extract {
        #[arg(help = "Path to the transport archive")]
        archive: String,
        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
}

pub fn run(command: Option<TransportCommands>) -> Result<()> {
    match command {
        Some(TransportCommands::Create { output, encrypt }) => {
            create::run(output.as_deref(), encrypt)
        }
        Some(TransportCommands::Extract { archive, output }) => {
            extract::run(&archive, output.as_deref())
        }
        None => {
            println!("Usage: gitehr transport <create|extract>");
            println!();
            println!("Subcommands:");
            println!("  create    Create a transport archive from this repository");
            println!("  extract   Extract a transport archive");
            Ok(())
        }
    }
}
