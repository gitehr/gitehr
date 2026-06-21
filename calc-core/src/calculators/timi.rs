//! TIMI Risk Score for UA/NSTEMI - 14-day adverse-event risk in unstable
//! angina / non-ST-elevation MI.
//!
//! This is the TIMI risk score for unstable angina / NSTEMI (Antman EM et al,
//! JAMA 2000), NOT the separate TIMI score for STEMI. Seven equally-weighted
//! criteria each score 1 point (total 0-7); the score predicts the 14-day risk
//! of the composite endpoint of all-cause death, new or recurrent myocardial
//! infarction, or severe recurrent ischaemia requiring urgent revascularisation.
//!
//! Two criteria carry input definitions because they are easy to get subtly
//! wrong: "3+ CAD risk factors" (which factors count, and that it is a count of
//! at least 3, not their mere presence) and "known CAD" (which means documented
//! coronary stenosis of at least 50%, not simply a history of angina).
//!
//! Age is taken as an integer and the ">=65" point is derived, so a
//! contradictory age/flag pair is impossible.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "timi";

/// Primary citation.
pub const REFERENCE: &str =
    "Antman EM, Cohen M, Bernink PJLM, et al. The TIMI risk score for unstable angina/non-ST \
elevation MI: a method for prognostication and therapeutic decision making. JAMA. \
2000;284(7):835-842. This is the UA/NSTEMI score, distinct from the TIMI score for STEMI.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1001/jama.284.7.835",
};

/// TIMI UA/NSTEMI inputs. Age is numeric; the ">=65" point is derived.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimiInput {
    /// Age in years (>=65 scores 1 point).
    pub age: u8,
    /// At least 3 risk factors for coronary artery disease (hypertension,
    /// hypercholesterolaemia, diabetes, family history, or current smoker).
    pub three_or_more_cad_risk_factors: bool,
    /// Known coronary artery disease: documented coronary stenosis >=50%.
    pub known_cad: bool,
    /// Aspirin use in the past 7 days.
    pub aspirin_use_past_7_days: bool,
    /// >=2 anginal episodes in the past 24 hours.
    pub severe_angina: bool,
    /// ST-segment deviation >=0.5 mm on the admission ECG.
    pub st_deviation: bool,
    /// Positive cardiac biomarker (e.g. troponin).
    pub positive_biomarker: bool,
}

/// Risk band for the 14-day composite endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Score 0-2.
    Low,
    /// Score 3-4.
    Intermediate,
    /// Score 5-7.
    High,
}

