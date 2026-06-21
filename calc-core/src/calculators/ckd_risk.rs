//! KDIGO CKD risk category - the eGFR x uACR "heatmap".
//!
//! Combines the CKD G-stage (from eGFR) and the albuminuria A-stage (from uACR
//! in mg/mmol) into the KDIGO prognosis risk category: the green / yellow /
//! orange / red heatmap that grades the risk of adverse outcomes (CKD
//! progression, kidney failure, acute kidney injury, cardiovascular events and
//! all-cause mortality) by the combination of the two axes.
//!
//! The staging helpers are reproduced inline here (rather than calling into the
//! `egfr` / `uacr` modules) so this calculator is self-contained: it takes the
//! two already-computed laboratory values and only classifies them.
//!
//! eGFR is taken in mL/min/1.73m^2 and ACR in mg/mmol; both are required in
//! those units, matching how the G/A grid is defined. The heatmap is the
//! canonical KDIGO grid (stable since KDIGO 2012, retained in KDIGO 2024 and
//! reproduced in NICE NG203):
//!
//! ```text
//!                     A1 (<3)    A2 (3-30)   A3 (>30)
//!   G1  (>=90)        Low        Moderate    High
//!   G2  (60-89)       Low        Moderate    High
//!   G3a (45-59)       Moderate   High        Very high
//!   G3b (30-44)       High       Very high   Very high
//!   G4  (15-29)       Very high  Very high   Very high
//!   G5  (<15)         Very high  Very high   Very high
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "ckd_risk";

/// Distribution licence: the KDIGO G/A risk classification (the heatmap) is a
/// published staging method; the grid is implemented here from the primary
/// guideline. The KDIGO guideline text is CC BY-NC-ND, but the categorical
/// staging grid is a fact/method rather than copyrightable content - the same
/// treatment as the `egfr` G-stages and the `uacr` A-stages.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary guideline (KDIGO 2024 G/A risk classification heatmap)",
    source_url: "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Kidney Disease: Improving Global Outcomes (KDIGO) CKD Work Group. KDIGO 2024 Clinical \
Practice Guideline for the Evaluation and Management of Chronic Kidney Disease. Kidney Int. \
2024;105(4S):S117-S314. doi:10.1016/j.kint.2023.10.018";

/// Inputs to the KDIGO G/A risk classification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CkdRiskInput {
    /// eGFR in mL/min/1.73m^2 (sets the G-stage).
    pub egfr: f64,
    /// Urine albumin-to-creatinine ratio in mg/mmol (sets the A-stage).
    pub acr: f64,
}

/// CKD G-stage by eGFR (KDIGO).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GStage {
    G1,
    G2,
    G3a,
    G3b,
    G4,
    G5,
}

impl GStage {
    /// G-stage from eGFR in mL/min/1.73m^2 (KDIGO: G1 >=90, G2 60-89,
    /// G3a 45-59, G3b 30-44, G4 15-29, G5 <15).
    fn from_egfr(egfr: f64) -> Self {
        if egfr >= 90.0 {
            GStage::G1
        } else if egfr >= 60.0 {
            GStage::G2
        } else if egfr >= 45.0 {
            GStage::G3a
        } else if egfr >= 30.0 {
            GStage::G3b
        } else if egfr >= 15.0 {
            GStage::G4
        } else {
            GStage::G5
        }
    }

    fn slug(self) -> &'static str {
        match self {
            GStage::G1 => "G1",
            GStage::G2 => "G2",
            GStage::G3a => "G3a",
            GStage::G3b => "G3b",
            GStage::G4 => "G4",
            GStage::G5 => "G5",
        }
    }
}

/// KDIGO albuminuria category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AStage {
    A1,
    A2,
    A3,
}

impl AStage {
    /// A-stage from the ratio in mg/mmol (KDIGO: A1 <3, A2 3-30, A3 >30).
    ///
    /// The 3 and 30 boundaries belong to A2 (KDIGO defines A2 as 3-30 mg/mmol),
    /// so a value exactly on a boundary stages as the more abnormal category
    /// only strictly above it.
    fn from_acr_mgmmol(acr: f64) -> Self {
        if acr < 3.0 {
            AStage::A1
        } else if acr <= 30.0 {
            AStage::A2
        } else {
            AStage::A3
        }
    }

