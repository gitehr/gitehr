//! MELD - Model for End-Stage Liver Disease (original 2001 version).
//!
//! `MELD = 3.78*ln(bilirubin) + 11.2*ln(INR) + 9.57*ln(creatinine) + 6.43`,
//! with bilirubin and creatinine in mg/dL. Developed by Kamath et al. (2001) to
//! predict short-term mortality in end-stage liver disease, and adopted by UNOS
//! in 2002 to prioritise liver-transplant allocation.
//!
//! The bounding rules matter as much as the formula. In order: each of
//! bilirubin, INR, and creatinine is floored at 1.0 (so no `ln` is ever
//! negative); creatinine is capped at 4.0 mg/dL; if the patient had dialysis
//! twice or more in the past week (or 24h of CVVHD) creatinine is set to 4.0;
//! and the final score is rounded to the nearest integer and clamped to 6-40
//! (UNOS).
//!
//! Bilirubin and creatinine are accepted in mg/dL or umol/L; UK laboratories
//! report umol/L, and the formula is defined in mg/dL, so the unit is a required
//! input rather than assumed - a wrong unit silently changes the result.
//!
//! This implements the *original* MELD. MELD-Na (adds serum sodium) and MELD 3.0
//! (adds sodium, albumin, and a sex term, with revised coefficients) are the
//! versions used for current UNOS/OPTN allocation; see the interpretation
//! caveat.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "meld";

/// Distribution licence: the MELD equation is a published method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (original MELD, 2001)",
    source_url: "https://doi.org/10.1053/jhep.2001.22172",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Kamath PS, Wiesner RH, Malinchoc M, et al. A model to predict survival in patients with \
end-stage liver disease. Hepatology. 2001;33(2):464-470. doi:10.1053/jhep.2001.22172. Bounding \
rules per UNOS/OPTN policy.";

/// umol/L per mg/dL for total bilirubin (molar mass 584.66 g/mol).
pub const BILIRUBIN_UMOL_PER_MGDL: f64 = 17.1;
/// umol/L per mg/dL for creatinine (molar mass 113.12 g/mol).
pub const CREATININE_UMOL_PER_MGDL: f64 = 88.4;

/// Lower bound applied to every input before the `ln` (keeps `ln` >= 0).
pub const INPUT_FLOOR: f64 = 1.0;
/// Upper cap applied to creatinine (mg/dL); also the dialysis substitute value.
pub const CREATININE_CAP: f64 = 4.0;
/// Lowest reportable MELD score (UNOS).
pub const SCORE_MIN: i32 = 6;
/// Highest reportable MELD score (UNOS).
pub const SCORE_MAX: i32 = 40;

/// Unit a laboratory value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// Inputs to the original MELD score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeldInput {
    /// Total serum bilirubin, in the unit given by `bilirubin_unit`.
    pub bilirubin: f64,
    pub bilirubin_unit: Unit,
    /// International normalised ratio (INR), dimensionless.
    pub inr: f64,
    /// Serum creatinine, in the unit given by `creatinine_unit`.
    pub creatinine: f64,
    pub creatinine_unit: Unit,
    /// True if the patient had dialysis twice or more in the past week, or 24h
    /// of continuous venovenous haemodialysis (CVVHD). Forces creatinine to 4.0.
    pub dialysis_twice_past_week: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct MeldOutcome {
    /// Final MELD score: rounded to the nearest integer and clamped to 6-40.
    pub score: i32,
    /// The raw formula output before rounding and clamping.
    pub raw_score: f64,
    /// Bilirubin (mg/dL) after flooring, as used in the formula.
    pub bilirubin_used: f64,
    /// INR after flooring, as used in the formula.
    pub inr_used: f64,
    /// Creatinine (mg/dL) after flooring, capping, and the dialysis rule.
    pub creatinine_used: f64,
    pub interpretation: String,
}

/// Floor a value at [`INPUT_FLOOR`].
fn floored(value: f64) -> f64 {
    value.max(INPUT_FLOOR)
}

