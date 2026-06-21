//! NHFS - Nottingham Hip Fracture Score (Maxwell et al. 2008).
//!
//! A preoperative score predicting 30-day mortality after hip fracture surgery,
//! derived and validated in the British Journal of Anaesthesia (Maxwell MJ,
//! Moran CG, Moppett IK. 2008;101(4):511-517). Seven weighted criteria sum to a
//! total of 0-10; the score maps to an approximate predicted 30-day mortality,
//! and is widely used (Association of Anaesthetists guidance, UK National Hip
//! Fracture Database) to identify high-risk patients and inform shared
//! decision-making and perioperative planning.
//!
//! Two clinical subtleties are encoded here:
//! - Age is a single numeric input mapping to three mutually-exclusive bands
//!   (<=65 = 0, 66-85 = 3, >=86 = 4), so contradictory age inputs are impossible.
//! - Haemoglobin is taken in g/dL, because the original threshold is <=10 g/dL.
//!   UK laboratories routinely report haemoglobin in g/L, where the equivalent
//!   threshold is <=100 g/L; the schema flags this unit explicitly to avoid a
//!   ten-fold error.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "nhfs";

/// Primary citation.
pub const REFERENCE: &str =
    "Maxwell MJ, Moran CG, Moppett IK. Development and validation of a preoperative scoring system \
to predict 30 day mortality in patients undergoing hip fracture surgery. Br J Anaesth. \
2008;101(4):511-517. doi:10.1093/bja/aen236";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature. The scoring algorithm carries no
/// proprietary licence and is reproduced freely in clinical practice and
/// national audit (e.g. the UK National Hip Fracture Database).
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/bja/aen236",
};

/// Upper bound of the lower age band: at or below this, age scores 0 points.
pub const AGE_NONE_MAX: u8 = 65;
/// Upper bound of the middle age band (66-85 inclusive), scoring 3 points.
pub const AGE_MID_MAX: u8 = 85;
/// Points for the middle age band (66-85).
pub const AGE_MID_POINTS: u8 = 3;
/// Points for the upper age band (>=86).
pub const AGE_HIGH_POINTS: u8 = 4;
/// Haemoglobin at or below this (g/dL) scores 1 point.
pub const HB_THRESHOLD_G_DL: f64 = 10.0;
/// AMTS at or below this (out of 10) scores 1 point.
pub const AMTS_THRESHOLD: u8 = 6;
/// Highest AMTS value (out of 10).
pub const AMTS_MAX: u8 = 10;

/// NHFS inputs. Age, haemoglobin, and AMTS are numeric; their point bands are
/// derived, so the criteria cannot contradict the raw values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NhfsInput {
    /// Age in years.
    pub age: u8,
    /// Male sex (scores 1 point).
    pub male: bool,
    /// Admission haemoglobin in g/dL (NOT g/L). <=10 g/dL scores 1 point.
    pub haemoglobin: f64,
    /// Abbreviated Mental Test Score, 0-10. A score <=6 scores 1 point.
    pub amts: u8,
    /// Living in an institution (residential or nursing care) before admission.
    pub institutionalised: bool,
    /// Two or more comorbidities (more than one).
    pub comorbidities_two_or_more: bool,
    /// Active malignancy within the last 20 years.
    pub malignancy: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct NhfsOutcome {
    /// Total score (0-10).
    pub score: u8,
    /// Points contributed by the age band (0, 3, or 4).
    pub age_points: u8,
    /// Approximate predicted 30-day mortality (%) for this score.
    pub mortality_pct: f64,
    pub interpretation: String,
}

/// Points contributed by the age band.
fn age_points(age: u8) -> u8 {
    if age > AGE_MID_MAX {
        AGE_HIGH_POINTS
    } else if age > AGE_NONE_MAX {
        AGE_MID_POINTS
    } else {
        0
    }
}

/// Approximate predicted 30-day mortality (%) by total NHFS score (0-10), per
/// the published score-to-mortality mapping (Maxwell et al. 2008 logistic model,
/// as tabulated in subsequent validations).
fn mortality_pct(score: u8) -> f64 {
    const TABLE: [f64; 11] = [0.7, 1.1, 1.7, 2.7, 4.4, 6.9, 11.0, 16.0, 24.0, 34.0, 45.0];
    TABLE[score.min(10) as usize]
}

