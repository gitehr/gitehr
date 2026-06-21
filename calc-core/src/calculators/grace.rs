//! GRACE - Global Registry of Acute Coronary Events risk score (Granger 2003).
//!
//! Predicts IN-HOSPITAL mortality after an acute coronary syndrome (ACS) from
//! eight admission variables. This implements the original POINT-BASED system
//! (the GRACE 1.0 nomogram of Granger et al., Arch Intern Med 2003): each
//! predictor maps to integer points via published bands, the points are summed,
//! and the total drives a low / intermediate / high risk category.
//!
//! Two things are clinically load-bearing and encoded with care:
//!
//! - Serum creatinine is accepted in mg/dL or umol/L. The GRACE creatinine
//!   bands are defined in mg/dL; UK laboratories report umol/L, so the unit is a
//!   REQUIRED input and the value is converted internally (1 mg/dL = 88.4
//!   umol/L). A wrong unit silently shifts the creatinine band - and the whole
//!   risk category - by a large margin.
//! - Killip class is a four-level clinical sign of heart failure on admission,
//!   not a free number. Its definition (I = no failure ... IV = cardiogenic
//!   shock) is carried on the input schema so a caller cannot guess.
//!
//! NOTE on the two GRACE variants: this is the discrete bracket-based GRACE 1.0
//! score (Granger 2003), NOT the later continuous GRACE 2.0 model, which uses
//! piecewise-linear interpolation and produces slightly different totals. The
//! risk-category cut-offs here are the widely-used single-set thresholds for
//! in-hospital mortality (low <=108, intermediate 109-140, high >140).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "grace";

/// Primary citation.
pub const REFERENCE: &str =
    "Granger CB, Goldberg RJ, Dabbous O, et al. Predictors of hospital mortality in the Global \
Registry of Acute Coronary Events. Arch Intern Med. 2003;163(19):2345-2353. \
doi:10.1001/archinte.163.19.2345";

/// Distribution licence: the GRACE point-based score is a published method,
/// implemented here from the primary literature (Granger 2003).
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (GRACE 1.0, Granger 2003)",
    source_url: "https://doi.org/10.1001/archinte.163.19.2345",
};

/// umol/L per mg/dL for creatinine (molar mass 113.12 g/mol). Same factor as
/// the eGFR calculator.
pub const UMOL_PER_MGDL: f64 = 88.4;

/// Unit the creatinine value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatinineUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// GRACE inputs: four physiology numbers, the creatinine unit, the Killip class,
/// and three binary admission findings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GraceInput {
    /// Age in years.
    pub age: u16,
    /// Heart rate, beats per minute.
    pub heart_rate: u16,
    /// Systolic blood pressure, mmHg.
    pub systolic_bp: u16,
    /// Serum creatinine, in the unit given by `creatinine_unit`.
    pub creatinine: f64,
    pub creatinine_unit: CreatinineUnit,
    /// Killip class on admission (1-4).
    pub killip_class: u8,
    /// Cardiac arrest at admission.
    pub cardiac_arrest_at_admission: bool,
    /// ST-segment deviation on the admission ECG.
    pub st_segment_deviation: bool,
    /// Elevated cardiac enzymes / biomarkers (e.g. troponin).
    pub elevated_cardiac_enzymes: bool,
}

/// In-hospital mortality risk category (GRACE 1.0, single-set cut-offs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskCategory {
    /// Total <=108: in-hospital mortality <1%.
    Low,
    /// Total 109-140: in-hospital mortality 1-3%.
    Intermediate,
    /// Total >140: in-hospital mortality >3%.
    High,
}

impl RiskCategory {
    fn from_total(total: u16) -> Self {
        if total <= 108 {
            RiskCategory::Low
        } else if total <= 140 {
            RiskCategory::Intermediate
        } else {
            RiskCategory::High
        }
    }

