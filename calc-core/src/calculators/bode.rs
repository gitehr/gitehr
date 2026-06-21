//! BODE Index - multidimensional prognosis in COPD.
//!
//! The BODE index (Celli et al., NEJM 2004) predicts ~4-year survival in COPD
//! better than FEV1 alone, by combining four domains: Body-mass index (B),
//! airflow Obstruction (O, FEV1 % predicted), Dyspnoea (D), and Exercise
//! capacity (E, six-minute walk distance). Each domain contributes a sub-score
//! and the total ranges 0-10; higher is worse.
//!
//! A common error is the dyspnoea component: BODE uses the MODIFIED MRC scale
//! (mMRC, graded 0-4), NOT the classic MRC scale (graded 1-5) in routine UK
//! use. The two carry the same descriptors offset by one (mMRC = MRC - 1), so a
//! grade quoted without its scale is ambiguous and feeding a classic 1-5 grade
//! into BODE over-scores dyspnoea by a band. This input is the mMRC 0-4 grade;
//! the ambiguity is flagged in the schema `definition` for that field.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "bode";

/// Distribution licence: the BODE index is a published clinical method,
/// implemented here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (BODE index, Celli et al. 2004)",
    source_url: "https://doi.org/10.1056/NEJMoa021322",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Celli BR, Cote CG, Marin JM, et al. The body-mass index, airflow obstruction, dyspnea, and \
exercise capacity index in chronic obstructive pulmonary disease. N Engl J Med. \
2004;350(10):1005-1012. doi:10.1056/NEJMoa021322";

/// Highest valid mMRC dyspnoea grade.
pub const MAX_MMRC: u8 = 4;

/// BODE index inputs. All four domains are required.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodeInput {
    /// Body-mass index, kg/m^2 (B).
    pub bmi: f64,
    /// Post-bronchodilator FEV1 as a percentage of predicted (O).
    pub fev1_percent_predicted: f64,
    /// Dyspnoea on the MODIFIED MRC scale, an integer 0-4 (D).
    pub mmrc_dyspnoea: u8,
    /// Six-minute walk distance, metres (E).
    pub six_minute_walk_distance_m: f64,
}

/// Four-year survival quartile (Celli et al. 2004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quartile {
    /// BODE 0-2.
    One,
    /// BODE 3-4.
    Two,
    /// BODE 5-6.
    Three,
    /// BODE 7-10.
    Four,
}

impl Quartile {
    fn from_score(score: u8) -> Self {
        if score <= 2 {
            Quartile::One
        } else if score <= 4 {
            Quartile::Two
        } else if score <= 6 {
            Quartile::Three
        } else {
            Quartile::Four
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Quartile::One => "0-2",
            Quartile::Two => "3-4",
            Quartile::Three => "5-6",
            Quartile::Four => "7-10",
        }
    }

    /// Approximate four-year survival for the quartile (Celli et al. 2004,
    /// Figure 3 / Table 3): 80%, 67%, 57%, 18% respectively.
    fn approx_four_year_survival_percent(self) -> u8 {
        match self {
            Quartile::One => 80,
            Quartile::Two => 67,
            Quartile::Three => 57,
            Quartile::Four => 18,
        }
    }
}

/// The computed outcome, with each domain's sub-score exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodeOutcome {
    /// Total BODE index (0-10).
    pub score: u8,
    /// Body-mass index sub-score (0-1).
    pub bmi_points: u8,
    /// Airflow-obstruction (FEV1) sub-score (0-3).
    pub fev1_points: u8,
    /// Dyspnoea (mMRC) sub-score (0-3).
    pub dyspnoea_points: u8,
    /// Exercise-capacity (6MWD) sub-score (0-3).
    pub exercise_points: u8,
    pub quartile: Quartile,
    pub interpretation: String,
}

/// B - BMI: >21 scores 0, <=21 scores 1.
fn bmi_points(bmi: f64) -> u8 {
    u8::from(bmi <= 21.0)
}

/// O - airflow obstruction by FEV1 % predicted.
fn fev1_points(fev1_pct: f64) -> u8 {
    if fev1_pct >= 65.0 {
        0
    } else if fev1_pct >= 50.0 {
        1
    } else if fev1_pct >= 36.0 {
        2
    } else {
        3
    }
}

