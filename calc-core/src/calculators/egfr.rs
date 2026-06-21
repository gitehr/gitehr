//! eGFR - estimated glomerular filtration rate, CKD-EPI 2021 creatinine equation.
//!
//! The race-free 2021 update (Inker et al., NEJM 2021), which removed the Black
//! coefficient present in the 2009 equation. Reports mL/min/1.73m^2 and the
//! CKD G-stage (G1-G5).
//!
//! Creatinine is accepted in mg/dL or umol/L; UK laboratories report umol/L,
//! and the equation is defined in mg/dL, so the unit is a required input rather
//! than assumed - a wrong unit silently changes the result by ~88x.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "egfr";

/// Primary citation.
pub const REFERENCE: &str =
    "Inker LA, Eneanya ND, Coresh J, et al. New creatinine- and cystatin C-based equations to \
estimate GFR without race. N Engl J Med. 2021;385(19):1737-1749. doi:10.1056/NEJMoa2102953";

/// umol/L per mg/dL for creatinine (molar mass 113.12 g/mol).
pub const UMOL_PER_MGDL: f64 = 88.4;

/// Sex, used to select the equation's creatinine coefficients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Unit the creatinine value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatinineUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// Inputs to the CKD-EPI 2021 creatinine equation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EgfrInput {
    /// Age in years (the equation is validated in adults, 18+).
    pub age: u8,
    pub sex: Sex,
    /// Serum creatinine, in the unit given by `creatinine_unit`.
    pub creatinine: f64,
    pub creatinine_unit: CreatinineUnit,
}

/// CKD G-stage by eGFR (KDIGO).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    G1,
    G2,
    G3a,
    G3b,
    G4,
    G5,
}

