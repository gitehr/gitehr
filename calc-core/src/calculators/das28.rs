//! DAS28 - Disease Activity Score in 28 joints for rheumatoid arthritis.
//!
//! Combines a tender and swollen 28-joint count, an inflammatory marker, and a
//! patient global-health VAS into a single disease-activity index. Two
//! variants are supported, selected by the `marker` input:
//!
//! - `DAS28-ESR = 0.56*sqrt(TJC28) + 0.28*sqrt(SJC28) + 0.70*ln(ESR) + 0.014*GH`
//! - `DAS28-CRP = 0.56*sqrt(TJC28) + 0.28*sqrt(SJC28) + 0.36*ln(CRP+1) + 0.014*GH + 0.96`
//!
//! where TJC28/SJC28 are the 28-joint counts (0-28), ESR is in mm/hr, CRP is in
//! mg/L, and GH is the patient global-health VAS (0-100 mm). The index is
//! reported to two decimal places with an activity band: remission (<2.6),
//! low (2.6 to <3.2), moderate (3.2 to 5.1), high (>5.1).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "das28";

/// Distribution licence: DAS28 is a published method, implemented here from the
/// primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1002/art.1780380107",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Prevoo MLL, van 't Hof MA, Kuper HH, et al. Modified disease activity scores that include \
twenty-eight-joint counts: development and validation in a prospective longitudinal study of \
patients with rheumatoid arthritis. Arthritis Rheum. 1995;38(1):44-48. \
doi:10.1002/art.1780380107";

/// Maximum count for each 28-joint assessment.
pub const MAX_JOINTS: u8 = 28;
/// Maximum patient global-health VAS, in mm.
pub const MAX_GLOBAL_HEALTH: f64 = 100.0;

/// Remission cut-off: below this is remission.
pub const REMISSION_CUTOFF: f64 = 2.6;
/// Upper bound of low disease activity (low is 2.6 to <3.2).
pub const LOW_CUTOFF: f64 = 3.2;
/// Upper bound of moderate disease activity (moderate is 3.2 to 5.1; above is high).
pub const MODERATE_CUTOFF: f64 = 5.1;

/// Inflammatory marker used, which selects the DAS28 variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Marker {
    /// Erythrocyte sedimentation rate, mm/hr (DAS28-ESR).
    Esr,
    /// C-reactive protein, mg/L (DAS28-CRP).
    Crp,
}

impl Marker {
    fn slug(self) -> &'static str {
        match self {
            Marker::Esr => "esr",
            Marker::Crp => "crp",
        }
    }

    fn formula(self) -> &'static str {
        match self {
            Marker::Esr => "DAS28-ESR",
            Marker::Crp => "DAS28-CRP",
        }
    }
}

/// Inputs to the DAS28 index.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Das28Input {
    /// Tender 28-joint count (0-28).
    pub tender_joint_count: u8,
    /// Swollen 28-joint count (0-28).
    pub swollen_joint_count: u8,
    /// Inflammatory marker used, selecting the ESR or CRP variant.
    pub marker: Marker,
    /// Marker value: ESR in mm/hr, or CRP in mg/L, per `marker`.
    pub marker_value: f64,
    /// Patient global health VAS (0-100 mm).
    pub global_health: f64,
}

/// Disease-activity band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Remission,
    Low,
    Moderate,
    High,
}

