//! CURB-65 - severity assessment for community-acquired pneumonia.
//!
//! Stratifies 30-day mortality risk and guides the place of care (home vs
//! hospital vs ICU) in community-acquired pneumonia (Lim et al. Thorax 2003;
//! BTS/NICE NG138). Each of five criteria scores 1 point, total 0-5.
//!
//! The caller passes raw observations and the five criteria are derived here, so
//! the easy-to-misapply thresholds live in one place rather than at every call
//! site. Two subtleties are encoded:
//! - The urea threshold is in **mmol/L** (>7 mmol/L), not mg/dL. The original
//!   paper's equivalent is BUN >19 mg/dL; passing a mg/dL value as mmol/L would
//!   wrongly score almost everyone. The input is named and documented in mmol/L.
//! - Confusion means *new-onset* confusion (AMT <=8 or new disorientation), not
//!   a patient's chronic baseline cognitive impairment.
//!
//! The urea-free variant CRB-65 (confusion, respiratory rate, blood pressure,
//! age) is noted in the interpretation as the primary-care alternative when
//! bloods are unavailable; it is not computed by this calculator.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "curb65";

/// Primary citation.
pub const REFERENCE: &str =
    "Lim WS, van der Eerden MM, Laing R, et al. Defining community acquired pneumonia severity on \
presentation to hospital: an international derivation and validation study. Thorax. \
2003;58(5):377-382. Management thresholds per BTS / NICE NG138.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1136/thorax.58.5.377",
};

/// CURB-65 inputs. The five scoring criteria are derived from raw observations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Curb65Input {
    /// New-onset confusion (e.g. AMT <=8 or new disorientation in person, place,
    /// or time). NOT a chronic baseline cognitive impairment.
    pub confusion: bool,
    /// Serum urea in **mmol/L**. Scores a point when > 7 mmol/L.
    pub urea_mmol_l: f64,
    /// Respiratory rate in breaths/min. Scores a point when >= 30.
    pub respiratory_rate: f64,
    /// Systolic blood pressure in mmHg. Scores (with diastolic) when < 90.
    pub systolic_bp: f64,
    /// Diastolic blood pressure in mmHg. Scores (with systolic) when <= 60.
    pub diastolic_bp: f64,
    /// Age in years. Scores a point when >= 65.
    pub age: u8,
}

/// Risk band derived from the total score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Score 0-1: low severity, consider home treatment.
    Low,
    /// Score 2: intermediate severity, consider hospital assessment.
    Intermediate,
    /// Score 3-5: high severity, manage in hospital, consider ICU at 4-5.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Curb65Outcome {
    /// Total score (0-5).
    pub score: u8,
    /// Whether each criterion was met, in C-U-R-B-65 order.
    pub confusion: bool,
    pub urea: bool,
    pub respiratory_rate: bool,
    pub blood_pressure: bool,
    pub age: bool,
    pub risk_band: RiskBand,
    pub interpretation: String,
}

/// Approximate 30-day mortality for each score (Lim et al. Thorax 2003), as a
/// percentage string for the interpretation text.
fn mortality_pct(score: u8) -> &'static str {
    match score {
        0 => "0.7%",
        1 => "3.2%",
        2 => "13%",
        3 => "17%",
        4 => "41.5%",
        _ => "57%",
    }
}

