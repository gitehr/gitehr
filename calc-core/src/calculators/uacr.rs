//! uACR - urine albumin-to-creatinine ratio, with KDIGO albuminuria (A) staging.
//!
//! The ratio is urine albumin divided by urine creatinine. UK laboratories
//! report it in mg/mmol; US laboratories report mg/g. The two differ by a factor
//! of ~8.84 (1 mg/mmol = 8.84 mg/g), so the unit is a required input rather than
//! assumed - a wrong unit silently shifts the KDIGO A-stage.
//!
//! Inputs accept either a directly-measured ratio (`acr` + `acr_unit`), or the
//! raw albumin and creatinine measurements (`albumin` mg/L + `creatinine`
//! mmol/L), from which the ratio is computed. The result is the ratio normalised
//! to mg/mmol plus the KDIGO albuminuria category (A1/A2/A3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "uacr";

/// Distribution licence: the KDIGO albuminuria categories (A1-A3) are a published
/// staging method; the thresholds and the ACR itself are implemented here from
/// the primary guideline. The KDIGO guideline text is CC BY-NC-ND, but the
/// numeric staging thresholds are facts/method rather than copyrightable content.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary guideline (KDIGO 2024 albuminuria categories A1-A3)",
    source_url: "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Kidney Disease: Improving Global Outcomes (KDIGO) CKD Work Group. KDIGO 2024 Clinical \
Practice Guideline for the Evaluation and Management of Chronic Kidney Disease. Kidney Int. \
2024;105(4S):S117-S314. doi:10.1016/j.kint.2023.10.018";

/// mg/g per mg/mmol for the albumin-to-creatinine ratio.
///
/// Creatinine molar mass is 113.12 g/mol, so 1 mmol = 113.12 mg and
/// 1 mg/mmol = 1000 / 113.12 = ~8.84 mg/g.
pub const MGG_PER_MGMMOL: f64 = 8.84;

/// Unit a directly-measured ACR value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcrUnit {
    #[serde(rename = "mg/mmol")]
    MgMmol,
    #[serde(rename = "mg/g")]
    MgG,
}

/// Inputs to the uACR calculation: either a measured ratio, or albumin and
/// creatinine to compute it from.
///
/// Exactly one of the two forms must be supplied. `acr` (+ `acr_unit`) takes a
/// ratio already reported by the laboratory; `albumin` (+ `creatinine`) takes
/// the raw measurements (albumin mg/L, creatinine mmol/L) and divides them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UacrInput {
    /// A directly-measured albumin-to-creatinine ratio, in the unit given by
    /// `acr_unit`. Mutually exclusive with `albumin`/`creatinine`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acr: Option<f64>,
    /// Unit of `acr`. Required when `acr` is given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acr_unit: Option<AcrUnit>,
    /// Urine albumin in mg/L. Supplied with `creatinine` instead of `acr`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub albumin: Option<f64>,
    /// Urine creatinine in mmol/L. Supplied with `albumin` instead of `acr`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creatinine: Option<f64>,
}

/// KDIGO albuminuria category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    A1,
    A2,
    A3,
}