/// D - dyspnoea by mMRC grade (0-4).
fn dyspnoea_points(mmrc: u8) -> u8 {
    match mmrc {
        0 | 1 => 0,
        2 => 1,
        3 => 2,
        _ => 3, // grade 4; input is validated to 0-4 before this is called.
    }
}

/// E - exercise capacity by six-minute walk distance (metres).
fn exercise_points(distance_m: f64) -> u8 {
    if distance_m >= 350.0 {
        0
    } else if distance_m >= 250.0 {
        1
    } else if distance_m >= 150.0 {
        2
    } else {
        3
    }
}

/// Pure scoring.
pub fn compute(input: &BodeInput) -> Result<BodeOutcome, CalcError> {
    if !input.bmi.is_finite() || input.bmi <= 0.0 {
        return Err(CalcError::InvalidInput(
            "bmi must be a positive number".into(),
        ));
    }
    if !input.fev1_percent_predicted.is_finite() || input.fev1_percent_predicted < 0.0 {
        return Err(CalcError::InvalidInput(
            "fev1_percent_predicted must be a non-negative number".into(),
        ));
    }
    if !input.six_minute_walk_distance_m.is_finite() || input.six_minute_walk_distance_m < 0.0 {
        return Err(CalcError::InvalidInput(
            "six_minute_walk_distance_m must be a non-negative number".into(),
        ));
    }
    if input.mmrc_dyspnoea > MAX_MMRC {
        return Err(CalcError::InvalidInput(format!(
            "mmrc_dyspnoea must be an integer from 0 to {MAX_MMRC} (the modified MRC scale), got {}. \
The classic MRC scale (1-5) is a different instrument - subtract 1 to convert",
            input.mmrc_dyspnoea
        )));
    }

    let bmi_points = bmi_points(input.bmi);
    let fev1_points = fev1_points(input.fev1_percent_predicted);
    let dyspnoea_points = dyspnoea_points(input.mmrc_dyspnoea);
    let exercise_points = exercise_points(input.six_minute_walk_distance_m);

    let score = bmi_points + fev1_points + dyspnoea_points + exercise_points;
    let quartile = Quartile::from_score(score);

    let interpretation = format!(
        "BODE index {score} of 10 (quartile {}). Approximate four-year survival ~{}% (Celli et al. \
2004). A higher index predicts worse survival and hospitalisation risk in COPD, integrating body \
mass, airflow obstruction, dyspnoea, and exercise capacity. The dyspnoea component is the modified \
MRC (mMRC, 0-4), not the classic MRC (1-5). This is a prognostic index, not a diagnostic or staging \
tool, and assumes stable COPD with the spirometry, mMRC grade, and a standardised six-minute walk \
all to hand.",
        quartile.slug(),
        quartile.approx_four_year_survival_percent()
    );

    Ok(BodeOutcome {
        score,
        bmi_points,
        fev1_points,
        dyspnoea_points,
        exercise_points,
        quartile,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &BodeInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("bmi_points".into(), json!(o.bmi_points));
    working.insert("fev1_points".into(), json!(o.fev1_points));
    working.insert("dyspnoea_points".into(), json!(o.dyspnoea_points));
    working.insert("exercise_points".into(), json!(o.exercise_points));
    working.insert("survival_quartile".into(), json!(o.quartile.slug()));
    working.insert(
        "approx_four_year_survival_percent".into(),
        json!(o.quartile.approx_four_year_survival_percent()),
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Bode;

impl Calculator for Bode {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "BODE Index (COPD prognosis)"
    }

    fn description(&self) -> &'static str {
        "Multidimensional prognostic index in COPD from BMI, FEV1, mMRC dyspnoea, and six-minute walk distance; predicts ~4-year survival."
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
            "title": "BodeInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "bmi", "fev1_percent_predicted", "mmrc_dyspnoea", "six_minute_walk_distance_m"
            ],
            "properties": {
                "bmi": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Body-mass index in kg/m^2 (B); >21 scores 0, <=21 scores 1"
                },
                "fev1_percent_predicted": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Post-bronchodilator FEV1 as % predicted (O); >=65 -> 0, 50-64 -> 1, 36-49 -> 2, <=35 -> 3",
                    "definition": {
                        "concept": "Airflow obstruction (O)",
                        "statement": "Forced expiratory volume in 1 second expressed as a percentage of the predicted value, measured post-bronchodilator.",
                        "caveats": "Use the post-bronchodilator FEV1 % predicted; the predicted value depends on the reference equation used (e.g. GLI). This is FEV1 % predicted, NOT the FEV1/FVC ratio.",
                        "source": {
                            "citation": "Celli BR et al. N Engl J Med. 2004;350(10):1005-1012.",
                            "url": "https://doi.org/10.1056/NEJMoa021322"
                        },
                        "status": "draft"
                    }
                },
                "mmrc_dyspnoea": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 4,
                    "description": "Dyspnoea on the MODIFIED MRC scale, 0-4 (D); 0-1 -> 0, 2 -> 1, 3 -> 2, 4 -> 3",
                    "definition": {
                        "concept": "Dyspnoea grade (D) - modified MRC (mMRC)",
                        "statement": "Breathlessness graded on the modified Medical Research Council (mMRC) scale, an integer from 0 (breathless only on strenuous exertion) to 4 (too breathless to leave the house or breathless when dressing).",
                        "includes": ["The modified MRC scale graded 0-4"],
                        "excludes": [
                            "The classic MRC dyspnoea scale graded 1-5 is a DIFFERENT instrument and must NOT be entered here: BODE uses the modified scale (mMRC = classic MRC grade - 1), and entering a classic 1-5 grade over-scores dyspnoea by one band"
                        ],
                        "caveats": "The same descriptors are renumbered between the two scales, so a grade quoted without naming its scale is ambiguous; confirm it is the mMRC (0-4).",
                        "source": {
                            "citation": "Celli BR et al. N Engl J Med. 2004;350(10):1005-1012.",
                            "url": "https://doi.org/10.1056/NEJMoa021322"
                        },
                        "status": "draft"
                    }
                },
                "six_minute_walk_distance_m": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Six-minute walk distance in metres (E); >=350 -> 0, 250-349 -> 1, 150-249 -> 2, <=149 -> 3",
                    "definition": {
                        "concept": "Exercise capacity (E)",
                        "statement": "Distance walked, in metres, during a standardised six-minute walk test (6MWT).",
                        "caveats": "Distance is in metres, not feet or yards. The 6MWT should follow a standardised protocol (e.g. ATS/ERS); the original cohort used the distance in metres.",
                        "source": {
                            "citation": "Celli BR et al. N Engl J Med. 2004;350(10):1005-1012.",
                            "url": "https://doi.org/10.1056/NEJMoa021322"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: BodeInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(bmi: f64, fev1: f64, mmrc: u8, walk: f64) -> BodeInput {
        BodeInput {
            bmi,
            fev1_percent_predicted: fev1,
            mmrc_dyspnoea: mmrc,
            six_minute_walk_distance_m: walk,
        }
    }

    #[test]
    fn worked_example_low_score() {
        // BMI 25 (0) + FEV1 70% (0) + mMRC 1 (0) + 6MWD 400m (0) = 0, quartile 0-2.
        let o = compute(&input(25.0, 70.0, 1, 400.0)).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.bmi_points, 0);
        assert_eq!(o.fev1_points, 0);
        assert_eq!(o.dyspnoea_points, 0);
        assert_eq!(o.exercise_points, 0);
        assert_eq!(o.quartile, Quartile::One);
        assert_eq!(o.quartile.approx_four_year_survival_percent(), 80);
    }

    #[test]
    fn worked_example_mixed() {
        // BMI 20 (1) + FEV1 45% (2) + mMRC 3 (2) + 6MWD 200m (2) = 7, quartile 7-10.
        let o = compute(&input(20.0, 45.0, 3, 200.0)).unwrap();
        assert_eq!(o.bmi_points, 1);
        assert_eq!(o.fev1_points, 2);
        assert_eq!(o.dyspnoea_points, 2);
        assert_eq!(o.exercise_points, 2);
        assert_eq!(o.score, 7);
        assert_eq!(o.quartile, Quartile::Four);
        assert_eq!(o.quartile.approx_four_year_survival_percent(), 18);
    }

    #[test]
    fn maximum_score_is_ten() {
        // BMI <=21 (1) + FEV1 <36 (3) + mMRC 4 (3) + 6MWD <150 (3) = 10.
        let o = compute(&input(18.0, 30.0, 4, 100.0)).unwrap();
        assert_eq!(o.score, 10);
        assert_eq!(o.quartile, Quartile::Four);
    }

    #[test]
    fn bmi_boundary() {
        assert_eq!(bmi_points(21.1), 0);
        assert_eq!(bmi_points(21.0), 1); // <=21 scores 1
        assert_eq!(bmi_points(20.9), 1);
    }

    #[test]
    fn fev1_boundaries() {
        assert_eq!(fev1_points(65.0), 0);
        assert_eq!(fev1_points(64.0), 1);
        assert_eq!(fev1_points(50.0), 1);
        assert_eq!(fev1_points(49.0), 2);
        assert_eq!(fev1_points(36.0), 2);
        assert_eq!(fev1_points(35.0), 3);
        assert_eq!(fev1_points(0.0), 3);
    }

    #[test]
    fn dyspnoea_boundaries() {
        assert_eq!(dyspnoea_points(0), 0);
        assert_eq!(dyspnoea_points(1), 0);
        assert_eq!(dyspnoea_points(2), 1);
        assert_eq!(dyspnoea_points(3), 2);
        assert_eq!(dyspnoea_points(4), 3);
    }

    #[test]
    fn exercise_boundaries() {
        assert_eq!(exercise_points(350.0), 0);
        assert_eq!(exercise_points(349.0), 1);
        assert_eq!(exercise_points(250.0), 1);
        assert_eq!(exercise_points(249.0), 2);
        assert_eq!(exercise_points(150.0), 2);
        assert_eq!(exercise_points(149.0), 3);
        assert_eq!(exercise_points(0.0), 3);
    }

    #[test]
    fn quartile_boundaries() {
        assert_eq!(Quartile::from_score(0), Quartile::One);
        assert_eq!(Quartile::from_score(2), Quartile::One);
        assert_eq!(Quartile::from_score(3), Quartile::Two);
        assert_eq!(Quartile::from_score(4), Quartile::Two);
        assert_eq!(Quartile::from_score(5), Quartile::Three);
        assert_eq!(Quartile::from_score(6), Quartile::Three);
        assert_eq!(Quartile::from_score(7), Quartile::Four);
        assert_eq!(Quartile::from_score(10), Quartile::Four);
    }

    #[test]
    fn rejects_out_of_range_mmrc() {
        // mMRC is 0-4; a classic MRC grade of 5 is the most common bad input.
        let err = compute(&input(25.0, 70.0, 5, 400.0)).unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }

    #[test]
    fn rejects_bad_numeric_input() {
        assert!(compute(&input(0.0, 70.0, 1, 400.0)).is_err());
        assert!(compute(&input(-1.0, 70.0, 1, 400.0)).is_err());
        assert!(compute(&input(25.0, -1.0, 1, 400.0)).is_err());
        assert!(compute(&input(25.0, 70.0, 1, -1.0)).is_err());
        assert!(compute(&input(f64::NAN, 70.0, 1, 400.0)).is_err());
    }

    #[test]
    fn schema_flags_mmrc_not_classic_mrc() {
        let schema = Bode.input_schema();
        let excludes = &schema["properties"]["mmrc_dyspnoea"]["definition"]["excludes"];
        assert!(excludes[0].as_str().unwrap().contains("classic MRC"));
        assert_eq!(schema["properties"]["mmrc_dyspnoea"]["maximum"], json!(4));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "bmi": 20.0,
            "fev1_percent_predicted": 45.0,
            "mmrc_dyspnoea": 3,
            "six_minute_walk_distance_m": 200.0
        });
        let dynamic = Bode.calculate(&value).unwrap();
        let typed = build_response(&input(20.0, 45.0, 3, 200.0)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(7));
    }
}
