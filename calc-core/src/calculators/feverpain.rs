//! FeverPAIN score for acute sore throat.
//!
//! Five-item validated score guiding antibiotic prescribing. Ported verbatim
//! from the web calculator (`calc-web/calculators/feverpain.html`); the band
//! thresholds, streptococcus isolation rates, and interpretation strings match
//! it exactly so results are identical across surfaces.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "feverpain";

/// Primary citation (matches the payload dispatched by the web calculator).
pub const REFERENCE: &str = "Little P, Stuart B, Hobbs FDR, et al. Lancet Infect Dis. 2014. \
Little P, Hobbs FR, Moore M, et al. Health Technol Assess. 2014;18(6):1-102.";

/// The five FeverPAIN criteria, each worth one point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeverPainInput {
    /// **F** — Fever in the last 24 hours.
    pub fever: bool,
    /// **P** — Purulence (pus on the tonsils).
    pub purulence: bool,
    /// **A** — Attend rapidly: symptom onset within 3 days (≤ 3 days).
    pub attend_rapidly: bool,
    /// **I** — Severely inflamed tonsils.
    pub inflamed_tonsils: bool,
    /// **N** — No cough or coryza.
    pub absence_of_cough: bool,
}

/// Prescribing band implied by the score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    NoAntibiotic,
    DelayedAntibiotic,
    ImmediateAntibiotic,
}

impl Band {
    /// Stable slug matching the web calculator's `level` value.
    pub fn slug(self) -> &'static str {
        match self {
            Band::NoAntibiotic => "no-rx",
            Band::DelayedAntibiotic => "delayed",
            Band::ImmediateAntibiotic => "immediate",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeverPainOutcome {
    pub score: u8,
    pub band: Band,
    /// Estimated streptococcus isolation rate for this score band.
    pub streptococcus_rate: &'static str,
    /// Short prescribing recommendation.
    pub recommendation: &'static str,
    pub interpretation: String,
}

/// Pure scoring. Mirrors the web calculator's `computeScore` + `getInterpretation`.
pub fn compute(input: FeverPainInput) -> FeverPainOutcome {
    let criteria = [
        input.fever,
        input.purulence,
        input.attend_rapidly,
        input.inflamed_tonsils,
        input.absence_of_cough,
    ];
    let score = criteria.iter().filter(|met| **met).count() as u8;

    let (band, streptococcus_rate, recommendation, interpretation) = match score {
        0 | 1 => (
            Band::NoAntibiotic,
            "13–18%",
            "No antibiotic prescribing",
            format!(
                "A score of {score} is associated with 13–18% isolation of streptococcus — \
close to background carriage rates. A no-prescribing strategy is appropriate \
after discussion with the patient."
            ),
        ),
        2 | 3 => (
            Band::DelayedAntibiotic,
            "34–40%",
            "Delayed antibiotic prescribing",
            format!(
                "A score of {score} is associated with 34–40% isolation of streptococcus. \
A delayed prescribing strategy is appropriate after discussion with the patient."
            ),
        ),
        _ => (
            Band::ImmediateAntibiotic,
            "62–65%",
            "Consider immediate antibiotic prescription",
            format!(
                "A score of {score} is associated with 62–65% isolation of streptococcus. \
Consider an immediate prescribing strategy if symptoms are severe, \
or a short delayed prescribing strategy (48 hours) may be appropriate."
            ),
        ),
    };

    FeverPainOutcome {
        score,
        band,
        streptococcus_rate,
        recommendation,
        interpretation,
    }
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: FeverPainInput) -> CalculationResponse {
    let o = compute(input);

    let mut working = Map::new();
    working.insert("score".into(), json!(o.score));
    working.insert("level".into(), json!(o.band.slug()));
    working.insert("fever_criterion".into(), json!(input.fever));
    working.insert("purulence_criterion".into(), json!(input.purulence));
    working.insert("attend_rapidly_criterion".into(), json!(input.attend_rapidly));
    working.insert("inflamed_criterion".into(), json!(input.inflamed_tonsils));
    working.insert("no_cough_criterion".into(), json!(input.absence_of_cough));
    working.insert("streptococcus_rate".into(), json!(o.streptococcus_rate));
    working.insert("prescribing_recommendation".into(), json!(o.recommendation));

    CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    }
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct FeverPain;

impl Calculator for FeverPain {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "FeverPAIN Score"
    }

    fn description(&self) -> &'static str {
        "Five-item score guiding antibiotic prescribing in acute sore throat \
(validated for adults and children aged 3+)."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "FeverPainInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["fever", "purulence", "attend_rapidly", "inflamed_tonsils", "absence_of_cough"],
            "properties": {
                "fever": { "type": "boolean", "description": "Fever in the last 24 hours" },
                "purulence": { "type": "boolean", "description": "Purulence (pus on the tonsils)" },
                "attend_rapidly": { "type": "boolean", "description": "Symptom onset within 3 days (≤ 3 days)" },
                "inflamed_tonsils": { "type": "boolean", "description": "Severely inflamed tonsils" },
                "absence_of_cough": { "type": "boolean", "description": "No cough or coryza" }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FeverPainInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        Ok(build_response(parsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(f: bool, p: bool, a: bool, i: bool, n: bool) -> FeverPainInput {
        FeverPainInput {
            fever: f,
            purulence: p,
            attend_rapidly: a,
            inflamed_tonsils: i,
            absence_of_cough: n,
        }
    }

    #[test]
    fn score_zero_is_no_antibiotic() {
        let o = compute(input(false, false, false, false, false));
        assert_eq!(o.score, 0);
        assert_eq!(o.band, Band::NoAntibiotic);
        assert_eq!(o.streptococcus_rate, "13–18%");
    }

    #[test]
    fn score_one_is_still_no_antibiotic() {
        let o = compute(input(true, false, false, false, false));
        assert_eq!(o.score, 1);
        assert_eq!(o.band, Band::NoAntibiotic);
    }

    #[test]
    fn score_two_and_three_are_delayed() {
        assert_eq!(compute(input(true, true, false, false, false)).band, Band::DelayedAntibiotic);
        assert_eq!(compute(input(true, true, true, false, false)).band, Band::DelayedAntibiotic);
    }

    #[test]
    fn score_four_and_five_are_immediate() {
        assert_eq!(compute(input(true, true, true, true, false)).band, Band::ImmediateAntibiotic);
        let o = compute(input(true, true, true, true, true));
        assert_eq!(o.score, 5);
        assert_eq!(o.band, Band::ImmediateAntibiotic);
        assert_eq!(o.streptococcus_rate, "62–65%");
    }

    #[test]
    fn response_carries_criteria_and_reference() {
        let r = build_response(input(true, false, true, false, true));
        assert_eq!(r.calculator, "feverpain");
        assert_eq!(r.result, json!(3));
        assert_eq!(r.working["level"], json!("delayed"));
        assert_eq!(r.working["fever_criterion"], json!(true));
        assert_eq!(r.working["purulence_criterion"], json!(false));
        assert!(r.reference.contains("Little P"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "fever": true, "purulence": true, "attend_rapidly": true,
            "inflamed_tonsils": true, "absence_of_cough": true
        });
        let dynamic = FeverPain.calculate(&value).unwrap();
        let typed = build_response(input(true, true, true, true, true));
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(FeverPain.calculate(&json!({ "fever": "yes" })).is_err());
    }
}
