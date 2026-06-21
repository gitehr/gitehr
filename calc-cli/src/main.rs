//! The standalone `calc` binary.
//!
//! A thin wrapper over [`calc_cli`]: all behaviour lives in the library so the
//! `gitehr calc` subcommand can reuse it without repetition.

use clap::Parser;

use calc_cli::CalcCommand;

/// Open clinical calculators - scoring at the command line.
#[derive(Debug, Parser)]
#[command(name = "calc", version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    command: CalcCommand,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    calc_cli::run(cli.command)
}
