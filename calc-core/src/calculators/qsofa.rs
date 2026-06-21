//! qSOFA - quick Sequential (Sepsis-related) Organ Failure Assessment.
//!
//! A three-item bedside prompt from the Sepsis-3 consensus (Singer et al. JAMA
//! 2016). Each criterion scores one point; a total of 2 or more in a patient
//! with suspected infection flags a higher risk of poor outcome (in-hospital
//! mortality or a prolonged ICU stay) and should prompt assessment for organ
//! dysfunction.
//!
//! Two clinical subtleties are encoded here:
//! - The criteria are *derived* from raw observations (respiratory rate,
//!   systolic blood pressure, and whether the patient has altered mentation,
//!   i.e. a Glasgow Coma Scale score below 15), so the boundary conditions
//!   (>= 22, <= 100, GCS < 15) live in one place and cannot be transcribed
//!   wrongly per call site.
//! - qSOFA is a *prognostic prompt, not a diagnosis*. A positive score should
//!   trigger further assessment, not sepsis-directed treatment by itself, and
//!   the Surviving Sepsis Campaign 2021 guidelines recommend against using
//!   qSOFA as a sole screening tool (preferring NEWS2 or SIRS alongside it).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "qsofa";

/// Primary citation.
pub const REFERENCE: &str =
    "Singer M, Deutschman CS, Seymour CW, et al. The Third International Consensus Definitions for \
Sepsis and Septic Shock (Sepsis-3). JAMA. 2016;315(8):801-810.";

/// Distribution licence: the score is a published clinical method from the
/// open Sepsis-3 consensus, implemented here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (Sepsis-3 consensus)",
    source_url: "https://doi.org/10.1001/jama.2016.0287",
};

/// Threshold (inclusive) at or above which respiratory rate scores a point.
const RESP_RATE_THRESHOLD: f64 = 22.0;
/// Threshold (inclusive) at or below which systolic blood pressure scores a point.
const SYSTOLIC_BP_THRESHOLD: f64 = 100.0;

/// qSOFA inputs. The two physiological criteria are numeric; mentation is a
/// boolean asserting a Glasgow Coma Scale score below 15.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QsofaInput {
    /// Respiratory rate in breaths per minute. Scores a point at >= 22/min.
    pub respiratory_rate: f64,
    /// Systolic blood pressure in mmHg. Scores a point at <= 100 mmHg.
    pub systolic_bp: f64,
    /// Altered mentation: Glasgow Coma Scale score below 15. Scores a point.
    pub altered_mentation: bool,
}

/// Risk band implied by the score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Score 0-1: not flagged by qSOFA.
    LowRisk,
    /// Score 2-3: higher risk of poor outcome; prompts further assessment.
    HighRisk,
}