    fn slug(self) -> &'static str {
        match self {
            AStage::A1 => "A1",
            AStage::A2 => "A2",
            AStage::A3 => "A3",
        }
    }
}

/// KDIGO prognosis risk category - the heatmap colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    /// Green.
    Low,
    /// Yellow.
    ModeratelyIncreased,
    /// Orange.
    High,
    /// Red.
    VeryHigh,
}

impl Risk {
    /// The KDIGO G/A heatmap: the risk category for a G/A combination.
    fn from_stages(g: GStage, a: AStage) -> Self {
        use AStage::*;
        use GStage::*;
        match (g, a) {
            // Green: preserved/mildly reduced eGFR with normal albuminuria.
            (G1 | G2, A1) => Risk::Low,
            // Yellow.
            (G1 | G2, A2) => Risk::ModeratelyIncreased,
            (G3a, A1) => Risk::ModeratelyIncreased,
            // Orange.
            (G1 | G2, A3) => Risk::High,
            (G3a, A2) => Risk::High,
            (G3b, A1) => Risk::High,
            // Red.
            (G3a, A3) => Risk::VeryHigh,
            (G3b, A2 | A3) => Risk::VeryHigh,
            (G4 | G5, _) => Risk::VeryHigh,
        }
    }

    /// Human-readable label, including the heatmap colour.
    fn label(self) -> &'static str {
        match self {
            Risk::Low => "Low risk (green)",
            Risk::ModeratelyIncreased => "Moderately increased risk (yellow)",
            Risk::High => "High risk (orange)",
            Risk::VeryHigh => "Very high risk (red)",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct CkdRiskOutcome {
    pub g_stage: GStage,
    pub a_stage: AStage,
    pub risk: Risk,
    pub interpretation: String,
}

/// Pure classification: G-stage x A-stage -> KDIGO risk category.
pub fn compute(input: &CkdRiskInput) -> Result<CkdRiskOutcome, CalcError> {
    if !input.egfr.is_finite() || input.egfr <= 0.0 {
        return Err(CalcError::InvalidInput(
            "egfr must be a positive number (mL/min/1.73m2)".into(),
        ));
    }
    if !input.acr.is_finite() || input.acr < 0.0 {
        return Err(CalcError::InvalidInput(
            "acr must be a non-negative number (mg/mmol)".into(),
        ));
    }

    let g_stage = GStage::from_egfr(input.egfr);
    let a_stage = AStage::from_acr_mgmmol(input.acr);
    let risk = Risk::from_stages(g_stage, a_stage);

    let interpretation = format!(
        "KDIGO classification {}{}: {}. The risk category combines the eGFR G-stage and the \
albuminuria A-stage to grade the risk of adverse outcomes (CKD progression, kidney failure, acute \
kidney injury, cardiovascular events and all-cause mortality). A diagnosis of CKD requires the \
abnormality to persist for more than 3 months; eGFR is unreliable in acute kidney injury and at \
extremes of muscle mass, and ACR can be transiently raised (e.g. urinary tract infection, recent \
vigorous exercise), so abnormal values are normally confirmed on repeat testing.",
        g_stage.slug(),
        a_stage.slug(),
        risk.label()
    );

    Ok(CkdRiskOutcome {
        g_stage,
        a_stage,
        risk,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &CkdRiskInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("g_stage".into(), json!(o.g_stage.slug()));
    working.insert("a_stage".into(), json!(o.a_stage.slug()));
    working.insert("risk_category".into(), json!(o.risk.label()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.risk.label()),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct CkdRisk;

impl Calculator for CkdRisk {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "KDIGO CKD risk category (eGFR x ACR heatmap)"
    }

    fn description(&self) -> &'static str {
        "Combines the eGFR G-stage and albuminuria A-stage into the KDIGO prognosis risk category (the green/yellow/orange/red heatmap)."
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
            "title": "CkdRiskInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["egfr", "acr"],
            "properties": {
                "egfr": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "eGFR in mL/min/1.73m2 (sets the KDIGO G-stage)",
                    "definition": {
                        "concept": "eGFR for G-staging",
                        "statement": "Estimated glomerular filtration rate in mL/min/1.73m2. Sets the G-stage: G1 >=90, G2 60-89, G3a 45-59, G3b 30-44, G4 15-29, G5 <15.",
                        "caveats": "eGFR is unreliable in acute kidney injury and at extremes of muscle mass; a diagnosis of CKD requires the abnormality to persist for more than 3 months.",
                        "source": {
                            "citation": "KDIGO 2024 CKD Guideline. Kidney Int. 2024;105(4S):S117-S314.",
                            "url": "https://kdigo.org/wp-content/uploads/2024/03/KDIGO-2024-CKD-Guideline.pdf"
                        },
                        "status": "draft"
                    }
                },
                "acr": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Urine albumin-to-creatinine ratio in mg/mmol (sets the KDIGO A-stage)",
                    "definition": {
                        "concept": "ACR for A-staging",
                        "statement": "Urine albumin-to-creatinine ratio in mg/mmol. Sets the A-stage: A1 <3, A2 3-30, A3 >30.",
                        "excludes": [
                            "Do NOT supply ACR in mg/g; the KDIGO grid here is defined in mg/mmol (1 mg/mmol = 8.84 mg/g) and the wrong unit silently shifts the A-stage and the risk category"
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
        let parsed: CkdRiskInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(egfr: f64, acr: f64) -> CkdRiskInput {
        CkdRiskInput { egfr, acr }
    }

    // One cell from each risk colour.

    #[test]
    fn green_low_g1_a1() {
        // eGFR 100 (G1), ACR 1 (A1) -> green.
        let o = compute(&input(100.0, 1.0)).unwrap();
        assert_eq!(o.g_stage, GStage::G1);
        assert_eq!(o.a_stage, AStage::A1);
        assert_eq!(o.risk, Risk::Low);
    }

    #[test]
    fn yellow_moderate_g3a_a1() {
        // eGFR 50 (G3a), ACR 1 (A1) -> yellow.
        let o = compute(&input(50.0, 1.0)).unwrap();
        assert_eq!(o.g_stage, GStage::G3a);
        assert_eq!(o.a_stage, AStage::A1);
        assert_eq!(o.risk, Risk::ModeratelyIncreased);
    }

    #[test]
    fn orange_high_g1_a3() {
        // eGFR 95 (G1), ACR 40 (A3) -> orange.
        let o = compute(&input(95.0, 40.0)).unwrap();
        assert_eq!(o.g_stage, GStage::G1);
        assert_eq!(o.a_stage, AStage::A3);
        assert_eq!(o.risk, Risk::High);
    }

    #[test]
    fn orange_high_g3b_a1() {
        // eGFR 35 (G3b), ACR 1 (A1) -> orange.
        let o = compute(&input(35.0, 1.0)).unwrap();
        assert_eq!(o.g_stage, GStage::G3b);
        assert_eq!(o.a_stage, AStage::A1);
        assert_eq!(o.risk, Risk::High);
    }

    #[test]
    fn red_very_high_g4_a1() {
        // eGFR 20 (G4), ACR 1 (A1) -> red (G4 is very high at any albuminuria).
        let o = compute(&input(20.0, 1.0)).unwrap();
        assert_eq!(o.g_stage, GStage::G4);
        assert_eq!(o.a_stage, AStage::A1);
        assert_eq!(o.risk, Risk::VeryHigh);
    }

    #[test]
    fn red_very_high_g3a_a3() {
        // eGFR 50 (G3a), ACR 50 (A3) -> red.
        let o = compute(&input(50.0, 50.0)).unwrap();
        assert_eq!(o.risk, Risk::VeryHigh);
    }

    // Exhaustive grid check against the published KDIGO heatmap.
    #[test]
    fn full_grid_matches_kdigo_heatmap() {
        use AStage::*;
        use GStage::*;
        use Risk::*;
        let cases = [
            (G1, A1, Low),
            (G1, A2, ModeratelyIncreased),
            (G1, A3, High),
            (G2, A1, Low),
            (G2, A2, ModeratelyIncreased),
            (G2, A3, High),
            (G3a, A1, ModeratelyIncreased),
            (G3a, A2, High),
            (G3a, A3, VeryHigh),
            (G3b, A1, High),
            (G3b, A2, VeryHigh),
            (G3b, A3, VeryHigh),
            (G4, A1, VeryHigh),
            (G4, A2, VeryHigh),
            (G4, A3, VeryHigh),
            (G5, A1, VeryHigh),
            (G5, A2, VeryHigh),
            (G5, A3, VeryHigh),
        ];
        for (g, a, expected) in cases {
            assert_eq!(
                Risk::from_stages(g, a),
                expected,
                "grid cell {}{} should be {:?}",
                g.slug(),
                a.slug(),
                expected
            );
        }
    }

    #[test]
    fn g_stage_boundaries() {
        assert_eq!(GStage::from_egfr(90.0), GStage::G1);
        assert_eq!(GStage::from_egfr(89.9), GStage::G2);
        assert_eq!(GStage::from_egfr(60.0), GStage::G2);
        assert_eq!(GStage::from_egfr(59.0), GStage::G3a);
        assert_eq!(GStage::from_egfr(45.0), GStage::G3a);
        assert_eq!(GStage::from_egfr(44.0), GStage::G3b);
        assert_eq!(GStage::from_egfr(30.0), GStage::G3b);
        assert_eq!(GStage::from_egfr(29.0), GStage::G4);
        assert_eq!(GStage::from_egfr(15.0), GStage::G4);
        assert_eq!(GStage::from_egfr(14.0), GStage::G5);
    }

    #[test]
    fn a_stage_boundaries() {
        assert_eq!(AStage::from_acr_mgmmol(2.99), AStage::A1);
        assert_eq!(AStage::from_acr_mgmmol(3.0), AStage::A2);
        assert_eq!(AStage::from_acr_mgmmol(30.0), AStage::A2);
        assert_eq!(AStage::from_acr_mgmmol(30.01), AStage::A3);
    }

    #[test]
    fn boundary_shifts_risk_category() {
        // At eGFR 60 (G2) the cell is green at A1 but a moderate-albuminuria
        // ACR of 3.0 (A2) moves it to yellow.
        assert_eq!(compute(&input(60.0, 2.99)).unwrap().risk, Risk::Low);
        assert_eq!(
            compute(&input(60.0, 3.0)).unwrap().risk,
            Risk::ModeratelyIncreased
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(compute(&input(0.0, 5.0)).is_err());
        assert!(compute(&input(-1.0, 5.0)).is_err());
        assert!(compute(&input(50.0, -1.0)).is_err());
        assert!(compute(&input(f64::NAN, 5.0)).is_err());
        assert!(compute(&input(50.0, f64::INFINITY)).is_err());
    }

    #[test]
    fn result_is_the_risk_category() {
        let r = build_response(&input(50.0, 1.0)).unwrap();
        assert_eq!(r.result, json!("Moderately increased risk (yellow)"));
        assert_eq!(r.working["g_stage"], json!("G3a"));
        assert_eq!(r.working["a_stage"], json!("A1"));
        assert_eq!(r.working["risk_category"], json!("Moderately increased risk (yellow)"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "egfr": 50.0, "acr": 1.0 });
        let dynamic = CkdRisk.calculate(&value).unwrap();
        let typed = build_response(&input(50.0, 1.0)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn rejects_unknown_fields() {
        let value = json!({ "egfr": 50.0, "acr": 1.0, "age": 60 });
        assert!(CkdRisk.calculate(&value).is_err());
    }

    #[test]
    fn schema_flags_acr_unit_exclusion() {
        let schema = CkdRisk.input_schema();
        let def = &schema["properties"]["acr"]["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("8.84"));
    }
}