    fn slug(self) -> &'static str {
        match self {
            RiskCategory::Low => "low",
            RiskCategory::Intermediate => "intermediate",
            RiskCategory::High => "high",
        }
    }

    /// Approximate in-hospital mortality band for the category.
    fn mortality(self) -> &'static str {
        match self {
            RiskCategory::Low => "<1%",
            RiskCategory::Intermediate => "1-3%",
            RiskCategory::High => ">3%",
        }
    }
}

/// The computed outcome, with every per-predictor sub-score exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraceOutcome {
    /// Total GRACE points.
    pub total: u16,
    pub age_points: u16,
    pub heart_rate_points: u16,
    pub systolic_bp_points: u16,
    pub creatinine_points: u16,
    pub killip_points: u16,
    pub cardiac_arrest_points: u16,
    pub st_deviation_points: u16,
    pub cardiac_enzymes_points: u16,
    pub category: RiskCategory,
    pub interpretation: String,
}

/// Age bands (years). Granger 2003 in-hospital nomogram.
fn age_points(age: u16) -> u16 {
    match age {
        0..=29 => 0,
        30..=39 => 8,
        40..=49 => 25,
        50..=59 => 41,
        60..=69 => 58,
        70..=79 => 75,
        80..=89 => 91,
        _ => 100, // >=90
    }
}

/// Heart rate bands (bpm).
fn heart_rate_points(hr: u16) -> u16 {
    match hr {
        0..=49 => 0,
        50..=69 => 3,
        70..=89 => 9,
        90..=109 => 15,
        110..=149 => 24,
        150..=199 => 38,
        _ => 46, // >=200
    }
}

/// Systolic blood pressure bands (mmHg). Scored inversely: lower BP -> more points.
fn systolic_bp_points(sbp: u16) -> u16 {
    match sbp {
        0..=79 => 58,
        80..=99 => 53,
        100..=119 => 43,
        120..=139 => 34,
        140..=159 => 24,
        160..=199 => 10,
        _ => 0, // >=200
    }
}

/// Serum creatinine bands, defined in mg/dL.
fn creatinine_points(scr_mgdl: f64) -> u16 {
    // Bands: <0.40, 0.40-0.79, 0.80-1.19, 1.20-1.59, 1.60-1.99, 2.00-3.99, >=4.00.
    if scr_mgdl < 0.40 {
        1
    } else if scr_mgdl < 0.80 {
        4
    } else if scr_mgdl < 1.20 {
        7
    } else if scr_mgdl < 1.60 {
        10
    } else if scr_mgdl < 2.00 {
        13
    } else if scr_mgdl < 4.00 {
        21
    } else {
        28
    }
}

/// Killip class points (I-IV). I = no heart failure; IV = cardiogenic shock.
fn killip_points(killip_class: u8) -> Result<u16, CalcError> {
    match killip_class {
        1 => Ok(0),
        2 => Ok(20),
        3 => Ok(39),
        4 => Ok(59),
        _ => Err(CalcError::InvalidInput(
            "killip_class must be 1, 2, 3, or 4".into(),
        )),
    }
}

