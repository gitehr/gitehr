//! UKELD - United Kingdom Model for End-Stage Liver Disease.
//!
//! `UKELD = 5.395*ln(INR) + 1.485*ln(creatinine) + 3.13*ln(bilirubin) -
//! 81.565*ln(sodium) + 435`, with creatinine and bilirubin in umol/L and sodium
//! in mmol/L. Result rounded to the nearest integer. Developed by Barber et al.
//! (2011) for the UK Liver Advisory Group and used by NHS Blood and Transplant
//! to set eligibility for the elective adult liver-transplant list.
//!
//! A UKELD of 49 corresponds to a ~9% predicted 1-year mortality and is the
//! minimum score required to be added to the UK liver-transplant waiting list.
//!
//! Crucially, UKELD is defined *natively in SI units* (umol/L and mmol/L), which
//! is exactly how UK laboratories report these values - unlike MELD, whose
//! published form is in mg/dL. Unit handling here therefore defaults to umol/L
//! and is optional: a caller working in mg/dL can label creatinine/bilirubin as
//! such and they are converted to umol/L before the formula. Sodium is always
//! mmol/L (the SI and conventional units coincide for sodium).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "ukeld";

/// Distribution licence: the UKELD equation is a published method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (UKELD, Barber et al. 2011)",
    source_url: "https://doi.org/10.1097/TP.0b013e318225db4d",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Barber K, Madden S, Allen J, et al. Elective liver transplant list mortality: development of a \
United Kingdom end-stage liver disease score. Transplantation. 2011;92(4):469-476. \
doi:10.1097/TP.0b013e318225db4d";

/// umol/L per mg/dL for creatinine (molar mass 113.12 g/mol).
pub const CREATININE_UMOL_PER_MGDL: f64 = 88.4;
/// umol/L per mg/dL for total bilirubin (molar mass 584.66 g/mol).
pub const BILIRUBIN_UMOL_PER_MGDL: f64 = 17.1;

/// Minimum UKELD score required to be added to the UK liver-transplant list,
/// corresponding to a ~9% predicted 1-year mortality.
pub const LISTING_THRESHOLD: i32 = 49;

/// Unit a creatinine or bilirubin value is expressed in.
///
/// UKELD is defined natively in umol/L (the default); mg/dL is offered as a
/// convenience for callers working in conventional units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Unit {
    #[default]
    #[serde(rename = "umol/L")]
    UmolL,
    #[serde(rename = "mg/dL")]
    MgDl,
}

/// Inputs to the UKELD score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UkeldInput {
    /// International normalised ratio (INR), dimensionless.
    pub inr: f64,
    /// Serum creatinine, in the unit given by `creatinine_unit` (default umol/L).
    pub creatinine: f64,
    #[serde(default)]
    pub creatinine_unit: Unit,
    /// Total serum bilirubin, in the unit given by `bilirubin_unit` (default umol/L).
    pub bilirubin: f64,
    #[serde(default)]
    pub bilirubin_unit: Unit,
    /// Serum sodium, in mmol/L.
    pub sodium: f64,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct UkeldOutcome {
    /// Final UKELD score, rounded to the nearest integer.
    pub score: i32,
    /// The raw formula output before rounding.
    pub raw_score: f64,
    /// Creatinine (umol/L) as used in the formula.
    pub creatinine_umol: f64,
    /// Bilirubin (umol/L) as used in the formula.
    pub bilirubin_umol: f64,
    /// True if the score is at or above the UK listing threshold (49).
    pub meets_listing_threshold: bool,
    pub interpretation: String,
}

/// Convert a creatinine or bilirubin value to umol/L given its unit.
fn to_umol(value: f64, unit: Unit, umol_per_mgdl: f64) -> f64 {
    match unit {
        Unit::UmolL => value,
        Unit::MgDl => value * umol_per_mgdl,
    }
}

