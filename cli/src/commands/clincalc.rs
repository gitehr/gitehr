// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `gitehr clincalc record` (R25): run a calculator through the external
//! `gitehr-clincalc` plugin and record the calculator, its version, inputs,
//! result, and citation as an immutable journal entry.
//!
//! Every other `gitehr clincalc ...` invocation continues to pass straight
//! through to the plugin unchanged (see [`super::plugin`]); `main.rs`
//! intercepts only the `record` subcommand ahead of that fallthrough, so
//! `gitehr clincalc list`, `gitehr clincalc <name> --schema`, and friends are
//! untouched by this module.

use anyhow::{Context, Result, bail};
use clap::Parser;
use std::io::{IsTerminal, Read};
use std::process::Command;

use super::journal::{self, ClincalcRecord};

#[derive(Parser)]
#[command(name = "gitehr clincalc record")]
struct RecordArgs {
    /// Calculator name, as accepted by `gitehr clincalc <name>`.
    name: String,
    /// JSON input: '-' for stdin, an inline JSON string, or a file path.
    #[arg(long)]
    input: String,
}

/// `args` is everything after `clincalc record`, e.g. `["feverpain",
/// "--input", "-"]`.
pub fn run(args: &[String]) -> Result<()> {
    let parsed = RecordArgs::try_parse_from(
        std::iter::once("gitehr clincalc record".to_string()).chain(args.iter().cloned()),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    let input_json = resolve_input(&parsed.input)?;
    let inputs: serde_json::Value =
        serde_json::from_str(&input_json).context("clincalc input is not valid JSON")?;

    let output = run_plugin(&parsed.name, &input_json)?;
    let result: serde_json::Value =
        serde_json::from_slice(&output).context("gitehr-clincalc did not return valid JSON")?;

    let record = ClincalcRecord {
        calculator: parsed.name.clone(),
        version: plugin_version(),
        inputs,
        result: result
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        interpretation: result
            .get("interpretation")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        reference: result
            .get("reference")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    };

    let body = format!(
        "# Clinical calculator: {}\n\n{}\n",
        record.calculator,
        record
            .interpretation
            .as_deref()
            .unwrap_or("(no interpretation provided)")
    );

    journal::create_journal_entry_with_clincalc(&body, record)
}

/// Resolve `--input` per the documented three-way contract: `-` reads stdin,
/// a value that looks like JSON is used inline, and anything else is read as
/// a file path.
fn resolve_input(input: &str) -> Result<String> {
    if input == "-" {
        let mut buf = String::new();
        if std::io::stdin().is_terminal() {
            bail!("--input - requires piped stdin");
        }
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("failed to read clincalc input from stdin")?;
        Ok(buf)
    } else {
        let trimmed = input.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            Ok(input.to_string())
        } else {
            std::fs::read_to_string(input)
                .with_context(|| format!("failed to read clincalc input from {input}"))
        }
    }
}

/// Run `gitehr-clincalc <name> --input <json> --format json`, on `$PATH`,
/// captured (not exec'd, unlike [`super::plugin::run`]) so the result can be
/// parsed and recorded.
fn run_plugin(name: &str, input_json: &str) -> Result<Vec<u8>> {
    let exe = which::which("gitehr-clincalc")
        .map_err(|_| anyhow::anyhow!("gitehr-clincalc not found on PATH"))?;

    let output = Command::new(&exe)
        .args([name, "--input", input_json, "--format", "json"])
        .output()
        .with_context(|| format!("failed to run {}", exe.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("gitehr-clincalc {name} failed: {}", stderr.trim());
    }

    Ok(output.stdout)
}

/// Best-effort plugin version, omitted (rather than faked) when it cannot be
/// determined.
fn plugin_version() -> Option<String> {
    let exe = which::which("gitehr-clincalc").ok()?;
    let output = Command::new(exe).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_input_treats_braces_and_brackets_as_inline_json() {
        assert_eq!(resolve_input(r#"{"a":1}"#).unwrap(), r#"{"a":1}"#);
        assert_eq!(resolve_input("[1,2,3]").unwrap(), "[1,2,3]");
    }

    #[test]
    fn resolve_input_reads_a_file_when_not_json_or_dash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input.json");
        std::fs::write(&path, r#"{"fever":true}"#).unwrap();

        let resolved = resolve_input(path.to_str().unwrap()).unwrap();
        assert_eq!(resolved, r#"{"fever":true}"#);
    }

    #[test]
    fn resolve_input_reports_a_clear_error_for_a_missing_file() {
        let err = resolve_input("/nonexistent/path/does-not-exist.json").unwrap_err();
        assert!(err.to_string().contains("failed to read clincalc input"));
    }
}
