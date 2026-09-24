// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueHint};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const CLINCALC_VERSION: &str = "0.3.4";

#[derive(Parser)]
#[command(name = "gitehr-clincalc")]
#[command(about = "Run clincalc calculators and record verified results in GitEHR")]
#[command(version)]
#[command(
    after_help = "Use `gitehr clincalc record <calculator> --input <JSON|FILE|->` to calculate and add an immutable journal entry. All other arguments are handled by clincalc."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<PluginCommand>,
    #[command(flatten)]
    calculator: clincalc::cli::CalcCommand,
}

#[derive(Subcommand)]
enum PluginCommand {
    #[command(about = "Calculate from JSON input and record the verified result in GitEHR")]
    Record(RecordCommand),
    #[command(about = "Print plugin and calculator-engine versions")]
    Version {
        #[arg(long, value_enum, default_value_t = clincalc::cli::OutputFormat::Text)]
        format: clincalc::cli::OutputFormat,
    },
}

#[derive(Args)]
struct RecordCommand {
    #[arg(help = "Calculator machine name (use `gitehr clincalc list` to discover names)")]
    calculator: String,
    #[arg(
        long,
        required = true,
        value_name = "JSON|FILE|-",
        value_hint = ValueHint::AnyPath,
        help = "JSON input: '-' for stdin, a file path, or an inline JSON string"
    )]
    input: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(PluginCommand::Record(command)) => record(command),
        Some(PluginCommand::Version { format }) => print_version(format),
        None => clincalc::cli::run(cli.calculator),
    }
}

fn print_version(format: clincalc::cli::OutputFormat) -> Result<()> {
    match format {
        clincalc::cli::OutputFormat::Text => {
            println!(
                "gitehr-clincalc {}\nclincalc {CLINCALC_VERSION}",
                env!("CARGO_PKG_VERSION")
            );
        }
        clincalc::cli::OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "clincalc_version": CLINCALC_VERSION,
            }))?
        ),
        clincalc::cli::OutputFormat::Markdown => println!(
            "# gitehr-clincalc {}\n\n- clincalc: {}",
            env!("CARGO_PKG_VERSION"),
            CLINCALC_VERSION
        ),
    }
    Ok(())
}

fn record(command: RecordCommand) -> Result<()> {
    let input = read_input(&command.input)?;
    let body = render_record(&command.calculator, input)?;
    add_journal_entry(&body)
}

fn read_input(source: &str) -> Result<Value> {
    let raw = if source == "-" {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .context("reading calculation input from stdin")?;
        input
    } else {
        let path = tilde_path(source);
        if path.is_file() {
            std::fs::read_to_string(&path)
                .with_context(|| format!("reading calculation input from {}", path.display()))?
        } else {
            source.to_string()
        }
    };
    serde_json::from_str(&raw).context("calculation input must be valid JSON")
}

fn tilde_path(source: &str) -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    match (source, home) {
        ("~", Some(home)) => PathBuf::from(home),
        (_, Some(home)) if source.starts_with("~/") => PathBuf::from(home).join(&source[2..]),
        _ => PathBuf::from(source),
    }
}

fn render_record(calculator_name: &str, input: Value) -> Result<String> {
    let calculator = clincalc::get(calculator_name).ok_or_else(|| {
        anyhow::anyhow!("unknown calculator: {calculator_name}; use `gitehr clincalc list`")
    })?;
    let response = calculator
        .calculate(&input)
        .map_err(|error| anyhow::anyhow!("invalid input for {calculator_name}: {error}"))?;

    if response.calculator != calculator.name()
        || response.interpretation.trim().is_empty()
        || response.reference.trim().is_empty()
    {
        bail!("clincalc returned an incomplete calculation response; refusing to record it");
    }

    let evidence = json!({
        "calculator": response.calculator,
        "clincalc_version": CLINCALC_VERSION,
        "input": input,
        "response": response,
    });
    let response = evidence["response"]
        .as_object()
        .expect("calculation response is an object");
    let result = serde_json::to_string(&response["result"])?;
    let interpretation = response["interpretation"]
        .as_str()
        .expect("calculation response interpretation is a string");
    let reference = response["reference"]
        .as_str()
        .expect("calculation response reference is a string");

    Ok(format!(
        "# Clinical calculation: {}\n\n**Engine:** clincalc {}\n\n**Result:** `{result}`\n\n## Interpretation\n\n{interpretation}\n\n## Reference\n\n{reference}\n\n## Verifiable calculation data\n\n{}",
        evidence["calculator"]
            .as_str()
            .expect("calculation response calculator is a string"),
        CLINCALC_VERSION,
        fenced_json(&evidence)?
    ))
}

fn fenced_json(value: &Value) -> Result<String> {
    let json = serde_json::to_string_pretty(value)?;
    let longest_backtick_run = json
        .as_bytes()
        .split(|byte| *byte != b'`')
        .map(|run| run.len())
        .max()
        .unwrap_or_default();
    let fence = "`".repeat(longest_backtick_run.saturating_add(1).max(3));
    Ok(format!("{fence}json\n{json}\n{fence}"))
}

fn add_journal_entry(body: &str) -> Result<()> {
    let mut child = Command::new("gitehr")
        .args(["journal", "add", "--file", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("starting `gitehr journal add`; install gitehr and ensure it is on PATH")?;
    let write_result = child
        .stdin
        .take()
        .expect("child stdin was piped")
        .write_all(body.as_bytes());
    let status = child.wait().context("waiting for `gitehr journal add`")?;
    write_result.context("sending calculation data to `gitehr journal add`")?;
    if !status.success() {
        bail!("`gitehr journal add` failed with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CLINCALC_VERSION, print_version, render_record};
    use clincalc::cli::OutputFormat;
    use serde_json::json;

    #[test]
    fn record_contains_complete_verifiable_calculation_data() {
        let body = render_record(
            "feverpain",
            json!({
                "fever": true,
                "purulence": true,
                "attend_rapidly": true,
                "inflamed_tonsils": false,
                "absence_of_cough": false,
            }),
        )
        .unwrap();

        assert!(body.contains("# Clinical calculation: feverpain"));
        assert!(body.contains("\"clincalc_version\": \"0.3.4\""));
        assert!(body.contains("\"input\""));
        assert!(body.contains("\"response\""));
        assert!(body.contains("\"reference\""));
    }

    #[test]
    fn machine_readable_version_is_available() {
        print_version(OutputFormat::Json).unwrap();
    }

    #[test]
    fn recorded_engine_version_matches_the_exact_workspace_dependency() {
        assert!(
            include_str!("../../Cargo.toml")
                .contains(&format!("clincalc = {{ version = \"={CLINCALC_VERSION}\""))
        );
    }
}