impl Stage {
    fn from_egfr(egfr: f64) -> Self {
        if egfr >= 90.0 {
            Stage::G1
        } else if egfr >= 60.0 {
            Stage::G2
        } else if egfr >= 45.0 {
            Stage::G3a
        } else if egfr >= 30.0 {
            Stage::G3b
        } else if egfr >= 15.0 {
            Stage::G4
        } else {
            Stage::G5
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Stage::G1 => "G1",
            Stage::G2 => "G2",
            Stage::G3a => "G3a",
            Stage::G3b => "G3b",
            Stage::G4 => "G4",
            Stage::G5 => "G5",
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            Stage::G1 => "normal or high",
            Stage::G2 => "mildly decreased",
            Stage::G3a => "mildly to moderately decreased",
            Stage::G3b => "moderately to severely decreased",
            Stage::G4 => "severely decreased",
            Stage::G5 => "kidney failure",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct EgfrOutcome {
    /// eGFR in mL/min/1.73m^2, rounded to a whole number.
    pub egfr: u32,
    pub stage: Stage,
    pub interpretation: String,
}

/// Pure scoring: the CKD-EPI 2021 creatinine equation.
pub fn compute(input: &EgfrInput) -> Result<EgfrOutcome, CalcError> {
    if input.creatinine <= 0.0 || !input.creatinine.is_finite() {
        return Err(CalcError::InvalidInput(
            "creatinine must be a positive number".into(),
        ));
    }
    if input.age < 18 {
        return Err(CalcError::InvalidInput(
            "CKD-EPI is validated in adults (age 18+)".into(),
        ));
    }

    // Normalise creatinine to mg/dL.
    let scr_mgdl = match input.creatinine_unit {
        CreatinineUnit::MgDl => input.creatinine,
        CreatinineUnit::UmolL => input.creatinine / UMOL_PER_MGDL,
    };

    let (kappa, alpha, female_factor) = match input.sex {
        Sex::Female => (0.7, -0.241, 1.012),
        Sex::Male => (0.9, -0.302, 1.0),
    };

    let ratio = scr_mgdl / kappa;
    let egfr_raw = 142.0
        * ratio.min(1.0).powf(alpha)
        * ratio.max(1.0).powf(-1.200)
        * 0.9938_f64.powi(input.age as i32)
        * female_factor;

    let egfr = egfr_raw.round() as u32;
    let stage = Stage::from_egfr(egfr_raw);

    let interpretation = format!(
        "eGFR {egfr} mL/min/1.73m2 ({} kidney function, CKD G-stage {}). CKD staging also \
requires albuminuria (ACR) for the full G/A classification, and a diagnosis of CKD requires the \
abnormality to persist for more than 3 months. eGFR is unreliable in acute kidney injury and at \
extremes of muscle mass.",
        stage.descriptor(),
        stage.slug()
    );

    Ok(EgfrOutcome {
        egfr,
        stage,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &EgfrInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("egfr_ml_min_1_73m2".into(), json!(o.egfr));
    working.insert("ckd_g_stage".into(), json!(o.stage.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.egfr),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Egfr;

impl Calculator for Egfr {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "eGFR (CKD-EPI 2021)"
    }

    fn description(&self) -> &'static str {
        "Estimated glomerular filtration rate from creatinine (race-free CKD-EPI 2021); reports CKD G-stage."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "EgfrInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "sex", "creatinine", "creatinine_unit"],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (validated in adults 18+)"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex used to select the creatinine coefficients",
                    "definition": {
                        "concept": "Sex for the CKD-EPI coefficient",
                        "statement": "The sex used to pick the equation's creatinine coefficients (kappa and alpha) and the female multiplier.",
                        "caveats": "This is the sex the original equation was fitted with; record-keeping for sex and gender is a separate concern.",
                        "source": {
                            "citation": "Inker LA et al. N Engl J Med. 2021;385(19):1737-1749.",
                            "url": "https://doi.org/10.1056/NEJMoa2102953"
                        },
                        "status": "draft"
                    }
                },
                "creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum creatinine value, in the unit given by creatinine_unit"
                },
                "creatinine_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the creatinine value",
                    "definition": {
                        "concept": "Creatinine unit",
                        "statement": "The CKD-EPI equation is defined in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 88.4 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~88x and the wrong label silently produces a grossly wrong eGFR"
                        ],
                        "source": {
                            "citation": "Inker LA et al. N Engl J Med. 2021;385(19):1737-1749.",
                            "url": "https://doi.org/10.1056/NEJMoa2102953"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: EgfrInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(age: u8, sex: Sex, creatinine: f64, unit: CreatinineUnit) -> EgfrInput {
        EgfrInput {
            age,
            sex,
            creatinine,
            creatinine_unit: unit,
        }
    }

    // Reference points computed directly from the published CKD-EPI 2021 equation.
    #[test]
    fn male_normal_creatinine() {
        // 50yo male, Scr 0.9 mg/dL (ratio = 1.0): 142 * 0.9938^50 = ~104.
        let o = compute(&input(50, Sex::Male, 0.9, CreatinineUnit::MgDl)).unwrap();
        assert!((o.egfr as i64 - 104).abs() <= 1, "expected ~104, got {}", o.egfr);
        assert_eq!(o.stage, Stage::G1);
    }

    #[test]
    fn female_normal_creatinine() {
        // 60yo female, Scr 0.8 mg/dL: ~84.
        let o = compute(&input(60, Sex::Female, 0.8, CreatinineUnit::MgDl)).unwrap();
        assert!((o.egfr as i64 - 84).abs() <= 1, "expected ~84, got {}", o.egfr);
        assert_eq!(o.stage, Stage::G2);
    }

    #[test]
    fn umol_per_l_matches_equivalent_mgdl() {
        // 0.9 mg/dL == 79.56 umol/L; both must give the same eGFR.
        let a = compute(&input(50, Sex::Male, 0.9, CreatinineUnit::MgDl)).unwrap();
        let b = compute(&input(50, Sex::Male, 0.9 * UMOL_PER_MGDL, CreatinineUnit::UmolL)).unwrap();
        assert_eq!(a.egfr, b.egfr);
    }

    #[test]
    fn low_egfr_stages_correctly() {
        // High creatinine -> low eGFR -> advanced stage.
        let o = compute(&input(70, Sex::Male, 3.0, CreatinineUnit::MgDl)).unwrap();
        assert!(o.egfr < 30, "expected advanced CKD, got {}", o.egfr);
        assert!(matches!(o.stage, Stage::G4 | Stage::G5));
    }

    #[test]
    fn stage_boundaries() {
        assert_eq!(Stage::from_egfr(90.0), Stage::G1);
        assert_eq!(Stage::from_egfr(89.9), Stage::G2);
        assert_eq!(Stage::from_egfr(60.0), Stage::G2);
        assert_eq!(Stage::from_egfr(59.0), Stage::G3a);
        assert_eq!(Stage::from_egfr(45.0), Stage::G3a);
        assert_eq!(Stage::from_egfr(44.0), Stage::G3b);
        assert_eq!(Stage::from_egfr(30.0), Stage::G3b);
        assert_eq!(Stage::from_egfr(29.0), Stage::G4);
        assert_eq!(Stage::from_egfr(15.0), Stage::G4);
        assert_eq!(Stage::from_egfr(14.0), Stage::G5);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(compute(&input(50, Sex::Male, 0.0, CreatinineUnit::MgDl)).is_err());
        assert!(compute(&input(50, Sex::Male, -1.0, CreatinineUnit::MgDl)).is_err());
        assert!(compute(&input(10, Sex::Male, 0.9, CreatinineUnit::MgDl)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "age": 50, "sex": "male", "creatinine": 0.9, "creatinine_unit": "mg/dL" });
        let dynamic = Egfr.calculate(&value).unwrap();
        let typed = build_response(&input(50, Sex::Male, 0.9, CreatinineUnit::MgDl)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn schema_flags_unit_exclusion() {
        let schema = Egfr.input_schema();
        let def = &schema["properties"]["creatinine_unit"]["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("88x"));
    }
}
