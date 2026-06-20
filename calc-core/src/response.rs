//! The result schema returned by every calculator.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A completed calculation, ready to be rendered, recorded, or dispatched.
///
/// This mirrors the JSON object the web calculators send through
/// `gitehr-bridge.js` (`calculator`, `result`, `interpretation`, `working`,
/// `reference`), so a result produced here and one produced in the browser are
/// the same shape by construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculationResponse {
    /// Machine name of the calculator, e.g. `"feverpain"`.
    pub calculator: String,
    /// The primary computed value — a number or short string.
    pub result: Value,
    /// Human-readable clinical interpretation.
    pub interpretation: String,
    /// Step-by-step breakdown of how the result was reached.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub working: Map<String, Value>,
    /// Primary citation / guideline reference.
    pub reference: String,
}

impl CalculationResponse {
    /// A plain-text summary suitable for a clipboard / journal entry.
    ///
    /// Intentionally free of any timestamp — the recording host adds that, so
    /// this output stays deterministic.
    pub fn to_summary_text(&self) -> String {
        let result = match &self.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        format!(
            "Calculator: {}\nResult: {}\nInterpretation: {}\nReference: {}",
            self.calculator, result, self.interpretation, self.reference
        )
    }
}
