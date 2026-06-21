//! The [`Calculator`] trait and its error type.

use serde_json::Value;

use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Something went wrong turning inputs into a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    /// The supplied inputs were malformed, out of range, or the wrong shape.
    InvalidInput(String),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
        }
    }
}

impl std::error::Error for CalcError {}

/// A clinical calculator: metadata plus a dynamic JSON entrypoint.
///
/// Each implementation also exposes a strongly-typed `Input`/`compute` pair in
/// its own module; the trait is the uniform surface the CLI, MCP server, and
/// GUI dispatch through without knowing about any specific calculator.
pub trait Calculator {
    /// Machine name, e.g. `"feverpain"`. Stable; used as a CLI subcommand and
    /// MCP tool name.
    fn name(&self) -> &'static str;

    /// Human-readable title, e.g. `"FeverPAIN Score"`.
    fn title(&self) -> &'static str;

    /// One-line description of what the calculator does.
    fn description(&self) -> &'static str;

    /// Primary citation / guideline reference.
    fn reference(&self) -> &'static str;

    /// The licence the calculator's clinical algorithm/content is distributed
    /// under, with a URL evidencing it.
    ///
    /// This is the algorithm's provenance (distinct from the AGPL code licence).
    /// It is a required method so the basis for shipping every calculator is
    /// always on record and can be re-evidenced from the cited source.
    fn license(&self) -> CalculatorLicense;

    /// JSON Schema describing the accepted inputs.
    ///
    /// Powers `--print-schema` and MCP tool definitions, and lets an LLM work
    /// out the required inputs without parsing prose help.
    fn input_schema(&self) -> Value;

    /// Compute a result from JSON inputs.
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError>;

    /// A fillable input template derived from [`input_schema`](Self::input_schema).
    ///
    /// Every input key is present with a placeholder describing the expected
    /// value, so a caller can fill it in and pass it straight back to
    /// [`calculate`](Self::calculate). Generated from the schema, so it cannot
    /// drift from the real contract.
    fn input_template(&self) -> Value {
        crate::template::template_from_schema(&self.input_schema())
    }
}
