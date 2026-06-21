//! ABCD2 - 2-day stroke risk after a transient ischaemic attack (TIA).
//!
//! A 0-7 point score (Johnston et al., Lancet 2007) stratifying early stroke
//! risk after a suspected TIA. Several criteria are easy to get subtly wrong, so
//! they are derived from raw clinical inputs rather than asked as points:
//! - Age and blood pressure are numeric; the points (age >=60; systolic >=140 OR
//!   diastolic >=90) are computed, so contradictory point/value pairs are
//!   impossible.
//! - Clinical features are mutually exclusive: unilateral weakness scores 2 and
//!   subsumes any speech disturbance; speech disturbance scores 1 only in the
//!   ABSENCE of weakness. A single enum prevents double-counting.
//! - Duration is a single banded enum (>=60 min = 2; 10-59 min = 1; <10 min = 0).
//!
//! Important caveat encoded in the interpretation: NICE NG128 (2019) recommends
//! NOT using ABCD2 (or ABCD3) to assess subsequent stroke risk or to decide the
//! urgency of referral. Everyone with a suspected TIA should be referred for
//! specialist assessment within 24 hours regardless of score. This calculator is
//! provided for education and for settings where the score is still requested,
//! not to gate referral.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "abcd2";

/// Primary citation.
pub const REFERENCE: &str =
    "Johnston SC, Rothwell PM, Nguyen-Huynh MN, et al. Validation and refinement of scores to \
predict very early stroke risk after transient ischaemic attack. Lancet. 2007;369(9558):283-292. \
NICE NG128 advises against using ABCD2 to guide referral urgency.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1016/S0140-6736(07)60150-0",
};

/// Clinical features of the TIA (C). Mutually exclusive: unilateral weakness
/// takes precedence over (and subsumes) any speech disturbance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClinicalFeatures {
    /// Unilateral weakness (with or without speech disturbance): 2 points.
    UnilateralWeakness,
    /// Speech disturbance WITHOUT weakness: 1 point.
    SpeechDisturbance,
    /// Other features (e.g. isolated sensory or visual symptoms): 0 points.
    Other,
}

impl ClinicalFeatures {
    fn points(self) -> u8 {
        match self {
            ClinicalFeatures::UnilateralWeakness => 2,
            ClinicalFeatures::SpeechDisturbance => 1,
            ClinicalFeatures::Other => 0,
        }
    }
}

/// Duration of the TIA symptoms (D). Banded so the points cannot contradict the
/// raw duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Duration {
    /// 60 minutes or more: 2 points.
    #[serde(rename = "60min-or-more")]
    SixtyMinOrMore,
    /// 10 to 59 minutes: 1 point.
    #[serde(rename = "10-59min")]
    TenToFiftyNineMin,
    /// Under 10 minutes: 0 points.
    #[serde(rename = "under-10min")]
    UnderTenMin,
}

impl Duration {
    fn points(self) -> u8 {
        match self {
            Duration::SixtyMinOrMore => 2,
            Duration::TenToFiftyNineMin => 1,
            Duration::UnderTenMin => 0,
        }
    }
}

/// ABCD2 inputs. Age and blood pressure are numeric; their points are derived.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Abcd2Input {
    /// Age in years (A): scores 1 if >=60.
    pub age: u8,
    /// Systolic blood pressure at presentation, mmHg (B).
    pub systolic_bp: u16,
    /// Diastolic blood pressure at presentation, mmHg (B).
    pub diastolic_bp: u16,
    /// Clinical features of the TIA (C).
    pub clinical_features: ClinicalFeatures,
    /// Duration of symptoms (D).
    pub duration: Duration,
    /// Diabetes mellitus (D): scores 1.
    pub diabetes: bool,
}

