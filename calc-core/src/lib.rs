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
pub mod license;
pub mod proprietary;
pub mod response;
pub mod template;

pub use calculator::{CalcError, Calculator};
pub use license::CalculatorLicense;
pub use proprietary::ProprietaryCalculator;
pub use response::CalculationResponse;
pub use template::template_from_schema;

/// Every calculator known to the engine, in display order.
///
/// This is the single registry the CLI, MCP server, and GUI all enumerate, so
/// adding a calculator in one place surfaces it everywhere.
pub fn all() -> Vec<Box<dyn Calculator>> {
    let mut list: Vec<Box<dyn Calculator>> = vec![
        Box::new(calculators::feverpain::FeverPain),
        Box::new(calculators::asrs::Asrs),
        Box::new(calculators::phq9::Phq9),
        Box::new(calculators::gad7::Gad7),
        Box::new(calculators::egfr::Egfr),
        Box::new(calculators::fib4::Fib4),
        Box::new(calculators::cha2ds2vasc::Cha2ds2Vasc),
        Box::new(calculators::auditc::AuditC),
        Box::new(calculators::audit::Audit),
        Box::new(calculators::epds::Epds),
        Box::new(calculators::ipss::Ipss),
        Box::new(calculators::amts::Amts),
        Box::new(calculators::mrc_dyspnoea::MrcDyspnoea),
        Box::new(calculators::news2::News2),
        Box::new(calculators::curb65::Curb65),
        Box::new(calculators::wells_dvt::WellsDvt),
        Box::new(calculators::wells_pe::WellsPe),
        Box::new(calculators::hasbled::HasBled),
        Box::new(calculators::abcd2::Abcd2),
        Box::new(calculators::qsofa::Qsofa),
        Box::new(calculators::fourat::FourAt),
        Box::new(calculators::das28::Das28),
        Box::new(calculators::uacr::Uacr),
        Box::new(calculators::sofa::Sofa),
        Box::new(calculators::heart::Heart),
        Box::new(calculators::timi::Timi),
        Box::new(calculators::child_pugh::ChildPugh),
        Box::new(calculators::meld::Meld),
        Box::new(calculators::padua::Padua),
        Box::new(calculators::ukeld::Ukeld),
        Box::new(calculators::nhfs::Nhfs),
        Box::new(calculators::bode::Bode),
        Box::new(calculators::abpi::Abpi),
        Box::new(calculators::waterlow::Waterlow),
    ];
    // Proprietary / licence-locked tools: registered so a clinician learns why
    // they are absent and where to turn, rather than finding silence.
    for p in proprietary::PROPRIETARY {
        list.push(Box::new(*p));
    }
    list
}

/// Look up a single calculator by its machine name (e.g. `"feverpain"`).
pub fn get(name: &str) -> Option<Box<dyn Calculator>> {
    all().into_iter().find(|c| c.name() == name)
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    /// Policy: every calculator must declare a non-empty distribution licence
    /// with a reverifiable source URL. This guards the requirement at CI time,
    /// so a new calculator cannot ship without recording the basis on which we
    /// distribute it.
    #[test]
    fn every_calculator_records_its_license() {
        for calc in all() {
            let lic = calc.license();
            assert!(
                !lic.license.trim().is_empty(),
                "{}: license must not be empty",
                calc.name()
            );
            assert!(
                lic.source_url.starts_with("http"),
                "{}: license source_url must be a URL, got {:?}",
                calc.name(),
                lic.source_url
            );
        }
    }
}