/// Pure scoring: the GRACE 1.0 point-based system.
pub fn compute(input: &GraceInput) -> Result<GraceOutcome, CalcError> {
    if input.creatinine <= 0.0 || !input.creatinine.is_finite() {
        return Err(CalcError::InvalidInput(
            "creatinine must be a positive number".into(),
        ));
    }
    // Guard against transcription errors rather than scoring implausible values.
    if input.age > 130 {
        return Err(CalcError::InvalidInput(
            "age must be within a plausible range (0-130 years)".into(),
        ));
    }
    if input.heart_rate > 350 {
        return Err(CalcError::InvalidInput(
            "heart_rate must be within a plausible range (0-350 bpm)".into(),
        ));
    }
    if input.systolic_bp > 320 {
        return Err(CalcError::InvalidInput(
            "systolic_bp must be within a plausible range (0-320 mmHg)".into(),
        ));
    }

    // Normalise creatinine to mg/dL, the unit the GRACE bands are defined in.
    let scr_mgdl = match input.creatinine_unit {
        CreatinineUnit::MgDl => input.creatinine,
        CreatinineUnit::UmolL => input.creatinine / UMOL_PER_MGDL,
    };

    let age_points = age_points(input.age);
    let heart_rate_points = heart_rate_points(input.heart_rate);
    let systolic_bp_points = systolic_bp_points(input.systolic_bp);
    let creatinine_points = creatinine_points(scr_mgdl);
    let killip_points = killip_points(input.killip_class)?;
    let cardiac_arrest_points = if input.cardiac_arrest_at_admission { 39 } else { 0 };
    let st_deviation_points = if input.st_segment_deviation { 28 } else { 0 };
    let cardiac_enzymes_points = if input.elevated_cardiac_enzymes { 14 } else { 0 };

    let total = age_points
        + heart_rate_points
        + systolic_bp_points
        + creatinine_points
        + killip_points
        + cardiac_arrest_points
        + st_deviation_points
        + cardiac_enzymes_points;

    let category = RiskCategory::from_total(total);

    let interpretation = format!(
        "GRACE {total} ({} risk): predicted in-hospital mortality {} for acute coronary syndrome. \
Cut-offs: low <=108, intermediate 109-140, high >140. This is the GRACE 1.0 point-based score \
(Granger 2003) and estimates risk only; it does not by itself dictate management. The continuous \
GRACE 2.0 model gives a precise mortality estimate and slightly different totals.",
        category.slug(),
        category.mortality(),
    );

    Ok(GraceOutcome {
        total,
        age_points,
        heart_rate_points,
        systolic_bp_points,
        creatinine_points,
        killip_points,
        cardiac_arrest_points,
        st_deviation_points,
        cardiac_enzymes_points,
        category,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &GraceInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("age_points".into(), json!(o.age_points));
    working.insert("heart_rate_points".into(), json!(o.heart_rate_points));
    working.insert("systolic_bp_points".into(), json!(o.systolic_bp_points));
    working.insert("creatinine_points".into(), json!(o.creatinine_points));
    working.insert("killip_points".into(), json!(o.killip_points));
    working.insert("cardiac_arrest_points".into(), json!(o.cardiac_arrest_points));
    working.insert("st_deviation_points".into(), json!(o.st_deviation_points));
    working.insert("cardiac_enzymes_points".into(), json!(o.cardiac_enzymes_points));
    working.insert("risk_category".into(), json!(o.category.slug()));
    working.insert("in_hospital_mortality".into(), json!(o.category.mortality()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Grace;

impl Calculator for Grace {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "GRACE ACS Risk Score (in-hospital mortality)"
    }

    fn description(&self) -> &'static str {
        "Point-based GRACE 1.0 score (Granger 2003) estimating in-hospital mortality risk in acute coronary syndrome."
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
            "title": "GraceInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "heart_rate", "systolic_bp", "creatinine", "creatinine_unit",
                "killip_class", "cardiac_arrest_at_admission", "st_segment_deviation",
                "elevated_cardiac_enzymes"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 130,
                    "description": "Age in years (banded every decade; <30 scores 0, >=90 scores 100)"
                },
                "heart_rate": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 350,
                    "description": "Heart rate, bpm (<50 scores 0, >=200 scores 46)"
                },
                "systolic_bp": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 320,
                    "description": "Systolic blood pressure, mmHg (scored inversely: <80 scores 58, >=200 scores 0)"
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
                        "statement": "The GRACE creatinine bands are defined in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 88.4 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~88x and the wrong label silently moves the creatinine band and the whole risk category"
                        ],
                        "source": {
                            "citation": "Granger CB et al. Arch Intern Med. 2003;163(19):2345-2353.",
                            "url": "https://doi.org/10.1001/archinte.163.19.2345"
                        },
                        "status": "draft"
                    }
                },
                "killip_class": {
                    "type": "integer",
                    "enum": [1, 2, 3, 4],
                    "description": "Killip class on admission (1-4)",
                    "definition": {
                        "concept": "Killip class",
                        "statement": "A four-level clinical classification of heart failure on admission in acute coronary syndrome.",
                        "includes": [
                            "Class I: no clinical signs of heart failure (scores 0)",
                            "Class II: rales/crackles in the lungs, an S3 gallop, or raised jugular venous pressure (scores 20)",
                            "Class III: frank acute pulmonary oedema (scores 39)",
                            "Class IV: cardiogenic shock or hypotension (systolic BP <90 mmHg) with peripheral vasoconstriction (scores 59)"
                        ],
                        "source": {
                            "citation": "Killip T, Kimball JT. Am J Cardiol. 1967;20(4):457-464; used as a GRACE predictor in Granger CB et al. Arch Intern Med. 2003.",
                            "url": "https://doi.org/10.1001/archinte.163.19.2345"
                        },
                        "status": "draft"
                    }
                },
                "cardiac_arrest_at_admission": {
                    "type": "boolean",
                    "description": "Cardiac arrest at admission (scores 39)"
                },
                "st_segment_deviation": {
                    "type": "boolean",
                    "description": "ST-segment deviation on the admission ECG (scores 28)"
                },
                "elevated_cardiac_enzymes": {
                    "type": "boolean",
                    "description": "Elevated cardiac enzymes / biomarkers, e.g. troponin (scores 14)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: GraceInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> GraceInput {
        GraceInput {
            age: 65,
            heart_rate: 80,
            systolic_bp: 130,
            creatinine: 1.2,
            creatinine_unit: CreatinineUnit::MgDl,
            killip_class: 1,
            cardiac_arrest_at_admission: false,
            st_segment_deviation: true,
            elevated_cardiac_enzymes: true,
        }
    }

    /// Validated worked example A (cross-checked against the mdapp.co bracket
    /// calculator and the Granger 2003 / wikidoc point tables).
    ///
    /// Age 65 (58) + HR 80 (9) + SBP 130 (34) + Cr 1.2 mg/dL (10) + Killip I (0)
    /// + no arrest (0) + ST deviation (28) + elevated enzymes (14) = 153 -> high.
    #[test]
    fn worked_example_a() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.age_points, 58);
        assert_eq!(o.heart_rate_points, 9);
        assert_eq!(o.systolic_bp_points, 34);
        assert_eq!(o.creatinine_points, 10);
        assert_eq!(o.killip_points, 0);
        assert_eq!(o.cardiac_arrest_points, 0);
        assert_eq!(o.st_deviation_points, 28);
        assert_eq!(o.cardiac_enzymes_points, 14);
        assert_eq!(o.total, 153);
        assert_eq!(o.category, RiskCategory::High);
    }

    /// Validated worked example B (cross-checked against the mdapp.co bracket
    /// calculator and the Granger 2003 / wikidoc point tables).
    ///
    /// Age 78 (75) + HR 115 (24) + SBP 95 (53) + Cr 2.5 mg/dL (21) + Killip III
    /// (39) + no arrest (0) + ST deviation (28) + elevated enzymes (14) = 254 -> high.
    #[test]
    fn worked_example_b() {
        let i = GraceInput {
            age: 78,
            heart_rate: 115,
            systolic_bp: 95,
            creatinine: 2.5,
            creatinine_unit: CreatinineUnit::MgDl,
            killip_class: 3,
            cardiac_arrest_at_admission: false,
            st_segment_deviation: true,
            elevated_cardiac_enzymes: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.age_points, 75);
        assert_eq!(o.heart_rate_points, 24);
        assert_eq!(o.systolic_bp_points, 53);
        assert_eq!(o.creatinine_points, 21);
        assert_eq!(o.killip_points, 39);
        assert_eq!(o.st_deviation_points, 28);
        assert_eq!(o.cardiac_enzymes_points, 14);
        assert_eq!(o.total, 254);
        assert_eq!(o.category, RiskCategory::High);
    }

    /// A genuinely low-risk young patient: every band at or near its floor.
    #[test]
    fn low_risk_floor() {
        let i = GraceInput {
            age: 25,
            heart_rate: 60,
            systolic_bp: 130,
            creatinine: 0.9,
            creatinine_unit: CreatinineUnit::MgDl,
            killip_class: 1,
            cardiac_arrest_at_admission: false,
            st_segment_deviation: false,
            elevated_cardiac_enzymes: false,
        };
        // 0 + 3 + 34 + 7 + 0 = 44.
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 44);
        assert_eq!(o.category, RiskCategory::Low);
    }

    #[test]
    fn age_bands() {
        assert_eq!(age_points(29), 0);
        assert_eq!(age_points(30), 8);
        assert_eq!(age_points(39), 8);
        assert_eq!(age_points(40), 25);
        assert_eq!(age_points(49), 25);
        assert_eq!(age_points(50), 41);
        assert_eq!(age_points(59), 41);
        assert_eq!(age_points(60), 58);
        assert_eq!(age_points(69), 58);
        assert_eq!(age_points(70), 75);
        assert_eq!(age_points(79), 75);
        assert_eq!(age_points(80), 91);
        assert_eq!(age_points(89), 91);
        assert_eq!(age_points(90), 100);
        assert_eq!(age_points(105), 100);
    }

    #[test]
    fn heart_rate_bands() {
        assert_eq!(heart_rate_points(49), 0);
        assert_eq!(heart_rate_points(50), 3);
        assert_eq!(heart_rate_points(69), 3);
        assert_eq!(heart_rate_points(70), 9);
        assert_eq!(heart_rate_points(89), 9);
        assert_eq!(heart_rate_points(90), 15);
        assert_eq!(heart_rate_points(109), 15);
        assert_eq!(heart_rate_points(110), 24);
        assert_eq!(heart_rate_points(149), 24);
        assert_eq!(heart_rate_points(150), 38);
        assert_eq!(heart_rate_points(199), 38);
        assert_eq!(heart_rate_points(200), 46);
        assert_eq!(heart_rate_points(250), 46);
    }

    #[test]
    fn systolic_bp_bands() {
        assert_eq!(systolic_bp_points(79), 58);
        assert_eq!(systolic_bp_points(80), 53);
        assert_eq!(systolic_bp_points(99), 53);
        assert_eq!(systolic_bp_points(100), 43);
        assert_eq!(systolic_bp_points(119), 43);
        assert_eq!(systolic_bp_points(120), 34);
        assert_eq!(systolic_bp_points(139), 34);
        assert_eq!(systolic_bp_points(140), 24);
        assert_eq!(systolic_bp_points(159), 24);
        assert_eq!(systolic_bp_points(160), 10);
        assert_eq!(systolic_bp_points(199), 10);
        assert_eq!(systolic_bp_points(200), 0);
        assert_eq!(systolic_bp_points(250), 0);
    }

    #[test]
    fn creatinine_bands_mgdl() {
        assert_eq!(creatinine_points(0.30), 1);
        assert_eq!(creatinine_points(0.39), 1);
        assert_eq!(creatinine_points(0.40), 4);
        assert_eq!(creatinine_points(0.79), 4);
        assert_eq!(creatinine_points(0.80), 7);
        assert_eq!(creatinine_points(1.19), 7);
        assert_eq!(creatinine_points(1.20), 10);
        assert_eq!(creatinine_points(1.59), 10);
        assert_eq!(creatinine_points(1.60), 13);
        assert_eq!(creatinine_points(1.99), 13);
        assert_eq!(creatinine_points(2.00), 21);
        assert_eq!(creatinine_points(3.99), 21);
        assert_eq!(creatinine_points(4.00), 28);
        assert_eq!(creatinine_points(8.0), 28);
    }

    #[test]
    fn killip_bands() {
        assert_eq!(killip_points(1).unwrap(), 0);
        assert_eq!(killip_points(2).unwrap(), 20);
        assert_eq!(killip_points(3).unwrap(), 39);
        assert_eq!(killip_points(4).unwrap(), 59);
        assert!(killip_points(0).is_err());
        assert!(killip_points(5).is_err());
    }

    #[test]
    fn binary_predictor_points() {
        let mut i = base();
        i.st_segment_deviation = false;
        i.elevated_cardiac_enzymes = false;
        i.cardiac_arrest_at_admission = false;
        let o = compute(&i).unwrap();
        assert_eq!(o.st_deviation_points, 0);
        assert_eq!(o.cardiac_enzymes_points, 0);
        assert_eq!(o.cardiac_arrest_points, 0);

        i.cardiac_arrest_at_admission = true;
        assert_eq!(compute(&i).unwrap().cardiac_arrest_points, 39);
    }

    #[test]
    fn umol_per_l_matches_equivalent_mgdl() {
        // 1.2 mg/dL == 106.08 umol/L; both must land in the same creatinine band.
        let a = compute(&base()).unwrap();
        let mut i = base();
        i.creatinine = 1.2 * UMOL_PER_MGDL;
        i.creatinine_unit = CreatinineUnit::UmolL;
        let b = compute(&i).unwrap();
        assert_eq!(a.creatinine_points, b.creatinine_points);
        assert_eq!(a.total, b.total);
    }

    #[test]
    fn category_boundaries() {
        assert_eq!(RiskCategory::from_total(0), RiskCategory::Low);
        assert_eq!(RiskCategory::from_total(108), RiskCategory::Low);
        assert_eq!(RiskCategory::from_total(109), RiskCategory::Intermediate);
        assert_eq!(RiskCategory::from_total(140), RiskCategory::Intermediate);
        assert_eq!(RiskCategory::from_total(141), RiskCategory::High);
    }

    #[test]
    fn rejects_bad_input() {
        let mut i = base();
        i.creatinine = 0.0;
        assert!(compute(&i).is_err());

        let mut i = base();
        i.creatinine = -1.0;
        assert!(compute(&i).is_err());

        let mut i = base();
        i.creatinine = f64::NAN;
        assert!(compute(&i).is_err());

        let mut i = base();
        i.killip_class = 0;
        assert!(compute(&i).is_err());

        let mut i = base();
        i.age = 200;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 65, "heart_rate": 80, "systolic_bp": 130,
            "creatinine": 1.2, "creatinine_unit": "mg/dL", "killip_class": 1,
            "cardiac_arrest_at_admission": false, "st_segment_deviation": true,
            "elevated_cardiac_enzymes": true
        });
        let dynamic = Grace.calculate(&value).unwrap();
        let typed = build_response(&base()).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(153));
    }

    #[test]
    fn schema_flags_unit_exclusion() {
        let schema = Grace.input_schema();
        let def = &schema["properties"]["creatinine_unit"]["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("88x"));
    }

    #[test]
    fn schema_defines_killip() {
        let schema = Grace.input_schema();
        let def = &schema["properties"]["killip_class"]["definition"];
        assert_eq!(def["concept"], "Killip class");
        assert!(def["includes"][0].as_str().unwrap().contains("Class I"));
        assert!(def["includes"][3].as_str().unwrap().contains("shock"));
    }

    #[test]
    fn working_map_has_every_subscore() {
        let resp = build_response(&base()).unwrap();
        for key in [
            "total_score",
            "age_points",
            "heart_rate_points",
            "systolic_bp_points",
            "creatinine_points",
            "killip_points",
            "cardiac_arrest_points",
            "st_deviation_points",
            "cardiac_enzymes_points",
            "risk_category",
            "in_hospital_mortality",
        ] {
            assert!(resp.working.contains_key(key), "missing {key}");
        }
    }
}