/// Pure scoring.
pub fn compute(input: &Curb65Input) -> Result<Curb65Outcome, CalcError> {
    if !input.urea_mmol_l.is_finite()
        || !input.respiratory_rate.is_finite()
        || !input.systolic_bp.is_finite()
        || !input.diastolic_bp.is_finite()
    {
        return Err(CalcError::InvalidInput(
            "urea, respiratory rate, and blood pressure must be finite numbers".into(),
        ));
    }
    if input.urea_mmol_l < 0.0 || input.respiratory_rate < 0.0 {
        return Err(CalcError::InvalidInput(
            "urea and respiratory rate cannot be negative".into(),
        ));
    }
    if input.systolic_bp < 0.0 || input.diastolic_bp < 0.0 {
        return Err(CalcError::InvalidInput(
            "blood pressure cannot be negative".into(),
        ));
    }

    let confusion = input.confusion;
    let urea = input.urea_mmol_l > 7.0;
    let respiratory_rate = input.respiratory_rate >= 30.0;
    // Either limb of the BP criterion scores the (single) point.
    let blood_pressure = input.systolic_bp < 90.0 || input.diastolic_bp <= 60.0;
    let age = input.age >= 65;

    let score = u8::from(confusion)
        + u8::from(urea)
        + u8::from(respiratory_rate)
        + u8::from(blood_pressure)
        + u8::from(age);

    let risk_band = match score {
        0 | 1 => RiskBand::Low,
        2 => RiskBand::Intermediate,
        _ => RiskBand::High,
    };

    let mortality = mortality_pct(score);
    let interpretation = match risk_band {
        RiskBand::Low => format!(
            "Score {score}: low severity (approx. {mortality} 30-day mortality). Consider home \
treatment if clinically suitable (BTS / NICE NG138)."
        ),
        RiskBand::Intermediate => format!(
            "Score {score}: intermediate severity (approx. {mortality} 30-day mortality). Consider \
hospital-supervised treatment or a short inpatient stay, with close review (BTS / NICE NG138)."
        ),
        RiskBand::High => {
            let icu = if score >= 4 {
                " At a score of 4-5, assess for intensive care."
            } else {
                ""
            };
            format!(
                "Score {score}: high severity (approx. {mortality} 30-day mortality). Manage in \
hospital as severe pneumonia.{icu} (BTS / NICE NG138). Where bloods are unavailable, CRB-65 (the \
urea-free variant) can be used in primary care."
            )
        }
    };

    Ok(Curb65Outcome {
        score,
        confusion,
        urea,
        respiratory_rate,
        blood_pressure,
        age,
        risk_band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Curb65Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("confusion".into(), json!(u8::from(o.confusion)));
    working.insert("urea_gt_7_mmol_l".into(), json!(u8::from(o.urea)));
    working.insert("respiratory_rate_ge_30".into(), json!(u8::from(o.respiratory_rate)));
    working.insert("low_blood_pressure".into(), json!(u8::from(o.blood_pressure)));
    working.insert("age_ge_65".into(), json!(u8::from(o.age)));
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
pub struct Curb65;

impl Calculator for Curb65 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "CURB-65 Pneumonia Severity"
    }

    fn description(&self) -> &'static str {
        "Severity and 30-day mortality risk in community-acquired pneumonia, guiding place of care \
(BTS / NICE NG138)."
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
            "title": "Curb65Input",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "confusion", "urea_mmol_l", "respiratory_rate",
                "systolic_bp", "diastolic_bp", "age"
            ],
            "properties": {
                "confusion": {
                    "type": "boolean",
                    "description": "New-onset confusion (e.g. AMT <=8 or new disorientation) - NOT chronic baseline impairment (C)",
                    "definition": {
                        "concept": "Confusion (C)",
                        "statement": "New-onset mental confusion, operationalised in the original study as an Abbreviated Mental Test (AMT) score of 8 or less, or new disorientation in person, place, or time.",
                        "includes": ["AMT <=8 measured at presentation", "New disorientation in person, place, or time", "Acute confusion / delirium new since baseline"],
                        "excludes": ["A patient's chronic, pre-existing cognitive impairment or established dementia at their usual baseline does NOT count - the confusion must be NEW"],
                        "snomedEcl": "<< 40917007 |Clouded consciousness (finding)| OR << 130987000 |Acute confusion (finding)|",
                        "source": { "citation": "Lim WS et al. Thorax. 2003;58(5):377-382.", "url": "https://doi.org/10.1136/thorax.58.5.377" },
                        "status": "draft"
                    }
                },
                "urea_mmol_l": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Serum urea in mmol/L; scores 1 when > 7 mmol/L (U)",
                    "definition": {
                        "concept": "Urea (U)",
                        "statement": "Serum urea greater than 7 mmol/L scores 1 point.",
                        "caveats": "UNIT TRAP: the threshold is 7 mmol/L. The original paper's equivalent is blood urea nitrogen (BUN) > 19 mg/dL. These are different scales (urea mmol/L vs BUN mg/dL): supply this value in mmol/L. Passing a mg/dL figure here would score almost everyone.",
                        "snomedEcl": "<< 35591007 |Serum urea level - finding|",
                        "source": { "citation": "Lim WS et al. Thorax. 2003;58(5):377-382.", "url": "https://doi.org/10.1136/thorax.58.5.377" },
                        "status": "draft"
                    }
                },
                "respiratory_rate": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Respiratory rate in breaths/min; scores 1 when >= 30 (R)"
                },
                "systolic_bp": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Systolic BP in mmHg; the BP point scores when systolic < 90 OR diastolic <= 60 (B)",
                    "definition": {
                        "concept": "Blood pressure (B)",
                        "statement": "A single point for low blood pressure: systolic < 90 mmHg OR diastolic <= 60 mmHg.",
                        "caveats": "EITHER limb scores the one point (they are not separate points). Note the thresholds differ: systolic is strictly < 90, diastolic is <= 60.",
                        "snomedEcl": "<< 45007003 |Low blood pressure (disorder)|",
                        "source": { "citation": "Lim WS et al. Thorax. 2003;58(5):377-382.", "url": "https://doi.org/10.1136/thorax.58.5.377" },
                        "status": "draft"
                    }
                },
                "diastolic_bp": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Diastolic BP in mmHg; the BP point scores when systolic < 90 OR diastolic <= 60 (B)",
                    "definition": {
                        "concept": "Blood pressure (B)",
                        "statement": "A single point for low blood pressure: systolic < 90 mmHg OR diastolic <= 60 mmHg.",
                        "caveats": "EITHER limb scores the one point (they are not separate points). Note the thresholds differ: systolic is strictly < 90, diastolic is <= 60.",
                        "snomedEcl": "<< 45007003 |Low blood pressure (disorder)|",
                        "source": { "citation": "Lim WS et al. Thorax. 2003;58(5):377-382.", "url": "https://doi.org/10.1136/thorax.58.5.377" },
                        "status": "draft"
                    }
                },
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 120,
                    "description": "Age in years; scores 1 when >= 65 (65)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Curb65Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A patient with no criteria met.
    fn well() -> Curb65Input {
        Curb65Input {
            confusion: false,
            urea_mmol_l: 5.0,
            respiratory_rate: 16.0,
            systolic_bp: 120.0,
            diastolic_bp: 80.0,
            age: 40,
        }
    }

    #[test]
    fn all_criteria_absent_scores_zero() {
        let o = compute(&well()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_band, RiskBand::Low);
    }

    #[test]
    fn all_criteria_present_scores_five() {
        let i = Curb65Input {
            confusion: true,
            urea_mmol_l: 12.0,
            respiratory_rate: 34.0,
            systolic_bp: 80.0,
            diastolic_bp: 50.0,
            age: 80,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 5);
        assert_eq!(o.risk_band, RiskBand::High);
        assert!(o.interpretation.contains("intensive care"));
    }

    #[test]
    fn urea_threshold_is_strictly_greater_than_seven() {
        let mut i = well();
        i.urea_mmol_l = 7.0;
        assert_eq!(compute(&i).unwrap().score, 0, "7.0 mmol/L must NOT score");
        i.urea_mmol_l = 7.1;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert!(o.urea);
    }

    #[test]
    fn respiratory_rate_threshold_is_inclusive_thirty() {
        let mut i = well();
        i.respiratory_rate = 29.0;
        assert_eq!(compute(&i).unwrap().score, 0);
        i.respiratory_rate = 30.0;
        assert_eq!(compute(&i).unwrap().score, 1);
    }

    #[test]
    fn systolic_limb_of_bp_criterion() {
        let mut i = well();
        i.systolic_bp = 90.0;
        assert_eq!(compute(&i).unwrap().score, 0, "systolic 90 is not < 90");
        i.systolic_bp = 89.0;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert!(o.blood_pressure);
    }

    #[test]
    fn diastolic_limb_of_bp_criterion() {
        let mut i = well();
        i.diastolic_bp = 61.0;
        assert_eq!(compute(&i).unwrap().score, 0, "diastolic 61 is not <= 60");
        i.diastolic_bp = 60.0;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1, "diastolic 60 scores via the <= limb");
        assert!(o.blood_pressure);
    }

    #[test]
    fn either_bp_limb_scores_only_one_point() {
        let mut i = well();
        i.systolic_bp = 80.0;
        i.diastolic_bp = 50.0;
        // Both limbs true, but the BP criterion is worth one point only.
        assert_eq!(compute(&i).unwrap().score, 1);
    }

    #[test]
    fn age_threshold_is_inclusive_sixtyfive() {
        let mut i = well();
        i.age = 64;
        assert_eq!(compute(&i).unwrap().score, 0);
        i.age = 65;
        assert_eq!(compute(&i).unwrap().score, 1);
    }

    #[test]
    fn confusion_scores_directly() {
        let mut i = well();
        i.confusion = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert!(o.confusion);
    }

    #[test]
    fn risk_bands_by_score() {
        // Build scores 0..=5 by adding criteria cumulatively.
        let mut i = well();
        assert_eq!(compute(&i).unwrap().risk_band, RiskBand::Low); // 0
        i.confusion = true;
        assert_eq!(compute(&i).unwrap().risk_band, RiskBand::Low); // 1
        i.age = 70;
        assert_eq!(compute(&i).unwrap().risk_band, RiskBand::Intermediate); // 2
        i.urea_mmol_l = 9.0;
        assert_eq!(compute(&i).unwrap().risk_band, RiskBand::High); // 3
    }

    #[test]
    fn negative_observation_is_rejected() {
        let mut i = well();
        i.urea_mmol_l = -1.0;
        assert!(matches!(compute(&i), Err(CalcError::InvalidInput(_))));
    }

    #[test]
    fn non_finite_observation_is_rejected() {
        let mut i = well();
        i.respiratory_rate = f64::NAN;
        assert!(matches!(compute(&i), Err(CalcError::InvalidInput(_))));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "confusion": false,
            "urea_mmol_l": 9.0,
            "respiratory_rate": 32.0,
            "systolic_bp": 110.0,
            "diastolic_bp": 70.0,
            "age": 72
        });
        let typed = Curb65Input {
            confusion: false,
            urea_mmol_l: 9.0,
            respiratory_rate: 32.0,
            systolic_bp: 110.0,
            diastolic_bp: 70.0,
            age: 72,
        };
        let dynamic = Curb65.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        // urea + RR + age = 3.
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn urea_definition_flags_unit_trap() {
        let schema = Curb65.input_schema();
        let caveats = schema["properties"]["urea_mmol_l"]["definition"]["caveats"]
            .as_str()
            .unwrap();
        assert!(caveats.contains("mmol/L"));
        assert!(caveats.to_lowercase().contains("mg/dl"));
    }

    #[test]
    fn confusion_definition_requires_new_onset() {
        let schema = Curb65.input_schema();
        let excludes = &schema["properties"]["confusion"]["definition"]["excludes"];
        assert!(excludes[0].as_str().unwrap().contains("NEW"));
    }
}
