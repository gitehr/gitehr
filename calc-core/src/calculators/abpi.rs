//! ABPI - Ankle-Brachial Pressure Index.
//!
//! A bedside ratio used mainly to screen for peripheral arterial disease (PAD)
//! and, critically, to decide whether compression therapy (for venous leg
//! ulcers/oedema) is safe. For each leg:
//!
//! ```text
//! ABPI(leg) = highest ankle systolic in that leg / highest brachial systolic of the two arms
//! ```
//!
//! The denominator is the higher of the two arms' brachial systolic pressures.
//! Both legs are reported; the lower (worse) ABPI drives the overall
//! interpretation, since the more ischaemic leg is the one that constrains
//! management.
//!
//! Safety point baked into the output: a *high* ABPI (typically >1.4) usually
//! means calcified, non-compressible vessels - common in diabetes and chronic
//! kidney disease - and does NOT mean good perfusion. Such readings are
//! unreliable and can mask severe disease.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "abpi";

/// Distribution licence: ABPI is a published bedside method (a simple ratio),
/// implemented here from primary guidance. Not subject to copyright.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from published clinical guidance (NICE CG147)",
    source_url: "https://www.nice.org.uk/guidance/cg147",
};

/// Primary citation.
pub const REFERENCE: &str =
    "National Institute for Health and Care Excellence. Peripheral arterial disease: diagnosis \
and management. Clinical guideline CG147. London: NICE; 2012 (updated 2020). \
https://www.nice.org.uk/guidance/cg147";

/// Inputs: four systolic pressures in mmHg.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AbpiInput {
    /// Highest ankle systolic pressure measured in the right leg (mmHg).
    /// Conventionally the higher of dorsalis pedis and posterior tibial.
    pub right_ankle_systolic: f64,
    /// Highest ankle systolic pressure measured in the left leg (mmHg).
    pub left_ankle_systolic: f64,
    /// Brachial systolic pressure in the right arm (mmHg).
    pub right_brachial_systolic: f64,
    /// Brachial systolic pressure in the left arm (mmHg).
    pub left_brachial_systolic: f64,
}

/// Clinical band an ABPI falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// > 1.4: abnormally high - calcified, non-compressible vessels.
    HighCalcified,
    /// 1.0 - 1.4: normal.
    Normal,
    /// 0.91 - 0.99: borderline.
    Borderline,
    /// 0.5 - 0.9: mild-to-moderate PAD.
    MildModeratePad,
    /// < 0.5: severe PAD / critical limb ischaemia.
    SeverePad,
}

impl Band {
    /// Classify a single leg's ABPI value.
    fn from_abpi(abpi: f64) -> Self {
        if abpi > 1.4 {
            Band::HighCalcified
        } else if abpi >= 1.0 {
            Band::Normal
        } else if abpi >= 0.91 {
            // 0.91 - 0.99 inclusive once rounded; values >= 0.91 and < 1.0.
            Band::Borderline
        } else if abpi >= 0.5 {
            Band::MildModeratePad
        } else {
            Band::SeverePad
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Band::HighCalcified => "high_calcified",
            Band::Normal => "normal",
            Band::Borderline => "borderline",
            Band::MildModeratePad => "mild_moderate_pad",
            Band::SeverePad => "severe_pad",
        }
    }

    /// Per-leg phrase describing the band.
    fn descriptor(self) -> &'static str {
        match self {
            Band::HighCalcified => {
                "abnormally high - calcified, non-compressible vessels (reading unreliable)"
            }
            Band::Normal => "normal",
            Band::Borderline => "borderline",
            Band::MildModeratePad => "mild-to-moderate peripheral arterial disease",
            Band::SeverePad => "severe PAD / critical limb ischaemia",
        }
    }

    /// Rank for picking the worst (most clinically concerning) leg.
    ///
    /// Lowest ABPI is worst, but a high/calcified reading is also unsafe because
    /// it cannot be trusted, so it is ranked alongside disease rather than as
    /// "best". Ordering used only when ABPI values tie.
    fn severity(self) -> u8 {
        match self {
            Band::SeverePad => 4,
            Band::MildModeratePad => 3,
            Band::HighCalcified => 2,
            Band::Borderline => 1,
            Band::Normal => 0,
        }
    }
}