/// Pure scoring: the UKELD equation.
pub fn compute(input: &UkeldInput) -> Result<UkeldOutcome, CalcError> {
    if input.inr <= 0.0 || input.creatinine <= 0.0 || input.bilirubin <= 0.0 || input.sodium <= 0.0 {
        return Err(CalcError::InvalidInput(
            "INR, creatinine, bilirubin, and sodium must be positive".into(),
        ));
    }
    if !input.inr.is_finite()
        || !input.creatinine.is_finite()
        || !input.bilirubin.is_finite()
        || !input.sodium.is_finite()
    {
        return Err(CalcError::InvalidInput("values must be finite".into()));
    }

    // Normalise creatinine and bilirubin to umol/L (UKELD's native unit).
    let creatinine_umol = to_umol(input.creatinine, input.creatinine_unit, CREATININE_UMOL_PER_MGDL);
    let bilirubin_umol = to_umol(input.bilirubin, input.bilirubin_unit, BILIRUBIN_UMOL_PER_MGDL);

    let raw_score = 5.395 * input.inr.ln()
        + 1.485 * creatinine_umol.ln()
        + 3.13 * bilirubin_umol.ln()
        - 81.565 * input.sodium.ln()
        + 435.0;

    let score = raw_score.round() as i32;
    let meets_listing_threshold = score >= LISTING_THRESHOLD;

    let interpretation = format!(
        "UKELD score {score}. The UK minimum score to be added to the elective adult \
liver-transplant list is {LISTING_THRESHOLD} (a UKELD of {LISTING_THRESHOLD} corresponds to a ~9% \
predicted 1-year mortality); this patient {} that threshold. Higher scores indicate greater \
waiting-list mortality risk. UKELD informs listing eligibility - the actual offer of an organ is \
decided by the Transplant Benefit Score, and exceptional cases may be listed via the National \
Appeals Panel below the threshold.",
        if meets_listing_threshold { "meets" } else { "does not meet" }
    );

    Ok(UkeldOutcome {
        score,
        raw_score,
        creatinine_umol,
        bilirubin_umol,
        meets_listing_threshold,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &UkeldInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("inr".into(), json!(input.inr));
    working.insert("creatinine_umol_used".into(), json!(o.creatinine_umol));
    working.insert("bilirubin_umol_used".into(), json!(o.bilirubin_umol));
    working.insert("sodium_mmol".into(), json!(input.sodium));
    working.insert("raw_score".into(), json!(o.raw_score));
    working.insert("ukeld_score".into(), json!(o.score));
    working.insert("listing_threshold".into(), json!(LISTING_THRESHOLD));
    working.insert("meets_listing_threshold".into(), json!(o.meets_listing_threshold));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Ukeld;

impl Calculator for Ukeld {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "UKELD (UK Model for End-Stage Liver Disease)"
    }

    fn description(&self) -> &'static str {
        "UK liver-transplant listing score from INR, creatinine, bilirubin, and sodium (Barber 2011); 49 is the listing threshold."
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
            "title": "UkeldInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["inr", "creatinine", "bilirubin", "sodium"],
            "properties": {
                "inr": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "International normalised ratio (INR), dimensionless"
                },
                "creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum creatinine, in the unit given by creatinine_unit (default umol/L)"
                },
                "creatinine_unit": {
                    "type": "string",
                    "enum": ["umol/L", "mg/dL"],
                    "default": "umol/L",
                    "description": "Unit of the creatinine value (defaults to umol/L)",
                    "definition": {
                        "concept": "Creatinine unit",
                        "statement": "UKELD is defined NATIVELY in umol/L - the unit UK laboratories report - unlike MELD, whose published form is mg/dL. The default here is umol/L; mg/dL is offered only as a convenience and is converted internally (1 mg/dL = 88.4 umol/L).",
                        "excludes": [
                            "Do NOT pass a mg/dL value while leaving the unit as the umol/L default: the two differ by ~88x and the wrong label silently produces a grossly wrong UKELD"
                        ],
                        "source": {
                            "citation": "Barber K et al. Transplantation. 2011;92(4):469-476.",
                            "url": "https://doi.org/10.1097/TP.0b013e318225db4d"
                        },
                        "status": "draft"
                    }
                },
                "bilirubin": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Total serum bilirubin, in the unit given by bilirubin_unit (default umol/L)"
                },
                "bilirubin_unit": {
                    "type": "string",
                    "enum": ["umol/L", "mg/dL"],
                    "default": "umol/L",
                    "description": "Unit of the bilirubin value (defaults to umol/L)",
                    "definition": {
                        "concept": "Bilirubin unit",
                        "statement": "UKELD is defined NATIVELY in umol/L - the unit UK laboratories report - unlike MELD, whose published form is mg/dL. The default here is umol/L; mg/dL is offered only as a convenience and is converted internally (1 mg/dL = 17.1 umol/L).",
                        "excludes": [
                            "Do NOT pass a mg/dL value while leaving the unit as the umol/L default: the two differ by ~17x and the wrong label silently produces a grossly wrong UKELD"
                        ],
                        "source": {
                            "citation": "Barber K et al. Transplantation. 2011;92(4):469-476.",
                            "url": "https://doi.org/10.1097/TP.0b013e318225db4d"
                        },
                        "status": "draft"
                    }
                },
                "sodium": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum sodium, in mmol/L",
                    "definition": {
                        "concept": "Serum sodium",
                        "statement": "Serum sodium in mmol/L. UKELD adds sodium to the MELD parameters because hyponatraemia is a potent predictor of waiting-list mortality in cirrhosis with ascites. For sodium the SI and conventional units coincide (mmol/L = mEq/L), so no conversion is offered.",
                        "source": {
                            "citation": "Barber K et al. Transplantation. 2011;92(4):469-476.",
                            "url": "https://doi.org/10.1097/TP.0b013e318225db4d"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: UkeldInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(inr: f64, creat: f64, creat_unit: Unit, bili: f64, bili_unit: Unit, na: f64) -> UkeldInput {
        UkeldInput {
            inr,
            creatinine: creat,
            creatinine_unit: creat_unit,
            bilirubin: bili,
            bilirubin_unit: bili_unit,
            sodium: na,
        }
    }

    #[test]
    fn worked_example_matches_known_ukeld() {
        // INR 1.0, creatinine 80 umol/L, bilirubin 20 umol/L, sodium 138 mmol/L.
        // 5.395*ln(1)        = 0
        // 1.485*ln(80)       = 6.5079
        // 3.13*ln(20)        = 9.3768
        // -81.565*ln(138)    = -401.8923
        // +435
        // raw ~= 48.99 -> 49.
        let o = compute(&input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).unwrap();
        assert!((o.raw_score - 48.99).abs() < 0.02, "raw {}", o.raw_score);
        assert_eq!(o.score, 49);
    }

    #[test]
    fn listing_threshold_flagged() {
        // The worked example sits exactly on the threshold of 49.
        let at = compute(&input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).unwrap();
        assert_eq!(at.score, LISTING_THRESHOLD);
        assert!(at.meets_listing_threshold);

        // Healthy bloods sit well below the threshold and do not meet it.
        let below = compute(&input(1.0, 70.0, Unit::UmolL, 10.0, Unit::UmolL, 142.0)).unwrap();
        assert!(below.score < LISTING_THRESHOLD, "score {}", below.score);
        assert!(!below.meets_listing_threshold);
    }

    #[test]
    fn umol_default_when_unit_omitted() {
        // Omitting the unit must default to umol/L, matching an explicit umol/L call.
        let value = json!({
            "inr": 1.0,
            "creatinine": 80.0,
            "bilirubin": 20.0,
            "sodium": 138.0
        });
        let defaulted: UkeldInput = serde_json::from_value(value).unwrap();
        assert_eq!(defaulted.creatinine_unit, Unit::UmolL);
        assert_eq!(defaulted.bilirubin_unit, Unit::UmolL);
        let explicit = input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0);
        assert_eq!(compute(&defaulted).unwrap(), compute(&explicit).unwrap());
    }

    #[test]
    fn mgdl_units_match_equivalent_umol() {
        // 80 umol/L creatinine == 80/88.4 mg/dL; 20 umol/L bilirubin == 20/17.1 mg/dL.
        let umol = compute(&input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).unwrap();
        let mgdl = compute(&input(
            1.0,
            80.0 / CREATININE_UMOL_PER_MGDL,
            Unit::MgDl,
            20.0 / BILIRUBIN_UMOL_PER_MGDL,
            Unit::MgDl,
            138.0,
        ))
        .unwrap();
        assert_eq!(umol.score, mgdl.score);
        assert!((umol.raw_score - mgdl.raw_score).abs() < 1e-9);
    }

    #[test]
    fn higher_score_with_worse_bloods() {
        // Worse synthetic function (high bilirubin, high INR, low sodium) raises UKELD.
        let mild = compute(&input(1.2, 90.0, Unit::UmolL, 40.0, Unit::UmolL, 136.0)).unwrap();
        let severe = compute(&input(2.5, 180.0, Unit::UmolL, 300.0, Unit::UmolL, 128.0)).unwrap();
        assert!(severe.score > mild.score, "{} vs {}", severe.score, mild.score);
        assert!(severe.meets_listing_threshold);
    }

    #[test]
    fn rejects_nonpositive_and_nonfinite() {
        assert!(compute(&input(0.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).is_err());
        assert!(compute(&input(1.0, 0.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).is_err());
        assert!(compute(&input(1.0, 80.0, Unit::UmolL, -1.0, Unit::UmolL, 138.0)).is_err());
        assert!(compute(&input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 0.0)).is_err());
        assert!(compute(&input(f64::NAN, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).is_err());
        assert!(compute(&input(1.0, 80.0, Unit::UmolL, f64::INFINITY, Unit::UmolL, 138.0)).is_err());
    }

    #[test]
    fn schema_flags_native_umol_and_unit_exclusions() {
        let schema = Ukeld.input_schema();
        let creat = &schema["properties"]["creatinine_unit"];
        assert_eq!(creat["default"], json!("umol/L"));
        assert!(creat["definition"]["statement"].as_str().unwrap().contains("NATIVELY in umol/L"));
        assert!(creat["definition"]["excludes"][0].as_str().unwrap().contains("88x"));
        let bili = &schema["properties"]["bilirubin_unit"];
        assert_eq!(bili["default"], json!("umol/L"));
        assert!(bili["definition"]["excludes"][0].as_str().unwrap().contains("17x"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "inr": 1.0,
            "creatinine": 80.0,
            "creatinine_unit": "umol/L",
            "bilirubin": 20.0,
            "bilirubin_unit": "umol/L",
            "sodium": 138.0
        });
        let dynamic = Ukeld.calculate(&value).unwrap();
        let typed = build_response(&input(1.0, 80.0, Unit::UmolL, 20.0, Unit::UmolL, 138.0)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(49));
        assert_eq!(dynamic.working["listing_threshold"], json!(LISTING_THRESHOLD));
    }
}