/// Pure scoring: the original MELD equation with UNOS bounding rules.
pub fn compute(input: &MeldInput) -> Result<MeldOutcome, CalcError> {
    if input.bilirubin <= 0.0 || input.inr <= 0.0 || input.creatinine <= 0.0 {
        return Err(CalcError::InvalidInput(
            "bilirubin, INR, and creatinine must be positive".into(),
        ));
    }
    if !input.bilirubin.is_finite() || !input.inr.is_finite() || !input.creatinine.is_finite() {
        return Err(CalcError::InvalidInput("values must be finite".into()));
    }

    // 1. Normalise bilirubin and creatinine to mg/dL.
    let bilirubin_mgdl = match input.bilirubin_unit {
        Unit::MgDl => input.bilirubin,
        Unit::UmolL => input.bilirubin / BILIRUBIN_UMOL_PER_MGDL,
    };
    let creatinine_mgdl = match input.creatinine_unit {
        Unit::MgDl => input.creatinine,
        Unit::UmolL => input.creatinine / CREATININE_UMOL_PER_MGDL,
    };

    // 2. Floor each input at 1.0 so no ln is negative.
    let bilirubin_used = floored(bilirubin_mgdl);
    let inr_used = floored(input.inr);
    let mut creatinine_used = floored(creatinine_mgdl);

    // 3. Cap creatinine at 4.0 mg/dL, and 4. force it to 4.0 on dialysis.
    if input.dialysis_twice_past_week || creatinine_used > CREATININE_CAP {
        creatinine_used = CREATININE_CAP;
    }

    // 5. Apply the formula.
    let raw_score = 3.78 * bilirubin_used.ln()
        + 11.2 * inr_used.ln()
        + 9.57 * creatinine_used.ln()
        + 6.43;

    // 6. Round to the nearest integer and clamp to 6-40.
    let score = (raw_score.round() as i32).clamp(SCORE_MIN, SCORE_MAX);

    let interpretation = format!(
        "MELD score {score} (original 2001 model). Higher scores indicate greater 3-month \
mortality risk in end-stage liver disease; the score is bounded to {SCORE_MIN}-{SCORE_MAX} (UNOS). \
This is the original MELD - current UNOS/OPTN transplant allocation uses MELD-Na or MELD 3.0, \
which add serum sodium (and, for MELD 3.0, albumin and a sex term) and will differ from this value."
    );

    Ok(MeldOutcome {
        score,
        raw_score,
        bilirubin_used,
        inr_used,
        creatinine_used,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &MeldInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("bilirubin_mgdl_used".into(), json!(o.bilirubin_used));
    working.insert("inr_used".into(), json!(o.inr_used));
    working.insert("creatinine_mgdl_used".into(), json!(o.creatinine_used));
    working.insert("raw_score".into(), json!(o.raw_score));
    working.insert("meld_score".into(), json!(o.score));
    working.insert("score_min".into(), json!(SCORE_MIN));
    working.insert("score_max".into(), json!(SCORE_MAX));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Meld;

impl Calculator for Meld {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "MELD Score (original, 2001)"
    }

    fn description(&self) -> &'static str {
        "Model for End-Stage Liver Disease: 3-month mortality risk from bilirubin, INR, and creatinine (Kamath 2001)."
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
            "title": "MeldInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "bilirubin",
                "bilirubin_unit",
                "inr",
                "creatinine",
                "creatinine_unit",
                "dialysis_twice_past_week"
            ],
            "properties": {
                "bilirubin": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Total serum bilirubin, in the unit given by bilirubin_unit (floored to 1.0 mg/dL before the ln)"
                },
                "bilirubin_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the bilirubin value",
                    "definition": {
                        "concept": "Bilirubin unit",
                        "statement": "MELD is defined with total bilirubin in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 17.1 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~17x and the wrong label silently produces a grossly wrong MELD"
                        ],
                        "source": {
                            "citation": "Kamath PS et al. Hepatology. 2001;33(2):464-470.",
                            "url": "https://doi.org/10.1053/jhep.2001.22172"
                        },
                        "status": "draft"
                    }
                },
                "inr": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "International normalised ratio (INR), dimensionless (floored to 1.0 before the ln)"
                },
                "creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum creatinine, in the unit given by creatinine_unit (floored to 1.0 and capped at 4.0 mg/dL)"
                },
                "creatinine_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the creatinine value",
                    "definition": {
                        "concept": "Creatinine unit",
                        "statement": "MELD is defined with creatinine in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 88.4 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~88x and the wrong label silently produces a grossly wrong MELD"
                        ],
                        "source": {
                            "citation": "Kamath PS et al. Hepatology. 2001;33(2):464-470.",
                            "url": "https://doi.org/10.1053/jhep.2001.22172"
                        },
                        "status": "draft"
                    }
                },
                "dialysis_twice_past_week": {
                    "type": "boolean",
                    "description": "True if the patient had dialysis twice or more in the past week, or 24h of CVVHD; forces creatinine to 4.0 mg/dL",
                    "definition": {
                        "concept": "Dialysis substitution rule",
                        "statement": "Per UNOS/OPTN policy, two or more dialysis sessions in the prior 7 days (or 24h of continuous venovenous haemodialysis) sets creatinine to 4.0 mg/dL regardless of the measured value, reflecting that dialysis artificially lowers serum creatinine.",
                        "source": {
                            "citation": "OPTN Policy 9 (liver allocation); Kamath PS et al. Hepatology. 2001;33(2):464-470.",
                            "url": "https://doi.org/10.1053/jhep.2001.22172"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: MeldInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(bili: f64, bili_unit: Unit, inr: f64, creat: f64, creat_unit: Unit, dialysis: bool) -> MeldInput {
        MeldInput {
            bilirubin: bili,
            bilirubin_unit: bili_unit,
            inr,
            creatinine: creat,
            creatinine_unit: creat_unit,
            dialysis_twice_past_week: dialysis,
        }
    }

    #[test]
    fn worked_example_matches_known_meld() {
        // bilirubin 2.0, INR 1.5, creatinine 1.5 mg/dL, no dialysis.
        // 3.78*ln(2) + 11.2*ln(1.5) + 9.57*ln(1.5) + 6.43
        // = 2.6198 + 4.5408 + 3.8800 + 6.43 = 17.47 -> 17.
        let o = compute(&input(2.0, Unit::MgDl, 1.5, 1.5, Unit::MgDl, false)).unwrap();
        assert!((o.raw_score - 17.47).abs() < 0.05, "raw {}", o.raw_score);
        assert_eq!(o.score, 17);
    }

    #[test]
    fn values_below_one_are_floored() {
        // All inputs < 1.0 floor to 1.0; ln(1)=0, so score = round(6.43) = 6.
        let o = compute(&input(0.3, Unit::MgDl, 0.9, 0.5, Unit::MgDl, false)).unwrap();
        assert_eq!(o.bilirubin_used, 1.0);
        assert_eq!(o.inr_used, 1.0);
        assert_eq!(o.creatinine_used, 1.0);
        assert_eq!(o.raw_score, 6.43);
        assert_eq!(o.score, 6);
    }

    #[test]
    fn creatinine_capped_at_four() {
        // creatinine 8.0 mg/dL caps to 4.0 before the ln.
        let capped = compute(&input(2.0, Unit::MgDl, 1.5, 8.0, Unit::MgDl, false)).unwrap();
        let at_cap = compute(&input(2.0, Unit::MgDl, 1.5, 4.0, Unit::MgDl, false)).unwrap();
        assert_eq!(capped.creatinine_used, 4.0);
        assert_eq!(capped.score, at_cap.score);
        assert_eq!(capped.raw_score, at_cap.raw_score);
    }

    #[test]
    fn dialysis_forces_creatinine_to_four() {
        // A low measured creatinine plus dialysis is substituted with 4.0.
        let dial = compute(&input(2.0, Unit::MgDl, 1.5, 1.0, Unit::MgDl, true)).unwrap();
        let four = compute(&input(2.0, Unit::MgDl, 1.5, 4.0, Unit::MgDl, false)).unwrap();
        assert_eq!(dial.creatinine_used, 4.0);
        assert_eq!(dial.score, four.score);
        assert_eq!(dial.raw_score, four.raw_score);
    }

    #[test]
    fn score_clamped_to_six_minimum() {
        // Healthy values floor everything to 1.0: raw 6.43 -> still 6, never below.
        let o = compute(&input(0.5, Unit::MgDl, 1.0, 0.8, Unit::MgDl, false)).unwrap();
        assert_eq!(o.score, SCORE_MIN);
    }

    #[test]
    fn score_clamped_to_forty_maximum() {
        // Extreme values push the raw score above 40; it clamps to 40.
        let o = compute(&input(50.0, Unit::MgDl, 10.0, 4.0, Unit::MgDl, true)).unwrap();
        assert!(o.raw_score > 40.0, "raw {}", o.raw_score);
        assert_eq!(o.score, SCORE_MAX);
    }

    #[test]
    fn umol_units_match_equivalent_mgdl() {
        // 2.0 mg/dL bilirubin == 34.2 umol/L; 1.5 mg/dL creatinine == 132.6 umol/L.
        let mgdl = compute(&input(2.0, Unit::MgDl, 1.5, 1.5, Unit::MgDl, false)).unwrap();
        let umol = compute(&input(
            2.0 * BILIRUBIN_UMOL_PER_MGDL,
            Unit::UmolL,
            1.5,
            1.5 * CREATININE_UMOL_PER_MGDL,
            Unit::UmolL,
            false,
        ))
        .unwrap();
        assert_eq!(mgdl.score, umol.score);
        assert!((mgdl.raw_score - umol.raw_score).abs() < 1e-9);
    }

    #[test]
    fn rejects_nonpositive_and_nonfinite() {
        assert!(compute(&input(0.0, Unit::MgDl, 1.5, 1.5, Unit::MgDl, false)).is_err());
        assert!(compute(&input(2.0, Unit::MgDl, 0.0, 1.5, Unit::MgDl, false)).is_err());
        assert!(compute(&input(2.0, Unit::MgDl, 1.5, -1.0, Unit::MgDl, false)).is_err());
        assert!(compute(&input(f64::NAN, Unit::MgDl, 1.5, 1.5, Unit::MgDl, false)).is_err());
    }

    #[test]
    fn schema_flags_unit_exclusions() {
        let schema = Meld.input_schema();
        let bili = &schema["properties"]["bilirubin_unit"]["definition"];
        assert!(bili["excludes"][0].as_str().unwrap().contains("17x"));
        let creat = &schema["properties"]["creatinine_unit"]["definition"];
        assert!(creat["excludes"][0].as_str().unwrap().contains("88x"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "bilirubin": 2.0,
            "bilirubin_unit": "mg/dL",
            "inr": 1.5,
            "creatinine": 1.5,
            "creatinine_unit": "mg/dL",
            "dialysis_twice_past_week": false
        });
        let dynamic = Meld.calculate(&value).unwrap();
        let typed = build_response(&input(2.0, Unit::MgDl, 1.5, 1.5, Unit::MgDl, false)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(17));
    }
}
