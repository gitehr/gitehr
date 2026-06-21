//! # calc-cli
//!
//! The command-line surface for the clinical calculators. This crate is the
//! single source of CLI behaviour: the standalone `calc` binary (`src/main.rs`)
//! and the `gitehr calc` subcommand both drive [`CalcCommand`] + [`run`], so
//! there is nothing to re-implement when embedding it in GitEHR.
//!
//! ## One regular surface for every calculator
//!
//! There are no per-calculator flags. Every calculator is driven through the
//! same registry-backed shape, so adding a calculator to `calc-core` gives it a
//! working CLI for free, and a human or an LLM learns the interface once:
//!
//! ```text
//! calc list                       # list available calculators
//! calc <name>                     # print a fillable INPUT TEMPLATE
//! calc <name> --schema            # print the JSON Schema (the full contract)
//! calc <name> --input -           # compute, reading JSON from stdin
//! calc <name> --input data.json   # compute, reading JSON from a file
//! calc <name> --input '{...}'     # compute, reading an inline JSON string
//! ```
//!
//! The template printed by `calc <name>` has the same shape as the input
//! `calc <name> --input` expects: fill in the placeholder values and pass it
//! back. Computing always requires an explicit `--input`, so a bare invocation
//! never blocks reading stdin.
//!
//! To embed in the gitehr CLI:
//!
//! ```ignore
//! #[derive(clap::Subcommand)]
//! enum Commands {
//!     // ...
//!     /// Clinical calculators
//!     Calc(calc_cli::CalcCommand),
//! }
//! // ...
//! Commands::Calc(cmd) => calc_cli::run(cmd)?,
//! ```

use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};

use calc_core::CalculationResponse;

/// How to render computed results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text.
    #[default]
    Text,
    /// Machine-readable JSON (the `CalculationResponse` schema).
    Json,
}

/// The `calc` command surface. Reused unchanged by `gitehr calc`.
///
/// A single shape covers discovery, schema, and compute for every calculator;
/// the calculator is selected by `name` and looked up in the `calc-core`
/// registry at runtime.
#[derive(Debug, Args)]
pub struct CalcCommand {
    /// Calculator machine name (e.g. `feverpain`). Omit, or use `list`, to see
    /// all available calculators.
    pub name: Option<String>,

    /// Compute a result from this JSON input: `-` for stdin, a file path, or an
    /// inline JSON string. Without it, a fillable input template is printed.
    #[arg(long, value_name = "JSON|FILE|-")]
    pub input: Option<String>,

    /// Print the calculator's JSON Schema (the full input contract) instead of a
    /// template.
    #[arg(long)]
    pub schema: bool,

    /// Print the calculator's distribution licence and the URL evidencing it.
    #[arg(long)]
    pub license: bool,

    /// Output format for computed results and `list`.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

/// Dispatch a parsed [`CalcCommand`].
pub fn run(cmd: CalcCommand) -> Result<()> {
    // No name (or `list`) means: show the catalogue.
    let name = match cmd.name.as_deref() {
        None | Some("list") => return print_list(cmd.format),
        Some(n) => n,
    };

    let calc = calc_core::get(name)
        .ok_or_else(|| anyhow!("unknown calculator: {name} (try `calc list`)"))?;

    // `--schema` prints the formal contract, regardless of everything else.
    if cmd.schema {
        println!("{}", serde_json::to_string_pretty(&calc.input_schema())?);
        return Ok(());
    }

    // `--license` prints the algorithm's distribution licence and evidence URL.
    if cmd.license {
        println!("{}", serde_json::to_string_pretty(&calc.license())?);
        return Ok(());
    }

    match cmd.input.as_deref() {
        // No input: print a fillable template and explain how to pass it back.
        None => {
            println!("{}", serde_json::to_string_pretty(&calc.input_template())?);
            eprintln!(
                "\nReplace each placeholder with a value, then compute with one of:\n  \
                 calc {name} --input <file.json>\n  \
                 calc {name} --input '<json>'\n  \
                 calc {name} --input -        # read JSON from stdin\n\
                 See the full input contract with: calc {name} --schema"
            );
            Ok(())
        }
        // Input supplied: validate (via the calculator's typed deserialization)
        // and compute.
        Some(src) => {
            let input = read_input(src)?;
            let response = calc.calculate(&input).map_err(|e| anyhow!("{e}"))?;
            emit(&response, cmd.format)
        }
    }
}

/// Resolve an `--input` argument to a JSON value.
///
/// `-` reads stdin; an existing file path is read from disk; anything else is
/// treated as an inline JSON string.
fn read_input(src: &str) -> Result<serde_json::Value> {
    let raw = if src == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else if Path::new(src).is_file() {
        std::fs::read_to_string(src)?
    } else {
        src.to_string()
    };

    serde_json::from_str(&raw)
        .map_err(|e| anyhow!("invalid JSON input: {e}\nSee the expected shape with: calc <name>"))
}

fn print_list(format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let items: Vec<_> = calc_core::all()
                .iter()
                .map(|c| {
                    let lic = c.license();
                    serde_json::json!({
                        "name": c.name(),
                        "title": c.title(),
                        "description": c.description(),
                        "license": lic.license,
                        "license_source": lic.source_url,
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