/// Pure scoring.
pub fn compute(input: &NhfsInput) -> Result<NhfsOutcome, CalcError> {
    if input.age > 120 {
        return Err(CalcError::InvalidInput(format!(
            "age {} is out of range (expected 0-120)",
            input.age
        )));
    }
    if input.amts > AMTS_MAX {
        return Err(CalcError::InvalidInput(format!(
            "amts {} is out of range (expected 0-{AMTS_MAX})",
            input.amts
        )));
    }
    if !input.haemoglobin.is_finite() || input.haemoglobin < 0.0 {
        return Err(CalcError::InvalidInput(format!(
            "haemoglobin {} is invalid (expected a non-negative number in g/dL)",
            input.haemoglobin
        )));
    }
    if input.haemoglobin > 30.0 {
        return Err(CalcError::InvalidInput(format!(
            "haemoglobin {} g/dL is implausibly high - did you enter g/L instead? (10 g/dL = 100 g/L)",
            input.haemoglobin
        )));
    }

    let age_points = age_points(input.age);
    let score = age_points
        + u8::from(input.male)
        + u8::from(input.haemoglobin <= HB_THRESHOLD_G_DL)
        + u8::from(input.amts <= AMTS_THRESHOLD)
        + u8::from(input.institutionalised)
        + u8::from(input.comorbidities_two_or_more)
        + u8::from(input.malignancy);

    let mortality_pct = mortality_pct(score);

    let interpretation = format!(
        "NHFS {score}/10: approximate predicted 30-day mortality {mortality_pct}% after hip \
fracture surgery. The score stratifies perioperative risk to inform shared decision-making, \
consent, and care planning; it is a population-level estimate, not an individual prognosis, and \
does not by itself contraindicate surgery."
    );

    Ok(NhfsOutcome {
        score,
        age_points,
        mortality_pct,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &NhfsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("age_points".into(), json!(o.age_points));
    working.insert("male".into(), json!(u8::from(input.male)));
    working.insert(
        "haemoglobin_low".into(),
        json!(u8::from(input.haemoglobin <= HB_THRESHOLD_G_DL)),
    );
    working.insert(
        "amts_impaired".into(),
        json!(u8::from(input.amts <= AMTS_THRESHOLD)),
    );
    working.insert("institutionalised".into(), json!(u8::from(input.institutionalised)));
    working.insert(
        "comorbidities_two_or_more".into(),
        json!(u8::from(input.comorbidities_two_or_more)),
    );
    working.insert("malignancy".into(), json!(u8::from(input.malignancy)));
    working.insert("predicted_30day_mortality_pct".into(), json!(o.mortality_pct));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Nhfs;

impl Calculator for Nhfs {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Nottingham Hip Fracture Score (NHFS)"
    }

    fn description(&self) -> &'static str {
        "Preoperative score (0-10) predicting 30-day mortality after hip fracture surgery."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let source = json!({
            "citation": "Maxwell MJ, Moran CG, Moppett IK. Br J Anaesth. 2008;101(4):511-517.",
            "url": "https://doi.org/10.1093/bja/aen236"
        });

        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "NhfsInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "male", "haemoglobin", "amts",
                "institutionalised", "comorbidities_two_or_more", "malignancy"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 120,
                    "description": "Age in years (<=65 scores 0, 66-85 scores 3, >=86 scores 4)",
                    "definition": {
                        "concept": "Age band",
                        "statement": "Age scores 0 points at or below 65, 3 points for 66-85, and 4 points at or above 86.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "male": {
                    "type": "boolean",
                    "description": "Male sex (scores 1 point)",
                    "definition": {
                        "concept": "Sex",
                        "statement": "Male sex scores 1 point; female scores 0.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "haemoglobin": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 30,
                    "description": "Admission haemoglobin in g/dL (NOT g/L); <=10 g/dL scores 1 point. UK labs usually report g/L, where the threshold is <=100 g/L.",
                    "definition": {
                        "concept": "Admission haemoglobin",
                        "statement": "Haemoglobin concentration on admission, scoring 1 point if at or below 10 g/dL.",
                        "caveats": "Unit is g/dL. UK laboratories typically report haemoglobin in g/L (10 g/dL = 100 g/L); entering a g/L value here would cause a ten-fold error.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "amts": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 10,
                    "description": "Abbreviated Mental Test Score, 0-10; <=6 scores 1 point (moderate-to-severe cognitive impairment)",
                    "definition": {
                        "concept": "Abbreviated Mental Test Score (AMTS)",
                        "statement": "Ten-item bedside cognitive screen (0-10); a score at or below 6 scores 1 point.",
                        "caveats": "The NHFS threshold (<=6) is lower than the usual AMTS impairment cut-off (<8), reflecting moderate-to-severe impairment.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "institutionalised": {
                    "type": "boolean",
                    "description": "Living in an institution (residential or nursing care) before admission (scores 1 point)",
                    "definition": {
                        "concept": "Institutionalised residence",
                        "statement": "Resident in an institution (residential or nursing care home) prior to admission.",
                        "includes": ["Residential care home", "Nursing home", "Long-stay/continuing care"],
                        "excludes": ["Living in own home, with or without a care package", "Sheltered/warden-controlled housing where the patient lives independently"],
                        "source": source,
                        "status": "draft"
                    }
                },
                "comorbidities_two_or_more": {
                    "type": "boolean",
                    "description": "Two or more comorbidities (more than one) (scores 1 point)",
                    "definition": {
                        "concept": "Number of comorbidities",
                        "statement": "Two or more comorbid conditions, scoring 1 point.",
                        "source": source,
                        "status": "draft"
                    }
                },
                "malignancy": {
                    "type": "boolean",
                    "description": "Active malignancy within the last 20 years (scores 1 point)",
                    "definition": {
                        "concept": "Malignancy",
                        "statement": "Active malignant disease within the last 20 years.",
                        "source": source,
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: NhfsInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(age: u8) -> NhfsInput {
        NhfsInput {
            age,
            male: false,
            haemoglobin: 13.0,
            amts: 10,
            institutionalised: false,
            comorbidities_two_or_more: false,
            malignancy: false,
        }
    }

    #[test]
    fn age_bands() {
        assert_eq!(age_points(65), 0);
        assert_eq!(age_points(66), 3);
        assert_eq!(age_points(85), 3);
        assert_eq!(age_points(86), 4);
        assert_eq!(age_points(50), 0);
    }

    #[test]
    fn worked_example() {
        // 88M, Hb 9.5 g/dL, AMTS 5, institutionalised, >=2 comorbidities, malignancy.
        // age 4 + male 1 + hb 1 + amts 1 + institution 1 + comorb 1 + malignancy 1 = 10.
        let i = NhfsInput {
            age: 88,
            male: true,
            haemoglobin: 9.5,
            amts: 5,
            institutionalised: true,
            comorbidities_two_or_more: true,
            malignancy: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 10);
        assert_eq!(o.age_points, 4);
        assert_eq!(o.mortality_pct, 45.0);
    }

    #[test]
    fn minimum_score_is_zero() {
        let o = compute(&base(60)).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.mortality_pct, 0.7);
    }

    #[test]
    fn hb_threshold() {
        // Exactly 10.0 g/dL scores the point; just above does not.
        let mut at = base(60);
        at.haemoglobin = 10.0;
        assert_eq!(compute(&at).unwrap().score, 1);

        let mut above = base(60);
        above.haemoglobin = 10.1;
        assert_eq!(compute(&above).unwrap().score, 0);
    }

    #[test]
    fn amts_threshold() {
        // AMTS 6 scores the point; AMTS 7 does not.
        let mut six = base(60);
        six.amts = 6;
        assert_eq!(compute(&six).unwrap().score, 1);

        let mut seven = base(60);
        seven.amts = 7;
        assert_eq!(compute(&seven).unwrap().score, 0);
    }

    #[test]
    fn each_boolean_adds_one() {
        let mut i = base(60);
        i.male = true;
        i.institutionalised = true;
        i.comorbidities_two_or_more = true;
        i.malignancy = true;
        // age 0 + 4 booleans = 4.
        assert_eq!(compute(&i).unwrap().score, 4);
        assert_eq!(compute(&i).unwrap().mortality_pct, 4.4);
    }

    #[test]
    fn mortality_mapping_is_monotonic() {
        let mut last = -1.0;
        for s in 0..=10u8 {
            let m = mortality_pct(s);
            assert!(m > last, "mortality must rise with score at {s}");
            last = m;
        }
        assert_eq!(mortality_pct(0), 0.7);
        assert_eq!(mortality_pct(10), 45.0);
    }

    #[test]
    fn rejects_out_of_range_amts() {
        let mut i = base(60);
        i.amts = 11;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_g_per_litre_mistake() {
        // 95 looks like g/L (i.e. 9.5 g/dL); flagged rather than silently scored.
        let mut i = base(60);
        i.haemoglobin = 95.0;
        let err = compute(&i).unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        assert!(Nhfs
            .calculate(&json!({
                "age": 70, "male": true, "haemoglobin": 12.0, "amts": 8,
                "institutionalised": false, "comorbidities_two_or_more": false,
                "malignancy": false, "extra": true
            }))
            .is_err());
    }

    #[test]
    fn missing_fields_are_rejected() {
        assert!(Nhfs
            .calculate(&json!({
                "age": 70, "male": true, "haemoglobin": 12.0, "amts": 8,
                "institutionalised": false, "comorbidities_two_or_more": false
            }))
            .is_err());
    }

    #[test]
    fn haemoglobin_definition_flags_unit() {
        let schema = Nhfs.input_schema();
        let caveats = schema["properties"]["haemoglobin"]["definition"]["caveats"]
            .as_str()
            .unwrap();
        assert!(caveats.contains("g/L"), "haemoglobin definition must flag the g/L unit pitfall");
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 78, "male": true, "haemoglobin": 9.0, "amts": 4,
            "institutionalised": false, "comorbidities_two_or_more": true,
            "malignancy": false
        });
        let typed = NhfsInput {
            age: 78,
            male: true,
            haemoglobin: 9.0,
            amts: 4,
            institutionalised: false,
            comorbidities_two_or_more: true,
            malignancy: false,
        };
        // age 3 + male 1 + hb 1 + amts 1 + comorb 1 = 7.
        let dynamic = Nhfs.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(7));
        assert_eq!(dynamic.working["predicted_30day_mortality_pct"], json!(16.0));
    }
}