impl Band {
    /// Stable slug.
    pub fn slug(self) -> &'static str {
        match self {
            Band::LowRisk => "low-risk",
            Band::HighRisk => "high-risk",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsofaOutcome {
    /// Total score (0-3).
    pub score: u8,
    pub band: Band,
    /// Whether the respiratory-rate criterion was met (>= 22/min).
    pub respiratory_rate_criterion: bool,
    /// Whether the systolic-BP criterion was met (<= 100 mmHg).
    pub systolic_bp_criterion: bool,
    /// Whether the altered-mentation criterion was met (GCS < 15).
    pub altered_mentation_criterion: bool,
    pub interpretation: String,
}

/// Pure scoring. Derives each criterion from the raw observations.
pub fn compute(input: &QsofaInput) -> Result<QsofaInput, CalcError> {
    // Validate the physiological inputs are finite and physically plausible so a
    // stray NaN or negative reading can't silently produce a criterion result.
    if !input.respiratory_rate.is_finite() || input.respiratory_rate < 0.0 {
        return Err(CalcError::InvalidInput(
            "respiratory_rate must be a non-negative number".into(),
        ));
    }
    if !input.systolic_bp.is_finite() || input.systolic_bp < 0.0 {
        return Err(CalcError::InvalidInput(
            "systolic_bp must be a non-negative number".into(),
        ));
    }
    Ok(*input)
}

/// Score a validated set of inputs.
fn outcome(input: &QsofaInput) -> QsofaOutcome {
    let respiratory_rate_criterion = input.respiratory_rate >= RESP_RATE_THRESHOLD;
    let systolic_bp_criterion = input.systolic_bp <= SYSTOLIC_BP_THRESHOLD;
    let altered_mentation_criterion = input.altered_mentation;

    let score = u8::from(respiratory_rate_criterion)
        + u8::from(systolic_bp_criterion)
        + u8::from(altered_mentation_criterion);

    let band = if score >= 2 { Band::HighRisk } else { Band::LowRisk };

    let interpretation = match band {
        Band::HighRisk => format!(
            "Score {score}: qSOFA positive (>= 2). In a patient with suspected infection this flags \
a higher risk of poor outcome (in-hospital mortality or prolonged ICU stay) and should prompt \
assessment for organ dysfunction and closer monitoring. qSOFA is a prognostic prompt, not a \
diagnosis of sepsis, and does not by itself warrant sepsis-directed treatment (Sepsis-3)."
        ),
        Band::LowRisk => format!(
            "Score {score}: qSOFA negative (< 2). This does not exclude sepsis or rule out \
deterioration; reassess if the clinical picture changes. The Surviving Sepsis Campaign 2021 \
guidelines recommend against using qSOFA as a sole screening tool - use NEWS2 or SIRS alongside it."
        ),
    };

    QsofaOutcome {
        score,
        band,
        respiratory_rate_criterion,
        systolic_bp_criterion,
        altered_mentation_criterion,
        interpretation,
    }
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &QsofaInput) -> Result<CalculationResponse, CalcError> {
    let validated = compute(input)?;
    let o = outcome(&validated);

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("level".into(), json!(o.band.slug()));
    working.insert("respiratory_rate".into(), json!(input.respiratory_rate));
    working.insert("respiratory_rate_criterion".into(), json!(o.respiratory_rate_criterion));
    working.insert("systolic_bp".into(), json!(input.systolic_bp));
    working.insert("systolic_bp_criterion".into(), json!(o.systolic_bp_criterion));
    working.insert("altered_mentation_criterion".into(), json!(o.altered_mentation_criterion));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Qsofa;

impl Calculator for Qsofa {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "qSOFA Score (Sepsis-3)"
    }

    fn description(&self) -> &'static str {
        "Quick bedside prompt flagging suspected-infection patients at higher risk of poor outcome \
(Sepsis-3). A prognostic prompt, not a diagnosis of sepsis."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "QsofaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["respiratory_rate", "systolic_bp", "altered_mentation"],
            "properties": {
                "respiratory_rate": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 120,
                    "description": "Respiratory rate in breaths per minute (>= 22 scores 1 point)",
                    "definition": {
                        "concept": "Respiratory rate criterion",
                        "statement": "A respiratory rate of 22 breaths/min or greater scores 1 point.",
                        "caveats": "Use an accurately counted rate; transient tachypnoea (e.g. immediately after exertion or distress) may not reflect the underlying state.",
                        "source": { "citation": "Singer M et al. JAMA. 2016;315(8):801-810.", "url": "https://doi.org/10.1001/jama.2016.0287" },
                        "status": "draft"
                    }
                },
                "systolic_bp": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 300,
                    "description": "Systolic blood pressure in mmHg (<= 100 scores 1 point)",
                    "definition": {
                        "concept": "Systolic blood pressure criterion",
                        "statement": "A systolic blood pressure of 100 mmHg or less scores 1 point.",
                        "caveats": "qSOFA uses systolic BP, not mean arterial pressure; it is a prompt, not the hypotension threshold used to define septic shock.",
                        "source": { "citation": "Singer M et al. JAMA. 2016;315(8):801-810.", "url": "https://doi.org/10.1001/jama.2016.0287" },
                        "status": "draft"
                    }
                },
                "altered_mentation": {
                    "type": "boolean",
                    "description": "Altered mentation: Glasgow Coma Scale score below 15 (scores 1 point)",
                    "definition": {
                        "concept": "Altered mentation criterion",
                        "statement": "Any Glasgow Coma Scale score below 15 (i.e. not fully alert and oriented) scores 1 point.",
                        "includes": ["New confusion, drowsiness, or reduced GCS", "Any GCS < 15 attributable to the acute illness"],
                        "excludes": ["A stable chronic baseline of impaired consciousness should be interpreted with care - the criterion targets acute change"],
                        "caveats": "Sepsis-3 uses GCS < 15 as the threshold; the abstract prose mentions GCS < 14, but the scoring criterion is any deviation from a fully alert GCS of 15.",
                        "snomedEcl": "<< 419284004 |Altered level of consciousness (finding)|",
                        "source": { "citation": "Singer M et al. JAMA. 2016;315(8):801-810.", "url": "https://doi.org/10.1001/jama.2016.0287" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: QsofaInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> QsofaInput {
        // A fully reassuring set of observations: score 0.
        QsofaInput {
            respiratory_rate: 16.0,
            systolic_bp: 120.0,
            altered_mentation: false,
        }
    }

    fn score_of(input: &QsofaInput) -> u8 {
        outcome(&compute(input).unwrap()).score
    }

    #[test]
    fn all_normal_scores_zero() {
        let o = outcome(&compute(&base()).unwrap());
        assert_eq!(o.score, 0);
        assert_eq!(o.band, Band::LowRisk);
        assert!(!o.respiratory_rate_criterion);
        assert!(!o.systolic_bp_criterion);
        assert!(!o.altered_mentation_criterion);
    }

    #[test]
    fn respiratory_rate_threshold_is_inclusive() {
        let mut i = base();
        i.respiratory_rate = 21.0;
        assert_eq!(score_of(&i), 0);
        i.respiratory_rate = 22.0;
        assert_eq!(score_of(&i), 1);
        assert!(outcome(&compute(&i).unwrap()).respiratory_rate_criterion);
    }

    #[test]
    fn systolic_bp_threshold_is_inclusive() {
        let mut i = base();
        i.systolic_bp = 101.0;
        assert_eq!(score_of(&i), 0);
        i.systolic_bp = 100.0;
        assert_eq!(score_of(&i), 1);
        assert!(outcome(&compute(&i).unwrap()).systolic_bp_criterion);
    }

    #[test]
    fn altered_mentation_scores_one() {
        let mut i = base();
        i.altered_mentation = true;
        assert_eq!(score_of(&i), 1);
    }

    #[test]
    fn two_criteria_is_high_risk() {
        // RR 24 + SBP 90 = 2 points -> positive.
        let i = QsofaInput {
            respiratory_rate: 24.0,
            systolic_bp: 90.0,
            altered_mentation: false,
        };
        let o = outcome(&compute(&i).unwrap());
        assert_eq!(o.score, 2);
        assert_eq!(o.band, Band::HighRisk);
        assert!(o.interpretation.contains("positive"));
    }

    #[test]
    fn maximum_score_is_three() {
        let i = QsofaInput {
            respiratory_rate: 30.0,
            systolic_bp: 80.0,
            altered_mentation: true,
        };
        let o = outcome(&compute(&i).unwrap());
        assert_eq!(o.score, 3);
        assert_eq!(o.band, Band::HighRisk);
    }

    #[test]
    fn single_criterion_stays_low_risk() {
        let mut i = base();
        i.altered_mentation = true;
        let o = outcome(&compute(&i).unwrap());
        assert_eq!(o.score, 1);
        assert_eq!(o.band, Band::LowRisk);
        assert!(o.interpretation.contains("negative"));
    }

    #[test]
    fn rejects_non_finite_or_negative_observations() {
        let mut i = base();
        i.respiratory_rate = f64::NAN;
        assert!(compute(&i).is_err());

        let mut i = base();
        i.systolic_bp = -5.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn build_response_carries_criteria_and_reference() {
        let i = QsofaInput {
            respiratory_rate: 26.0,
            systolic_bp: 95.0,
            altered_mentation: false,
        };
        let r = build_response(&i).unwrap();
        assert_eq!(r.calculator, "qsofa");
        assert_eq!(r.result, json!(2));
        assert_eq!(r.working["level"], json!("high-risk"));
        assert_eq!(r.working["respiratory_rate_criterion"], json!(true));
        assert_eq!(r.working["systolic_bp_criterion"], json!(true));
        assert_eq!(r.working["altered_mentation_criterion"], json!(false));
        assert!(r.reference.contains("Singer M"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "respiratory_rate": 24.0,
            "systolic_bp": 90.0,
            "altered_mentation": true
        });
        let typed = QsofaInput {
            respiratory_rate: 24.0,
            systolic_bp: 90.0,
            altered_mentation: true,
        };
        let dynamic = Qsofa.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Qsofa.calculate(&json!({ "respiratory_rate": "fast" })).is_err());
    }

    #[test]
    fn mentation_definition_notes_gcs_threshold() {
        let schema = Qsofa.input_schema();
        let statement = schema["properties"]["altered_mentation"]["definition"]["statement"]
            .as_str()
            .unwrap();
        assert!(statement.contains("below 15"));
    }
}