/// ABPI for one leg, with its band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegResult {
    /// ABPI ratio, rounded to two decimal places.
    pub abpi: f64,
    pub band: Band,
}

/// The computed outcome for both legs.
#[derive(Debug, Clone, PartialEq)]
pub struct AbpiOutcome {
    pub right: LegResult,
    pub left: LegResult,
    /// Higher brachial systolic used as the denominator (mmHg).
    pub brachial_used: f64,
    /// The worse (lower / less safe) leg's ABPI, used for overall band.
    pub overall_abpi: f64,
    pub overall_band: Band,
    pub interpretation: String,
}

/// Round to two decimal places (the conventional ABPI precision).
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn require_positive(value: f64, label: &str) -> Result<(), CalcError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(CalcError::InvalidInput(format!(
            "{label} must be a positive number (mmHg)"
        )));
    }
    Ok(())
}

/// Pure computation: per-leg ABPI plus overall interpretation.
pub fn compute(input: &AbpiInput) -> Result<AbpiOutcome, CalcError> {
    require_positive(input.right_ankle_systolic, "right_ankle_systolic")?;
    require_positive(input.left_ankle_systolic, "left_ankle_systolic")?;
    require_positive(input.right_brachial_systolic, "right_brachial_systolic")?;
    require_positive(input.left_brachial_systolic, "left_brachial_systolic")?;

    // Denominator is the higher of the two arms' brachial systolic pressures.
    let brachial_used = input
        .right_brachial_systolic
        .max(input.left_brachial_systolic);

    let right_abpi = round2(input.right_ankle_systolic / brachial_used);
    let left_abpi = round2(input.left_ankle_systolic / brachial_used);

    let right = LegResult {
        abpi: right_abpi,
        band: Band::from_abpi(right_abpi),
    };
    let left = LegResult {
        abpi: left_abpi,
        band: Band::from_abpi(left_abpi),
    };

    // Worst leg drives the overall interpretation: rank primarily by clinical
    // severity (so a high/calcified leg, which is unsafe because the reading is
    // unreliable, is not treated as fine just because its ratio is large), then
    // by lower ABPI within the same band.
    let worst = if (left.band.severity(), -left.abpi) > (right.band.severity(), -right.abpi) {
        left
    } else {
        right
    };

    let overall_abpi = worst.abpi;
    let overall_band = worst.band;

    let action = match overall_band {
        Band::HighCalcified => {
            "A high ABPI usually means calcified, non-compressible arteries (common in diabetes \
and CKD) and does NOT indicate good perfusion - the reading is unreliable and may mask severe \
disease. Do not use it to clear compression therapy; assess perfusion by other means (e.g. toe \
pressures, waveforms) and seek vascular advice if PAD is suspected."
        }
        Band::Normal => {
            "Normal ABPI: significant PAD is unlikely and compression therapy is generally safe \
per local policy. A normal/raised ABPI does not fully exclude PAD if clinical suspicion is high."
        }
        Band::Borderline => {
            "Borderline ABPI: PAD cannot be confidently excluded. Correlate clinically; consider \
caution with compression and reassessment or further vascular assessment per local policy."
        }
        Band::MildModeratePad => {
            "Mild-to-moderate PAD. Compression therapy is generally safe with caution per local \
policy (reduced compression and close monitoring are often advised in this range). Manage \
cardiovascular risk factors and consider vascular referral."
        }
        Band::SeverePad => {
            "Severe PAD / critical limb ischaemia. Do NOT apply compression therapy. Arrange \
urgent vascular referral."
        }
    };

    let interpretation = format!(
        "Right leg ABPI {right_abpi:.2} ({}); left leg ABPI {left_abpi:.2} ({}). Denominator \
(highest brachial systolic) {brachial_used:.0} mmHg. Overall (worse leg) ABPI {overall_abpi:.2}: \
{}. {action}",
        right.band.descriptor(),
        left.band.descriptor(),
        overall_band.descriptor(),
    );

    Ok(AbpiOutcome {
        right,
        left,
        brachial_used,
        overall_abpi,
        overall_band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &AbpiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("right_abpi".into(), json!(o.right.abpi));
    working.insert("left_abpi".into(), json!(o.left.abpi));
    working.insert("brachial_used".into(), json!(o.brachial_used));
    working.insert(
        "right_interpretation".into(),
        json!(o.right.band.descriptor()),
    );
    working.insert("left_interpretation".into(), json!(o.left.band.descriptor()));
    working.insert("right_band".into(), json!(o.right.band.slug()));
    working.insert("left_band".into(), json!(o.left.band.slug()));
    working.insert("overall_band".into(), json!(o.overall_band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        // The worse leg's ABPI is the headline number.
        result: json!(o.overall_abpi),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Abpi;

impl Calculator for Abpi {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ABPI (Ankle-Brachial Pressure Index)"
    }

    fn description(&self) -> &'static str {
        "Ankle-Brachial Pressure Index per leg from ankle and brachial systolic pressures; screens for peripheral arterial disease and informs compression-therapy safety."
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
            "title": "AbpiInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "right_ankle_systolic",
                "left_ankle_systolic",
                "right_brachial_systolic",
                "left_brachial_systolic"
            ],
            "properties": {
                "right_ankle_systolic": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Highest ankle systolic pressure in the right leg (mmHg)",
                    "definition": {
                        "concept": "Right ankle systolic pressure",
                        "statement": "The higher of the right dorsalis pedis and posterior tibial systolic pressures, by Doppler.",
                        "caveats": "Use the higher of the two ankle vessels for that leg; the ankle reading forms the numerator of that leg's ABPI.",
                        "source": {
                            "citation": "NICE CG147. Peripheral arterial disease: diagnosis and management.",
                            "url": "https://www.nice.org.uk/guidance/cg147"
                        },
                        "status": "draft"
                    }
                },
                "left_ankle_systolic": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Highest ankle systolic pressure in the left leg (mmHg)"
                },
                "right_brachial_systolic": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Brachial systolic pressure in the right arm (mmHg)"
                },
                "left_brachial_systolic": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Brachial systolic pressure in the left arm (mmHg)",
                    "definition": {
                        "concept": "Brachial systolic pressure",
                        "statement": "The higher of the two arms' brachial systolic pressures is the single denominator for both legs' ABPI.",
                        "excludes": [
                            "A HIGH ABPI (>1.4) does NOT mean good perfusion: it usually reflects calcified, non-compressible vessels (common in diabetes and CKD), gives a falsely raised ratio, and must not be used to clear compression therapy"
                        ],
                        "source": {
                            "citation": "NICE CG147. Peripheral arterial disease: diagnosis and management.",
                            "url": "https://www.nice.org.uk/guidance/cg147"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AbpiInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(ra: f64, la: f64, rb: f64, lb: f64) -> AbpiInput {
        AbpiInput {
            right_ankle_systolic: ra,
            left_ankle_systolic: la,
            right_brachial_systolic: rb,
            left_brachial_systolic: lb,
        }
    }

    #[test]
    fn normal_both_legs() {
        // Ankles 130/130, brachials 120/120 -> 1.08 each, normal.
        let o = compute(&input(130.0, 130.0, 120.0, 120.0)).unwrap();
        assert_eq!(o.right.abpi, 1.08);
        assert_eq!(o.left.abpi, 1.08);
        assert_eq!(o.brachial_used, 120.0);
        assert_eq!(o.overall_band, Band::Normal);
        assert_eq!(o.overall_abpi, 1.08);
    }

    #[test]
    fn higher_brachial_is_the_denominator() {
        // Arms differ: 110 and 140 -> denominator must be 140.
        let o = compute(&input(120.0, 120.0, 110.0, 140.0)).unwrap();
        assert_eq!(o.brachial_used, 140.0);
        assert_eq!(o.right.abpi, round2(120.0 / 140.0)); // 0.86
        assert_eq!(o.left.abpi, round2(120.0 / 140.0));
    }

    #[test]
    fn mild_moderate_pad() {
        // Ankle 90, brachial 120 -> 0.75, mild-to-moderate PAD.
        let o = compute(&input(90.0, 90.0, 120.0, 120.0)).unwrap();
        assert_eq!(o.overall_abpi, 0.75);
        assert_eq!(o.overall_band, Band::MildModeratePad);
        assert!(o.interpretation.contains("caution"));
    }

    #[test]
    fn severe_pad_blocks_compression() {
        // Ankle 50, brachial 120 -> 0.42, severe PAD / CLI.
        let o = compute(&input(50.0, 50.0, 120.0, 120.0)).unwrap();
        assert_eq!(o.overall_abpi, 0.42);
        assert_eq!(o.overall_band, Band::SeverePad);
        assert!(o.interpretation.contains("Do NOT apply compression"));
        assert!(o.interpretation.contains("urgent vascular referral"));
    }

    #[test]
    fn high_calcified_is_flagged_unreliable() {
        // Ankle 200, brachial 120 -> 1.67, calcified / non-compressible.
        let o = compute(&input(200.0, 200.0, 120.0, 120.0)).unwrap();
        assert!(o.overall_abpi > 1.4);
        assert_eq!(o.overall_band, Band::HighCalcified);
        assert!(o.interpretation.contains("does NOT indicate good perfusion"));
        assert!(o.interpretation.contains("calcified"));
    }

    #[test]
    fn differing_legs_report_both_and_worst_drives_overall() {
        // Right normal (1.08), left PAD (0.67). Worst (left) drives overall.
        let o = compute(&input(130.0, 80.0, 120.0, 120.0)).unwrap();
        assert_eq!(o.right.abpi, 1.08);
        assert_eq!(o.right.band, Band::Normal);
        assert_eq!(o.left.abpi, round2(80.0 / 120.0)); // 0.67
        assert_eq!(o.left.band, Band::MildModeratePad);
        assert_eq!(o.overall_abpi, o.left.abpi);
        assert_eq!(o.overall_band, Band::MildModeratePad);
    }

    #[test]
    fn calcified_leg_outranks_normal_leg_overall() {
        // One leg calcified-high, the other normal. The calcified leg is unsafe
        // (unreliable), so it should drive the overall band, not be ignored.
        let o = compute(&input(200.0, 120.0, 120.0, 120.0)).unwrap();
        assert_eq!(o.right.band, Band::HighCalcified);
        assert_eq!(o.left.band, Band::Normal);
        assert_eq!(o.overall_band, Band::HighCalcified);
    }

    #[test]
    fn band_boundaries() {
        assert_eq!(Band::from_abpi(1.41), Band::HighCalcified);
        assert_eq!(Band::from_abpi(1.40), Band::Normal);
        assert_eq!(Band::from_abpi(1.00), Band::Normal);
        assert_eq!(Band::from_abpi(0.99), Band::Borderline);
        assert_eq!(Band::from_abpi(0.91), Band::Borderline);
        assert_eq!(Band::from_abpi(0.90), Band::MildModeratePad);
        assert_eq!(Band::from_abpi(0.50), Band::MildModeratePad);
        assert_eq!(Band::from_abpi(0.49), Band::SeverePad);
    }

    #[test]
    fn rejects_non_positive_and_non_finite() {
        assert!(compute(&input(0.0, 120.0, 120.0, 120.0)).is_err());
        assert!(compute(&input(120.0, -1.0, 120.0, 120.0)).is_err());
        assert!(compute(&input(120.0, 120.0, 0.0, 120.0)).is_err());
        assert!(compute(&input(120.0, 120.0, 120.0, f64::NAN)).is_err());
        assert!(compute(&input(120.0, 120.0, 120.0, f64::INFINITY)).is_err());
    }

    #[test]
    fn working_map_has_expected_keys() {
        let r = build_response(&input(130.0, 80.0, 120.0, 120.0)).unwrap();
        assert!(r.working.contains_key("right_abpi"));
        assert!(r.working.contains_key("left_abpi"));
        assert!(r.working.contains_key("brachial_used"));
        assert!(r.working.contains_key("right_interpretation"));
        assert!(r.working.contains_key("left_interpretation"));
        assert_eq!(r.working["brachial_used"], json!(120.0));
    }

    #[test]
    fn schema_flags_high_abpi_safety_exclusion() {
        let schema = Abpi.input_schema();
        let def = &schema["properties"]["left_brachial_systolic"]["definition"];
        assert!(def["excludes"][0]
            .as_str()
            .unwrap()
            .contains("good perfusion"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "right_ankle_systolic": 130.0,
            "left_ankle_systolic": 80.0,
            "right_brachial_systolic": 120.0,
            "left_brachial_systolic": 110.0
        });
        let dynamic = Abpi.calculate(&value).unwrap();
        let typed = build_response(&input(130.0, 80.0, 120.0, 110.0)).unwrap();
        assert_eq!(dynamic, typed);
    }
}
