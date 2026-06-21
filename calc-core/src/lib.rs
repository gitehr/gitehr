//! # calc-core
//!
//! The pure scoring engine behind the GitEHR clinical calculators.
//!
//! This crate is deliberately a **leaf**: it depends only on `serde` and
//! `serde_json`, never on the rest of GitEHR and never on an async runtime.
//! That is what lets the same logic drive four surfaces without divergence:
//!
//! - the standalone `calc` binary and the `gitehr calc` subcommand (via `calc-cli`)
//! - the GitEHR MCP server (each calculator exposed as a tool)
//! - the GitEHR Tauri GUI (called natively over a Tauri command)
//! - a separately-distributed desktop/mobile calculator app
//!
//! Every calculator implements the [`Calculator`] trait and returns a
//! [`CalculationResponse`] — the Rust counterpart of the JSON schema the
//! web calculators dispatch via `gitehr-bridge.js`.
//!
//! Scoring functions are pure: no clock, no I/O, no global state. A host that
//! needs a timestamp stamps it when recording the result, so the core stays
//! deterministic and trivially testable.

pub mod calculator;
pub mod calculators;
pub mod response;
pub mod template;

pub use calculator::{CalcError, Calculator};
pub use response::CalculationResponse;
pub use template::template_from_schema;

/// Every calculator known to the engine, in display order.
///
/// This is the single registry the CLI, MCP server, and GUI all enumerate, so
/// adding a calculator in one place surfaces it everywhere.
pub fn all() -> Vec<Box<dyn Calculator>> {
    vec![
        Box::new(calculators::feverpain::FeverPain),
        Box::new(calculators::asrs::Asrs),
    ]
}

/// Look up a single calculator by its machine name (e.g. `"feverpain"`).
pub fn get(name: &str) -> Option<Box<dyn Calculator>> {
    all().into_iter().find(|c| c.name() == name)
}