impl Stage {
    /// Category from the ratio in mg/mmol (KDIGO: A1 <3, A2 3-30, A3 >30).
    ///
    /// The 3 and 30 boundaries are inclusive of A2 (KDIGO defines A2 as
    /// 3-30 mg/mmol), so values exactly at a boundary stage as the more
    /// abnormal of the adjacent pair only above it.
    fn from_acr_mgmmol(acr: f64) -> Self {
        if acr < 3.0 {
            Stage::A1
        } else if acr <= 30.0 {
            Stage::A2
        } else {
            Stage::A3
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Stage::A1 => "A1",
            Stage::A2 => "A2",
            Stage::A3 => "A3",
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            Stage::A1 => "normal to mildly increased",
            Stage::A2 => "moderately increased",
            Stage::A3 => "severely increased",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct UacrOutcome {
    /// ACR in mg/mmol, rounded to one decimal place.
    pub acr_mgmmol: f64,
    pub stage: Stage,
    pub interpretation: String,
}

/// Pure calculation: normalise the ratio to mg/mmol and assign the KDIGO stage.
pub fn compute(input: &UacrInput) -> Result<UacrOutcome, CalcError> {
    // Resolve the ratio in mg/mmol from whichever input form was supplied.
    let has_ratio = input.acr.is_some();
    let has_pair = input.albumin.is_some() || input.creatinine.is_some();

    let acr_mgmmol = match (has_ratio, has_pair) {
        (true, true) => {
            return Err(CalcError::InvalidInput(
                "supply either acr (+ acr_unit) or albumin + creatinine, not both".into(),
            ));
        }
        (false, false) => {
            return Err(CalcError::InvalidInput(
                "supply either acr (+ acr_unit) or albumin + creatinine".into(),
            ));
        }
        (true, false) => {
            let acr = input.acr.unwrap();
            let unit = input.acr_unit.ok_or_else(|| {
                CalcError::InvalidInput("acr_unit is required when acr is given".into())
            })?;
            if acr < 0.0 || !acr.is_finite() {
                return Err(CalcError::InvalidInput(
                    "acr must be a non-negative number".into(),
                ));
            }
            match unit {
                AcrUnit::MgMmol => acr,
                AcrUnit::MgG => acr / MGG_PER_MGMMOL,
            }
        }
        (false, true) => {
            let albumin = input.albumin.ok_or_else(|| {
                CalcError::InvalidInput("albumin is required alongside creatinine".into())
            })?;
            let creatinine = input.creatinine.ok_or_else(|| {
                CalcError::InvalidInput("creatinine is required alongside albumin".into())
            })?;
            if albumin < 0.0 || !albumin.is_finite() {
                return Err(CalcError::InvalidInput(
                    "albumin must be a non-negative number (mg/L)".into(),
                ));
            }
            if creatinine <= 0.0 || !creatinine.is_finite() {
                return Err(CalcError::InvalidInput(
                    "creatinine must be a positive number (mmol/L)".into(),
                ));
            }
            // albumin mg/L divided by creatinine mmol/L gives mg/mmol directly.
            albumin / creatinine
        }
    };

    let stage = Stage::from_acr_mgmmol(acr_mgmmol);
    let acr_rounded = (acr_mgmmol * 10.0).round() / 10.0;

    let interpretation = format!(
        "uACR {acr_rounded} mg/mmol ({} albuminuria, KDIGO category {}). Albuminuria categories \
combine with eGFR (G-stage) for the full CKD G/A classification; a diagnosis of CKD requires the \
abnormality to persist for more than 3 months. ACR can be transiently raised by urinary tract \
infection, menstrual contamination, recent vigorous exercise, or marked hyperglycaemia, so a \
raised result is normally confirmed on a repeat early-morning sample.",
        stage.descriptor(),
        stage.slug()
    );

    Ok(UacrOutcome {
        acr_mgmmol: acr_rounded,
        stage,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &UacrInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("acr_mg_mmol".into(), json!(o.acr_mgmmol));
    working.insert("kdigo_a_stage".into(), json!(o.stage.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.acr_mgmmol),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Uacr;

impl Calculator for Uacr {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "uACR (urine albumin-to-creatinine ratio)"
    }

    fn description(&self) -> &'static str {
        "Urine albumin-to-creatinine ratio from a measured ratio or raw albumin/creatinine; reports the KDIGO albuminuria category (A1-A3)."
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
            "title": "UacrInput",
            "type": "object",
            "additionalProperties": false,
            "description": "Supply EITHER a measured ratio (acr + acr_unit) OR raw measurements (albumin + creatinine).",
            "oneOf": [
                {
                    "required": ["acr", "acr_unit"],
                    "not": { "anyOf": [ { "required": ["albumin"] }, { "required": ["creatinine"] } ] }
                },
                {
                    "required": ["albumin", "creatinine"],
                    "not": { "required": ["acr"] }
                }
            ],
            "properties": {
                "acr": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Directly-measured albumin-to-creatinine ratio, in the unit given by acr_unit"
                },
                "acr_unit": {
                    "type": "string",
                    "enum": ["mg/mmol", "mg/g"],
                    "description": "Unit of the acr value",
                    "definition": {
                        "concept": "ACR unit",
                        "statement": "KDIGO staging is defined in mg/mmol (A1 <3, A2 3-30, A3 >30). UK laboratories report mg/mmol; US laboratories report mg/g. A mg/g value is converted internally (1 mg/mmol = 8.84 mg/g).",
                        "excludes": [
                            "Do NOT pass a mg/g value while labelling it mg/mmol: the two differ by ~8.84x and the wrong label silently changes the KDIGO A-stage"
                        ],
                        "source": {
                            "citation": "KDIGO 2024 CKD Guideline. Kidney Int. 2024;105(4S):S117-S314.",
                            "url": "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf"
                        },
                        "status": "draft"
                    }
                },
                "albumin": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Urine albumin in mg/L (supplied with creatinine instead of acr)",
                    "definition": {
                        "concept": "Urine albumin",
                        "statement": "Urine albumin concentration in mg/L. Dividing by urine creatinine in mmol/L yields the ACR directly in mg/mmol.",
                        "excludes": [
                            "Do NOT supply albumin in mg/dL or g/L; the value must be mg/L so that mg/L divided by mmol/L gives mg/mmol"
                        ],
                        "source": {
                            "citation": "KDIGO 2024 CKD Guideline. Kidney Int. 2024;105(4S):S117-S314.",
                            "url": "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf"
                        },
                        "status": "draft"
                    }
                },
                "creatinine": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Urine creatinine in mmol/L (supplied with albumin instead of acr)",
                    "definition": {
                        "concept": "Urine creatinine",
                        "statement": "Urine creatinine concentration in mmol/L, the denominator of the ratio. UK laboratories report mmol/L; mg/L of albumin over mmol/L of creatinine gives mg/mmol.",
                        "excludes": [
                            "Do NOT supply creatinine in mg/dL; it must be mmol/L for the ratio to come out in mg/mmol"
                        ],
                        "source": {
                            "citation": "KDIGO 2024 CKD Guideline. Kidney Int. 2024;105(4S):S117-S314.",
                            "url": "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: UacrInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(acr: f64, unit: AcrUnit) -> UacrInput {
        UacrInput {
            acr: Some(acr),
            acr_unit: Some(unit),
            albumin: None,
            creatinine: None,
        }
    }

    fn pair(albumin: f64, creatinine: f64) -> UacrInput {
        UacrInput {
            acr: None,
            acr_unit: None,
            albumin: Some(albumin),
            creatinine: Some(creatinine),
        }
    }

    #[test]
    fn a1_normal() {
        let o = compute(&ratio(1.5, AcrUnit::MgMmol)).unwrap();
        assert_eq!(o.stage, Stage::A1);
        assert_eq!(o.acr_mgmmol, 1.5);
    }

    #[test]
    fn a2_moderate() {
        let o = compute(&ratio(15.0, AcrUnit::MgMmol)).unwrap();
        assert_eq!(o.stage, Stage::A2);
    }

    #[test]
    fn a3_severe() {
        let o = compute(&ratio(45.0, AcrUnit::MgMmol)).unwrap();
        assert_eq!(o.stage, Stage::A3);
    }

    #[test]
    fn stage_boundaries() {
        // A1/A2 boundary at 3 mg/mmol; A2/A3 boundary at 30 mg/mmol.
        assert_eq!(Stage::from_acr_mgmmol(2.99), Stage::A1);
        assert_eq!(Stage::from_acr_mgmmol(3.0), Stage::A2);
        assert_eq!(Stage::from_acr_mgmmol(30.0), Stage::A2);
        assert_eq!(Stage::from_acr_mgmmol(30.01), Stage::A3);
    }

    #[test]
    fn mgg_matches_equivalent_mgmmol() {
        // 30 mg/mmol == 265.2 mg/g; both must stage and round identically.
        let a = compute(&ratio(30.0, AcrUnit::MgMmol)).unwrap();
        let b = compute(&ratio(30.0 * MGG_PER_MGMMOL, AcrUnit::MgG)).unwrap();
        assert_eq!(a.stage, b.stage);
        assert_eq!(a.acr_mgmmol, b.acr_mgmmol);
    }

    #[test]
    fn mgg_unit_changes_stage() {
        // 100 labelled mg/g is A2 (~11.3 mg/mmol); the same 100 as mg/mmol is A3.
        let as_mgg = compute(&ratio(100.0, AcrUnit::MgG)).unwrap();
        let as_mgmmol = compute(&ratio(100.0, AcrUnit::MgMmol)).unwrap();
        assert_eq!(as_mgg.stage, Stage::A2);
        assert_eq!(as_mgmmol.stage, Stage::A3);
    }

    #[test]
    fn albumin_creatinine_pair_computes_ratio() {
        // 35 mg/L albumin over 10 mmol/L creatinine = 3.5 mg/mmol -> A2.
        let o = compute(&pair(35.0, 10.0)).unwrap();
        assert_eq!(o.acr_mgmmol, 3.5);
        assert_eq!(o.stage, Stage::A2);
    }

    #[test]
    fn pair_and_ratio_agree() {
        // 150 mg/L over 10 mmol/L = 15 mg/mmol, same as the ratio form.
        let from_pair = compute(&pair(150.0, 10.0)).unwrap();
        let from_ratio = compute(&ratio(15.0, AcrUnit::MgMmol)).unwrap();
        assert_eq!(from_pair.acr_mgmmol, from_ratio.acr_mgmmol);
        assert_eq!(from_pair.stage, from_ratio.stage);
    }

    #[test]
    fn rejects_both_forms() {
        let both = UacrInput {
            acr: Some(5.0),
            acr_unit: Some(AcrUnit::MgMmol),
            albumin: Some(50.0),
            creatinine: Some(10.0),
        };
        assert!(compute(&both).is_err());
    }

    #[test]
    fn rejects_empty() {
        let empty = UacrInput {
            acr: None,
            acr_unit: None,
            albumin: None,
            creatinine: None,
        };
        assert!(compute(&empty).is_err());
    }

    #[test]
    fn rejects_acr_without_unit() {
        let no_unit = UacrInput {
            acr: Some(5.0),
            acr_unit: None,
            albumin: None,
            creatinine: None,
        };
        assert!(compute(&no_unit).is_err());
    }

    #[test]
    fn rejects_bad_values() {
        assert!(compute(&ratio(-1.0, AcrUnit::MgMmol)).is_err());
        assert!(compute(&pair(50.0, 0.0)).is_err());
        assert!(compute(&pair(-5.0, 10.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "acr": 15.0, "acr_unit": "mg/mmol" });
        let dynamic = Uacr.calculate(&value).unwrap();
        let typed = build_response(&ratio(15.0, AcrUnit::MgMmol)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_calculate_pair_form() {
        let value = json!({ "albumin": 150.0, "creatinine": 10.0 });
        let dynamic = Uacr.calculate(&value).unwrap();
        let typed = build_response(&pair(150.0, 10.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn schema_flags_unit_exclusion() {
        let schema = Uacr.input_schema();
        let def = &schema["properties"]["acr_unit"]["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("8.84x"));
    }
}
