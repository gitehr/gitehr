//! # calc-cli
//!
//! The command-line surface for the clinical calculators. This crate is the
//! single source of CLI behaviour: the standalone `calc` binary (`src/main.rs`)
//! and the `gitehr calc` subcommand both drive [`CalcCommand`] + [`run`], so
//! there is nothing to re-implement when embedding it in GitEHR.
//!
//! To embed in the gitehr CLI:
//!
//! ```ignore
//! #[derive(clap::Subcommand)]
//! enum Commands {
//!     // ...
//!     /// Clinical calculators
//!     Calc {
//!         #[command(subcommand)]
//!         command: calc_cli::CalcCommand,
//!         #[arg(long, value_enum, default_value_t = calc_cli::OutputFormat::Text)]
//!         format: calc_cli::OutputFormat,
//!     },
//! }
//! // ...
//! Commands::Calc { command, format } => calc_cli::run(command, format)?,
//! ```

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand, ValueEnum};

use calc_core::calculators::{asrs, feverpain};
use calc_core::CalculationResponse;

/// How to render results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text.
    Text,
    /// Machine-readable JSON (the `CalculationResponse` schema).
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Text
    }
}

/// The `calc` subcommands. Reused unchanged by `gitehr calc`.
#[derive(Debug, Subcommand)]
pub enum CalcCommand {
    /// List the available calculators.
    List,

    /// FeverPAIN score for acute sore throat (antibiotic stewardship).
    Feverpain(FeverpainArgs),

    /// ASRS-v1.1 adult ADHD self-report screener.
    Asrs(AsrsArgs),
}

#[derive(Debug, Args)]
pub struct FeverpainArgs {
    /// Fever in the last 24 hours.
    #[arg(long)]
    pub fever: bool,
    /// Purulence (pus on the tonsils).
    #[arg(long)]
    pub purulence: bool,
    /// Symptom onset within 3 days (attend rapidly).
    #[arg(long)]
    pub attend_rapidly: bool,
    /// Severely inflamed tonsils.
    #[arg(long)]
    pub inflamed_tonsils: bool,
    /// No cough or coryza.
    #[arg(long)]
    pub absence_of_cough: bool,
    /// Print the JSON Schema for this calculator's inputs and exit.
    #[arg(long)]
    pub print_schema: bool,
}

#[derive(Debug, Args)]
pub struct AsrsArgs {
    /// The 18 frequency responses Q1–Q18, comma-separated, each 0–4
    /// (0=Never, 1=Rarely, 2=Sometimes, 3=Often, 4=Very Often).
    #[arg(long, value_delimiter = ',')]
    pub responses: Option<Vec<u8>>,
    /// Print the JSON Schema for this calculator's inputs and exit.
    #[arg(long)]
    pub print_schema: bool,
}

/// Dispatch a parsed [`CalcCommand`].
pub fn run(command: CalcCommand, format: OutputFormat) -> Result<()> {
    match command {
        CalcCommand::List => print_list(format),
        CalcCommand::Feverpain(args) => {
            if args.print_schema {
                return print_schema(feverpain::NAME, format);
            }
            let input = feverpain::FeverPainInput {
                fever: args.fever,
                purulence: args.purulence,
                attend_rapidly: args.attend_rapidly,
                inflamed_tonsils: args.inflamed_tonsils,
                absence_of_cough: args.absence_of_cough,
            };
            emit(&feverpain::build_response(input), format)
        }
        CalcCommand::Asrs(args) => {
            if args.print_schema {
                return print_schema(asrs::NAME, format);
            }
            let responses = args.responses.ok_or_else(|| {
                anyhow!("--responses is required (18 comma-separated values 0–4); see `calc asrs --print-schema`")
            })?;
            let response = asrs::build_response(&asrs::AsrsInput { responses })?;
            emit(&response, format)
        }
    }
}

fn print_list(format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let items: Vec<_> = calc_core::all()
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "name": c.name(),
                        "title": c.title(),
                        "description": c.description(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text => {
            for c in calc_core::all() {
                println!("{:<12}  {}", c.name(), c.title());
            }
        }
    }
    Ok(())
}

fn print_schema(name: &str, _format: OutputFormat) -> Result<()> {
    let calc = calc_core::get(name).ok_or_else(|| anyhow!("unknown calculator: {name}"))?;
    // Schema is always JSON, regardless of result format.
    println!("{}", serde_json::to_string_pretty(&calc.input_schema())?);
    Ok(())
}

fn emit(response: &CalculationResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(response)?),
        OutputFormat::Text => println!("{}", render_text(response)),
    }
    Ok(())
}

/// Render a result as a clinician-facing text block.
fn render_text(r: &CalculationResponse) -> String {
    let mut out = String::new();
    out.push_str(&format!("{} = {}\n\n", r.calculator, value_to_string(&r.result)));
    out.push_str(&r.interpretation);
    if !r.working.is_empty() {
        out.push_str("\n\nWorking:");
        for (k, v) in &r.working {
            out.push_str(&format!("\n  {k}: {}", value_to_string(v)));
        }
    }
    out.push_str(&format!("\n\nReference: {}", r.reference));
    out
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