impl Activity {
    fn from_index(index: f64) -> Self {
        if index < REMISSION_CUTOFF {
            Activity::Remission
        } else if index < LOW_CUTOFF {
            Activity::Low
        } else if index <= MODERATE_CUTOFF {
            Activity::Moderate
        } else {
            Activity::High
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Activity::Remission => "remission",
            Activity::Low => "low",
            Activity::Moderate => "moderate",
            Activity::High => "high",
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            Activity::Remission => "remission",
            Activity::Low => "low disease activity",
            Activity::Moderate => "moderate disease activity",
            Activity::High => "high disease activity",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct Das28Outcome {
    /// The DAS28 index, rounded to two decimal places.
    pub index: f64,
    pub activity: Activity,
    /// Which variant was applied.
    pub marker: Marker,
    pub interpretation: String,
}

/// Pure scoring: the DAS28 index for the selected marker.
pub fn compute(input: &Das28Input) -> Result<Das28Outcome, CalcError> {
    if input.tender_joint_count > MAX_JOINTS || input.swollen_joint_count > MAX_JOINTS {
        return Err(CalcError::InvalidInput(format!(
            "joint counts must be between 0 and {MAX_JOINTS}"
        )));
    }
    if !input.global_health.is_finite()
        || input.global_health < 0.0
        || input.global_health > MAX_GLOBAL_HEALTH
    {
        return Err(CalcError::InvalidInput(format!(
            "global health must be between 0 and {MAX_GLOBAL_HEALTH}"
        )));
    }
    if !input.marker_value.is_finite() || input.marker_value <= 0.0 {
        // ln(marker) for ESR and ln(CRP+1) both require a positive marker value;
        // ESR additionally must be positive for ln to be defined.
        return Err(CalcError::InvalidInput(
            "marker value must be a positive number".into(),
        ));
    }

    let tjc = input.tender_joint_count as f64;
    let sjc = input.swollen_joint_count as f64;

    let raw = match input.marker {
        Marker::Esr => {
            0.56 * tjc.sqrt()
                + 0.28 * sjc.sqrt()
                + 0.70 * input.marker_value.ln()
                + 0.014 * input.global_health
        }
        Marker::Crp => {
            0.56 * tjc.sqrt()
                + 0.28 * sjc.sqrt()
                + 0.36 * (input.marker_value + 1.0).ln()
                + 0.014 * input.global_health
                + 0.96
        }
    };

    let index = (raw * 100.0).round() / 100.0;
    let activity = Activity::from_index(index);

    let interpretation = format!(
        "{} {index}: {} (remission <{REMISSION_CUTOFF}, low {REMISSION_CUTOFF} to <{LOW_CUTOFF}, \
moderate {LOW_CUTOFF} to {MODERATE_CUTOFF}, high >{MODERATE_CUTOFF}). DAS28 guides treat-to-target \
decisions in rheumatoid arthritis; ESR- and CRP-based scores are not interchangeable, so track a \
patient with one variant consistently.",
        input.marker.formula(),
        activity.descriptor()
    );

    Ok(Das28Outcome {
        index,
        activity,
        marker: input.marker,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Das28Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("das28_index".into(), json!(o.index));
    working.insert("activity".into(), json!(o.activity.slug()));
    working.insert("formula".into(), json!(o.marker.formula()));
    working.insert("marker".into(), json!(o.marker.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.index),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Das28;

impl Calculator for Das28 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "DAS28 (Rheumatoid Arthritis Disease Activity)"
    }

    fn description(&self) -> &'static str {
        "Disease Activity Score in 28 joints for rheumatoid arthritis, from tender/swollen joint counts, an ESR or CRP marker, and patient global health."
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
            "title": "Das28Input",
            "type": "object",
            "additionalProperties": false,
            "required": ["tender_joint_count", "swollen_joint_count", "marker", "marker_value", "global_health"],
            "properties": {
                "tender_joint_count": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 28,
                    "description": "Number of tender joints out of the 28 assessed (0-28)"
                },
                "swollen_joint_count": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 28,
                    "description": "Number of swollen joints out of the 28 assessed (0-28)"
                },
                "marker": {
                    "type": "string",
                    "enum": ["esr", "crp"],
                    "description": "Inflammatory marker used, which selects the DAS28-ESR or DAS28-CRP variant",
                    "definition": {
                        "concept": "DAS28 marker variant",
                        "statement": "DAS28 has two variants: DAS28-ESR uses ESR (mm/hr) and DAS28-CRP uses CRP (mg/L). The variant determines both the formula coefficients and the marker_value unit.",
                        "excludes": [
                            "DAS28-ESR and DAS28-CRP scores are NOT interchangeable; do not compare or trend a patient across the two variants"
                        ],
                        "source": {
                            "citation": "Prevoo MLL et al. Arthritis Rheum. 1995;38(1):44-48.",
                            "url": "https://doi.org/10.1002/art.1780380107"
                        },
                        "status": "draft"
                    }
                },
                "marker_value": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Marker value: ESR in mm/hr (marker=esr) or CRP in mg/L (marker=crp)",
                    "definition": {
                        "concept": "DAS28 marker value and unit",
                        "statement": "ESR is entered in mm/hr and CRP in mg/L. The DAS28-CRP formula uses ln(CRP+1), so a CRP of 0 is admissible in principle; this implementation requires a positive marker value because ESR enters as ln(ESR).",
                        "excludes": [
                            "Do NOT pass CRP in mg/dL; the formula expects mg/L"
                        ],
                        "source": {
                            "citation": "Prevoo MLL et al. Arthritis Rheum. 1995;38(1):44-48.",
                            "url": "https://doi.org/10.1002/art.1780380107"
                        },
                        "status": "draft"
                    }
                },
                "global_health": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Patient global health assessment on a 0-100 mm visual analogue scale"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Das28Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        tjc: u8,
        sjc: u8,
        marker: Marker,
        marker_value: f64,
        global_health: f64,
    ) -> Das28Input {
        Das28Input {
            tender_joint_count: tjc,
            swollen_joint_count: sjc,
            marker,
            marker_value,
            global_health,
        }
    }

    #[test]
    fn esr_worked_example() {
        // TJC 5, SJC 3, ESR 30, GH 50:
        // 0.56*sqrt(5) + 0.28*sqrt(3) + 0.70*ln(30) + 0.014*50
        // = 0.56*2.2360680 + 0.28*1.7320508 + 0.70*3.4011974 + 0.7
        // = 1.2521981 + 0.4849742 + 2.3808382 + 0.7 = 4.8180105 -> 4.82.
        let o = compute(&input(5, 3, Marker::Esr, 30.0, 50.0)).unwrap();
        assert_eq!(o.index, 4.82);
        assert_eq!(o.activity, Activity::Moderate);
        assert_eq!(o.marker, Marker::Esr);
    }

    #[test]
    fn crp_worked_example() {
        // TJC 5, SJC 3, CRP 10, GH 50:
        // 0.56*sqrt(5) + 0.28*sqrt(3) + 0.36*ln(11) + 0.014*50 + 0.96
        // = 1.2521981 + 0.4849742 + 0.36*2.3978953 + 0.7 + 0.96
        // = 1.2521981 + 0.4849742 + 0.8632423 + 0.7 + 0.96 = 4.2604146 -> 4.26.
        let o = compute(&input(5, 3, Marker::Crp, 10.0, 50.0)).unwrap();
        assert_eq!(o.index, 4.26);
        assert_eq!(o.activity, Activity::Moderate);
        assert_eq!(o.marker, Marker::Crp);
    }

    #[test]
    fn remission_band() {
        // All-zero counts, low ESR, no global health: small index in remission.
        // TJC 0, SJC 0, ESR 5, GH 0: 0.70*ln(5) = 0.70*1.6094379 = 1.1266 -> 1.13.
        let o = compute(&input(0, 0, Marker::Esr, 5.0, 0.0)).unwrap();
        assert_eq!(o.index, 1.13);
        assert_eq!(o.activity, Activity::Remission);
    }

    #[test]
    fn high_band() {
        // Maximal disease: TJC 28, SJC 28, ESR 100, GH 100.
        // 0.56*sqrt(28)+0.28*sqrt(28)+0.70*ln(100)+0.014*100
        // = 0.56*5.2915026 + 0.28*5.2915026 + 0.70*4.6051702 + 1.4
        // = 2.9632415 + 1.4816207 + 3.2236191 + 1.4 = 9.0684813 -> 9.07.
        let o = compute(&input(28, 28, Marker::Esr, 100.0, 100.0)).unwrap();
        assert_eq!(o.index, 9.07);
        assert_eq!(o.activity, Activity::High);
    }

    #[test]
    fn activity_band_boundaries() {
        // Band edges, verified against the from_index thresholds directly.
        assert_eq!(Activity::from_index(2.59), Activity::Remission);
        assert_eq!(Activity::from_index(2.6), Activity::Low);
        assert_eq!(Activity::from_index(3.19), Activity::Low);
        assert_eq!(Activity::from_index(3.2), Activity::Moderate);
        assert_eq!(Activity::from_index(5.1), Activity::Moderate);
        assert_eq!(Activity::from_index(5.11), Activity::High);
    }

    #[test]
    fn rejects_out_of_range_inputs() {
        // Joint counts above 28.
        assert!(compute(&input(29, 0, Marker::Esr, 30.0, 50.0)).is_err());
        assert!(compute(&input(0, 29, Marker::Esr, 30.0, 50.0)).is_err());
        // Global health out of 0-100.
        assert!(compute(&input(5, 3, Marker::Esr, 30.0, 101.0)).is_err());
        assert!(compute(&input(5, 3, Marker::Esr, 30.0, -1.0)).is_err());
        // Non-positive / non-finite marker value.
        assert!(compute(&input(5, 3, Marker::Esr, 0.0, 50.0)).is_err());
        assert!(compute(&input(5, 3, Marker::Crp, -1.0, 50.0)).is_err());
        assert!(compute(&input(5, 3, Marker::Esr, f64::NAN, 50.0)).is_err());
    }

    #[test]
    fn esr_and_crp_differ() {
        // Same clinical inputs, different variant: scores should not coincide.
        let esr = compute(&input(5, 3, Marker::Esr, 30.0, 50.0)).unwrap();
        let crp = compute(&input(5, 3, Marker::Crp, 30.0, 50.0)).unwrap();
        assert_ne!(esr.index, crp.index);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "tender_joint_count": 5,
            "swollen_joint_count": 3,
            "marker": "esr",
            "marker_value": 30,
            "global_health": 50
        });
        let dynamic = Das28.calculate(&value).unwrap();
        let typed = build_response(&input(5, 3, Marker::Esr, 30.0, 50.0)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(4.82));
    }

    #[test]
    fn dynamic_calculate_matches_typed_crp() {
        let value = json!({
            "tender_joint_count": 5,
            "swollen_joint_count": 3,
            "marker": "crp",
            "marker_value": 10,
            "global_health": 50
        });
        let dynamic = Das28.calculate(&value).unwrap();
        let typed = build_response(&input(5, 3, Marker::Crp, 10.0, 50.0)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(4.26));
    }

    #[test]
    fn schema_constrains_inputs() {
        let schema = Das28.input_schema();
        let props = &schema["properties"];
        assert_eq!(props["tender_joint_count"]["maximum"], json!(28));
        assert_eq!(props["swollen_joint_count"]["maximum"], json!(28));
        assert_eq!(props["global_health"]["maximum"], json!(100));
        assert_eq!(props["marker"]["enum"], json!(["esr", "crp"]));
    }
}
