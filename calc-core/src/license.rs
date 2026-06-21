//! Distribution licence / provenance for a calculator's clinical algorithm.

use serde::{Deserialize, Serialize};

/// The terms under which a calculator's clinical algorithm or content is
/// distributed, with a URL evidencing them so the basis can be reverified.
///
/// This is the licence of the *algorithm and clinical content*, which is
/// distinct from the licence of the calc-core source code (AGPL-3.0). Pure
/// scoring algorithms are generally not subject to copyright and are
/// implemented here from the primary literature; some instruments (for example
/// questionnaires) carry an explicit licence or public-domain grant from their
/// owner. Either way, every [`Calculator`](crate::Calculator) must declare one,
/// so the basis for distributing each calculator is always on record and can be
/// re-evidenced from the cited source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculatorLicense {
    /// The licence or terms the algorithm/content is available under: an SPDX
    /// identifier where one applies, otherwise a short description (for example
    /// "Public domain - no permission required").
    pub license: &'static str,
    /// A URL evidencing the licence or terms, for reverification.
    pub source_url: &'static str,
}