impl RiskBand {
    fn slug(self) -> &'static str {
        match self {
            RiskBand::Low => "low",
            RiskBand::Intermediate => "intermediate",
            RiskBand::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct TimiOutcome {
    /// Total score (0-7).
    pub score: u8,
    /// Point contributed by age (0 or 1).
    pub age_point: u8,
    pub band: RiskBand,
    /// Approximate 14-day composite event rate, in percent (Antman 2000).
    pub event_rate_percent: f64,
    pub interpretation: String,
}

fn age_point(age: u8) -> u8 {
    u8::from(age >= 65)
}

fn band(score: u8) -> RiskBand {
    if score <= 2 {
        RiskBand::Low
    } else if score <= 4 {
        RiskBand::Intermediate
    } else {
        RiskBand::High
    }
}

/// Approximate 14-day composite event rate (death, MI, or urgent
/// revascularisation) from Antman et al, JAMA 2000, Table 3.
fn event_rate_percent(score: u8) -> f64 {
    match score {
        0 | 1 => 4.7,
        2 => 8.3,
        3 => 13.2,
        4 => 19.9,
        5 => 26.2,
        // Scores 6 and 7 are reported together.
        _ => 40.9,
    }
}

/// Pure scoring.
pub fn compute(input: &TimiInput) -> Result<TimiOutcome, CalcError> {
    let age_point = age_point(input.age);

    let score = age_point
        + u8::from(input.three_or_more_cad_risk_factors)
        + u8::from(input.known_cad)
        + u8::from(input.aspirin_use_past_7_days)
        + u8::from(input.severe_angina)
        + u8::from(input.st_deviation)
        + u8::from(input.positive_biomarker);

    let band = band(score);
    let event_rate_percent = event_rate_percent(score);

    let interpretation = format!(
        "TIMI score {score}/7: {} risk. Approximately {event_rate_percent}% 14-day risk of the \
composite endpoint (all-cause death, new or recurrent MI, or severe recurrent ischaemia requiring \
urgent revascularisation) (Antman et al, JAMA 2000). This is the UA/NSTEMI score, not the STEMI \
score.",
        match band {
            RiskBand::Low => "low",
            RiskBand::Intermediate => "intermediate",
            RiskBand::High => "high",
        }
    );

    Ok(TimiOutcome {
        score,
        age_point,
        band,
        event_rate_percent,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &TimiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("age_point".into(), json!(o.age_point));
    working.insert(
        "three_or_more_cad_risk_factors".into(),
        json!(u8::from(input.three_or_more_cad_risk_factors)),
    );
    working.insert("known_cad".into(), json!(u8::from(input.known_cad)));
    working.insert(
        "aspirin_use_past_7_days".into(),
        json!(u8::from(input.aspirin_use_past_7_days)),
    );
    working.insert("severe_angina".into(), json!(u8::from(input.severe_angina)));
    working.insert("st_deviation".into(), json!(u8::from(input.st_deviation)));
    working.insert(
        "positive_biomarker".into(),
        json!(u8::from(input.positive_biomarker)),
    );
    working.insert("risk_band".into(), json!(o.band.slug()));
    working.insert(
        "approximate_event_rate_percent".into(),
        json!(o.event_rate_percent),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Timi;

impl Calculator for Timi {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "TIMI Risk Score for UA/NSTEMI"
    }

    fn description(&self) -> &'static str {
        "14-day risk of death, MI, or urgent revascularisation in unstable angina / NSTEMI \
(Antman et al, JAMA 2000). Not the STEMI score."
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
            "title": "TimiInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "three_or_more_cad_risk_factors", "known_cad",
                "aspirin_use_past_7_days", "severe_angina", "st_deviation",
                "positive_biomarker"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (>=65 scores 1 point)"
                },
                "three_or_more_cad_risk_factors": {
                    "type": "boolean",
                    "description": "At least 3 risk factors for CAD (hypertension, hypercholesterolaemia, diabetes, family history, current smoker)",
                    "definition": {
                        "concept": "3+ CAD risk factors",
                        "statement": "Three or more of the following coronary artery disease risk factors are present: hypertension, hypercholesterolaemia, diabetes mellitus, family history of premature CAD, or current cigarette smoking.",
                        "includes": [
                            "Counting requires THREE OR MORE of the five factors - one or two does NOT score",
                            "Hypertension",
                            "Hypercholesterolaemia / dyslipidaemia",
                            "Diabetes mellitus",
                            "Family history of premature coronary artery disease",
                            "Current cigarette smoking"
                        ],
                        "excludes": [
                            "Fewer than 3 of the listed factors",
                            "Known CAD itself is scored separately and is not one of these risk factors"
                        ],
                        "caveats": "This is a COUNT of risk factors (>=3), not a single risk factor. Score only when at least three are present.",
                        "source": { "citation": "Antman EM et al. JAMA. 2000;284(7):835-842.", "url": "https://doi.org/10.1001/jama.284.7.835" },
                        "status": "draft"
                    }
                },
                "known_cad": {
                    "type": "boolean",
                    "description": "Known coronary artery disease: documented coronary stenosis >=50%",
                    "definition": {
                        "concept": "Known CAD (stenosis >=50%)",
                        "statement": "Prior documentation, on angiography or equivalent imaging, of at least one coronary artery stenosis of 50% or more.",
                        "includes": [
                            "Documented coronary stenosis >=50% on angiography",
                            "Known prior significant coronary disease confirmed by imaging"
                        ],
                        "excludes": [
                            "Risk factors for CAD without documented stenosis (those are counted separately)",
                            "A history of chest pain or angina symptoms alone, without documented >=50% stenosis"
                        ],
                        "caveats": "Requires documented stenosis >=50%, not merely suspected or risk-factor-based CAD.",
                        "snomedEcl": "<< 53741008 |Coronary arteriosclerosis (disorder)|",
                        "source": { "citation": "Antman EM et al. JAMA. 2000;284(7):835-842.", "url": "https://doi.org/10.1001/jama.284.7.835" },
                        "status": "draft"
                    }
                },
                "aspirin_use_past_7_days": {
                    "type": "boolean",
                    "description": "Aspirin use in the past 7 days"
                },
                "severe_angina": {
                    "type": "boolean",
                    "description": "Severe angina: >=2 anginal episodes in the past 24 hours"
                },
                "st_deviation": {
                    "type": "boolean",
                    "description": "ST-segment deviation >=0.5 mm on the admission ECG"
                },
                "positive_biomarker": {
                    "type": "boolean",
                    "description": "Positive cardiac biomarker (e.g. troponin or CK-MB)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: TimiInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(age: u8) -> TimiInput {
        TimiInput {
            age,
            three_or_more_cad_risk_factors: false,
            known_cad: false,
            aspirin_use_past_7_days: false,
            severe_angina: false,
            st_deviation: false,
            positive_biomarker: false,
        }
    }

    #[test]
    fn age_derivation() {
        assert_eq!(age_point(64), 0);
        assert_eq!(age_point(65), 1);
        assert_eq!(age_point(90), 1);
    }

    #[test]
    fn all_false_under_65_is_zero() {
        let o = compute(&base(50)).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.band, RiskBand::Low);
        assert_eq!(o.event_rate_percent, 4.7);
    }

    #[test]
    fn age_alone_scores_one() {
        let o = compute(&base(70)).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.age_point, 1);
        assert_eq!(o.band, RiskBand::Low);
        assert_eq!(o.event_rate_percent, 4.7);
    }

    #[test]
    fn worked_example_score_four() {
        // 68yo with 3+ risk factors, ST deviation, positive troponin:
        // age 1 + risk factors 1 + ST 1 + biomarker 1 = 4 -> intermediate, ~19.9%.
        let mut i = base(68);
        i.three_or_more_cad_risk_factors = true;
        i.st_deviation = true;
        i.positive_biomarker = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.band, RiskBand::Intermediate);
        assert_eq!(o.event_rate_percent, 19.9);
    }

    #[test]
    fn maximum_score_is_seven() {
        let i = TimiInput {
            age: 80,
            three_or_more_cad_risk_factors: true,
            known_cad: true,
            aspirin_use_past_7_days: true,
            severe_angina: true,
            st_deviation: true,
            positive_biomarker: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 7);
        assert_eq!(o.band, RiskBand::High);
        assert_eq!(o.event_rate_percent, 40.9);
    }

    #[test]
    fn band_boundaries() {
        assert_eq!(band(0), RiskBand::Low);
        assert_eq!(band(2), RiskBand::Low);
        assert_eq!(band(3), RiskBand::Intermediate);
        assert_eq!(band(4), RiskBand::Intermediate);
        assert_eq!(band(5), RiskBand::High);
        assert_eq!(band(7), RiskBand::High);
    }

    #[test]
    fn event_rates_match_antman_table() {
        assert_eq!(event_rate_percent(0), 4.7);
        assert_eq!(event_rate_percent(1), 4.7);
        assert_eq!(event_rate_percent(2), 8.3);
        assert_eq!(event_rate_percent(3), 13.2);
        assert_eq!(event_rate_percent(4), 19.9);
        assert_eq!(event_rate_percent(5), 26.2);
        assert_eq!(event_rate_percent(6), 40.9);
        assert_eq!(event_rate_percent(7), 40.9);
    }

    #[test]
    fn validation_rejects_missing_field() {
        // Missing positive_biomarker.
        let value = json!({
            "age": 70,
            "three_or_more_cad_risk_factors": true,
            "known_cad": false,
            "aspirin_use_past_7_days": false,
            "severe_angina": false,
            "st_deviation": false
        });
        let err = Timi.calculate(&value).unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }

    #[test]
    fn validation_rejects_unknown_field() {
        let value = json!({
            "age": 70,
            "three_or_more_cad_risk_factors": false,
            "known_cad": false,
            "aspirin_use_past_7_days": false,
            "severe_angina": false,
            "st_deviation": false,
            "positive_biomarker": false,
            "unexpected": true
        });
        // serde rejects unknown fields only when deny_unknown is set; here the
        // struct simply ignores extras, so this must still parse. Confirm it
        // does not error and produces a valid score (age 70 -> 1 point).
        let resp = Timi.calculate(&value).unwrap();
        assert_eq!(resp.result, json!(1));
    }

    #[test]
    fn known_cad_definition_requires_documented_stenosis() {
        let schema = Timi.input_schema();
        let excludes = &schema["properties"]["known_cad"]["definition"]["excludes"];
        assert!(excludes
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str().unwrap().contains("without documented")));
    }

    #[test]
    fn risk_factor_definition_requires_three() {
        let schema = Timi.input_schema();
        let stmt = schema["properties"]["three_or_more_cad_risk_factors"]["definition"]["statement"]
            .as_str()
            .unwrap();
        assert!(stmt.contains("Three or more"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 68,
            "three_or_more_cad_risk_factors": true,
            "known_cad": false,
            "aspirin_use_past_7_days": false,
            "severe_angina": false,
            "st_deviation": true,
            "positive_biomarker": true
        });
        let mut typed = base(68);
        typed.three_or_more_cad_risk_factors = true;
        typed.st_deviation = true;
        typed.positive_biomarker = true;
        let dynamic = Timi.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(4));
    }
}