/// Stroke-risk band (Johnston et al., Lancet 2007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Score 0-3: low risk (~1.0% 2-day stroke risk).
    Low,
    /// Score 4-5: moderate risk (~4.1% 2-day stroke risk).
    Moderate,
    /// Score 6-7: high risk (~8.1% 2-day stroke risk).
    High,
}

impl RiskBand {
    fn slug(self) -> &'static str {
        match self {
            RiskBand::Low => "low",
            RiskBand::Moderate => "moderate",
            RiskBand::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abcd2Outcome {
    /// Total score (0-7).
    pub score: u8,
    /// Point for age (0 or 1).
    pub age_points: u8,
    /// Point for blood pressure (0 or 1).
    pub bp_points: u8,
    /// Points for clinical features (0, 1, or 2).
    pub clinical_points: u8,
    /// Points for duration (0, 1, or 2).
    pub duration_points: u8,
    /// Point for diabetes (0 or 1).
    pub diabetes_points: u8,
    pub risk_band: RiskBand,
    pub interpretation: String,
}

fn age_points(age: u8) -> u8 {
    u8::from(age >= 60)
}

fn bp_points(systolic: u16, diastolic: u16) -> u8 {
    u8::from(systolic >= 140 || diastolic >= 90)
}

fn risk_band(score: u8) -> RiskBand {
    if score >= 6 {
        RiskBand::High
    } else if score >= 4 {
        RiskBand::Moderate
    } else {
        RiskBand::Low
    }
}

/// The NICE NG128 caveat appended to every interpretation.
const NICE_CAVEAT: &str = "NICE NG128 advises NOT using ABCD2 to assess subsequent stroke risk or \
to decide referral urgency: refer everyone with a suspected TIA for specialist assessment within 24 \
hours regardless of score.";

/// Pure scoring.
pub fn compute(input: &Abcd2Input) -> Result<Abcd2Outcome, CalcError> {
    let age_points = age_points(input.age);
    let bp_points = bp_points(input.systolic_bp, input.diastolic_bp);
    let clinical_points = input.clinical_features.points();
    let duration_points = input.duration.points();
    let diabetes_points = u8::from(input.diabetes);

    let score = age_points + bp_points + clinical_points + duration_points + diabetes_points;
    let band = risk_band(score);

    let risk_phrase = match band {
        RiskBand::Low => "low risk (about 1.0% 2-day stroke risk in the derivation cohorts)",
        RiskBand::Moderate => "moderate risk (about 4.1% 2-day stroke risk in the derivation cohorts)",
        RiskBand::High => "high risk (about 8.1% 2-day stroke risk in the derivation cohorts)",
    };

    let interpretation = format!("Score {score}: {risk_phrase}. {NICE_CAVEAT}");

    Ok(Abcd2Outcome {
        score,
        age_points,
        bp_points,
        clinical_points,
        duration_points,
        diabetes_points,
        risk_band: band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Abcd2Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("age_points".into(), json!(o.age_points));
    working.insert("blood_pressure_points".into(), json!(o.bp_points));
    working.insert("clinical_features_points".into(), json!(o.clinical_points));
    working.insert("duration_points".into(), json!(o.duration_points));
    working.insert("diabetes_points".into(), json!(o.diabetes_points));
    working.insert("risk_band".into(), json!(o.risk_band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Abcd2;

impl Calculator for Abcd2 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ABCD2 Score (Stroke Risk after TIA)"
    }

    fn description(&self) -> &'static str {
        "2-day stroke risk after a transient ischaemic attack. Note NICE NG128 advises against \
using ABCD2 to guide referral urgency."
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
            "title": "Abcd2Input",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "systolic_bp", "diastolic_bp", "clinical_features",
                "duration", "diabetes"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 16,
                    "maximum": 120,
                    "description": "Age in years (A); scores 1 if 60 or over"
                },
                "systolic_bp": {
                    "type": "integer",
                    "minimum": 50,
                    "maximum": 300,
                    "description": "Systolic BP at presentation, mmHg (B); the BP point scores 1 if systolic >=140 OR diastolic >=90"
                },
                "diastolic_bp": {
                    "type": "integer",
                    "minimum": 20,
                    "maximum": 200,
                    "description": "Diastolic BP at presentation, mmHg (B); the BP point scores 1 if systolic >=140 OR diastolic >=90"
                },
                "clinical_features": {
                    "type": "string",
                    "enum": ["unilateral-weakness", "speech-disturbance", "other"],
                    "description": "Clinical features of the TIA (C); unilateral weakness scores 2, speech disturbance without weakness scores 1",
                    "definition": {
                        "concept": "Clinical features (C)",
                        "statement": "The presenting deficit, scored on its most weighty feature.",
                        "includes": [
                            "unilateral-weakness: any unilateral motor weakness, with or without speech disturbance, scores 2",
                            "speech-disturbance: dysphasia/dysarthria WITHOUT any weakness scores 1",
                            "other: any other deficit (e.g. isolated sensory or visual symptoms) scores 0"
                        ],
                        "excludes": ["Do NOT add the weakness and speech points together: the categories are mutually exclusive and weakness subsumes speech."],
                        "source": { "citation": "Johnston SC et al. Lancet. 2007;369(9558):283-292.", "url": "https://doi.org/10.1016/S0140-6736(07)60150-0" },
                        "status": "draft"
                    }
                },
                "duration": {
                    "type": "string",
                    "enum": ["60min-or-more", "10-59min", "under-10min"],
                    "description": "Duration of symptoms (D); >=60 min scores 2, 10-59 min scores 1, <10 min scores 0",
                    "definition": {
                        "concept": "Duration (D)",
                        "statement": "Total duration of the TIA symptoms.",
                        "includes": [
                            "60min-or-more: symptoms lasting 60 minutes or longer scores 2",
                            "10-59min: symptoms lasting 10 to 59 minutes scores 1",
                            "under-10min: symptoms lasting under 10 minutes scores 0"
                        ],
                        "source": { "citation": "Johnston SC et al. Lancet. 2007;369(9558):283-292.", "url": "https://doi.org/10.1016/S0140-6736(07)60150-0" },
                        "status": "draft"
                    }
                },
                "diabetes": {
                    "type": "boolean",
                    "description": "Diabetes mellitus, type 1 or type 2 (D); scores 1",
                    "definition": {
                        "concept": "Diabetes mellitus (D)",
                        "statement": "Established type 1 or type 2 diabetes mellitus.",
                        "includes": ["Type 1 diabetes", "Type 2 diabetes", "On glucose-lowering treatment"],
                        "excludes": ["Pre-diabetes / impaired glucose tolerance does NOT count"],
                        "snomedEcl": "<< 73211009 |Diabetes mellitus (disorder)| MINUS << 714628002 |Prediabetes (finding)|",
                        "source": { "citation": "Johnston SC et al. Lancet. 2007;369(9558):283-292.", "url": "https://doi.org/10.1016/S0140-6736(07)60150-0" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Abcd2Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Abcd2Input {
        Abcd2Input {
            age: 50,
            systolic_bp: 120,
            diastolic_bp: 80,
            clinical_features: ClinicalFeatures::Other,
            duration: Duration::UnderTenMin,
            diabetes: false,
        }
    }

    #[test]
    fn all_negative_scores_zero() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_band, RiskBand::Low);
    }

    #[test]
    fn age_point() {
        assert_eq!(age_points(59), 0);
        assert_eq!(age_points(60), 1);
    }

    #[test]
    fn bp_point_systolic_or_diastolic() {
        assert_eq!(bp_points(139, 89), 0);
        assert_eq!(bp_points(140, 80), 1);
        assert_eq!(bp_points(120, 90), 1);
        assert_eq!(bp_points(140, 90), 1);
    }

    #[test]
    fn clinical_features_points() {
        assert_eq!(ClinicalFeatures::UnilateralWeakness.points(), 2);
        assert_eq!(ClinicalFeatures::SpeechDisturbance.points(), 1);
        assert_eq!(ClinicalFeatures::Other.points(), 0);
    }

    #[test]
    fn duration_points() {
        assert_eq!(Duration::SixtyMinOrMore.points(), 2);
        assert_eq!(Duration::TenToFiftyNineMin.points(), 1);
        assert_eq!(Duration::UnderTenMin.points(), 0);
    }

    #[test]
    fn risk_bands() {
        assert_eq!(risk_band(0), RiskBand::Low);
        assert_eq!(risk_band(3), RiskBand::Low);
        assert_eq!(risk_band(4), RiskBand::Moderate);
        assert_eq!(risk_band(5), RiskBand::Moderate);
        assert_eq!(risk_band(6), RiskBand::High);
        assert_eq!(risk_band(7), RiskBand::High);
    }

    #[test]
    fn maximum_score_is_seven() {
        let i = Abcd2Input {
            age: 75,
            systolic_bp: 160,
            diastolic_bp: 95,
            clinical_features: ClinicalFeatures::UnilateralWeakness,
            duration: Duration::SixtyMinOrMore,
            diabetes: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 7);
        assert_eq!(o.risk_band, RiskBand::High);
        // 1 + 1 + 2 + 2 + 1 = 7.
        assert_eq!(o.age_points, 1);
        assert_eq!(o.bp_points, 1);
        assert_eq!(o.clinical_points, 2);
        assert_eq!(o.duration_points, 2);
        assert_eq!(o.diabetes_points, 1);
    }

    #[test]
    fn worked_example_moderate() {
        // 65yo, BP 150/85, speech disturbance, 30 min, no diabetes:
        // age 1 + bp 1 + speech 1 + duration 1 + dm 0 = 4 -> moderate.
        let i = Abcd2Input {
            age: 65,
            systolic_bp: 150,
            diastolic_bp: 85,
            clinical_features: ClinicalFeatures::SpeechDisturbance,
            duration: Duration::TenToFiftyNineMin,
            diabetes: false,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.risk_band, RiskBand::Moderate);
    }

    #[test]
    fn weakness_subsumes_speech_no_double_count() {
        // Unilateral weakness scores 2, never 2 + 1.
        let mut i = base();
        i.clinical_features = ClinicalFeatures::UnilateralWeakness;
        let o = compute(&i).unwrap();
        assert_eq!(o.clinical_points, 2);
        assert_eq!(o.score, 2);
    }

    #[test]
    fn interpretation_includes_nice_caveat() {
        let o = compute(&base()).unwrap();
        assert!(o.interpretation.contains("NICE NG128"));
        assert!(o.interpretation.contains("within 24"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 65,
            "systolic_bp": 150,
            "diastolic_bp": 85,
            "clinical_features": "speech-disturbance",
            "duration": "10-59min",
            "diabetes": false
        });
        let typed = Abcd2Input {
            age: 65,
            systolic_bp: 150,
            diastolic_bp: 85,
            clinical_features: ClinicalFeatures::SpeechDisturbance,
            duration: Duration::TenToFiftyNineMin,
            diabetes: false,
        };
        let dynamic = Abcd2.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(4));
    }

    #[test]
    fn duration_enum_serde_renames() {
        let v: Duration = serde_json::from_value(json!("60min-or-more")).unwrap();
        assert_eq!(v, Duration::SixtyMinOrMore);
        let v: Duration = serde_json::from_value(json!("under-10min")).unwrap();
        assert_eq!(v, Duration::UnderTenMin);
    }

    #[test]
    fn rejects_malformed_input() {
        // Missing a required field.
        let value = json!({
            "age": 65, "systolic_bp": 150, "diastolic_bp": 85,
            "clinical_features": "other", "duration": "under-10min"
        });
        assert!(Abcd2.calculate(&value).is_err());
    }
}
