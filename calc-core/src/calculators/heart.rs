//! HEART Score - risk of major adverse cardiac events (MACE) in ED chest pain.
//!
//! Stratifies patients presenting to the emergency department with chest pain by
//! their 6-week risk of MACE (acute MI, PCI, CABG, or death), to guide discharge
//! versus admission versus early invasive management (Six AJ et al. 2008;
//! validated in Backus BE et al. 2013).
//!
//! Five components - History, ECG, Age, Risk factors, Troponin - each scored 0,
//! 1, or 2, for a total of 0-10. Two of these are encoded with input definitions
//! because they are easy to score wrongly:
//! - History is a subjective gestalt of clinical suspicion, not a checklist.
//! - Risk factors are counted, but a *history of atherosclerotic disease* short-
//!   circuits the count and scores the maximum 2 regardless of how many of the
//!   listed factors are present.
//!
//! Age is a single numeric input mapping to the three mutually-exclusive bands
//! (<45 = 0, 45-64 = 1, >=65 = 2), so contradictory age inputs are impossible.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "heart";

/// Primary citation.
pub const REFERENCE: &str =
    "Six AJ, Backus BE, Kelder JC. Chest pain in the emergency room: value of the HEART score. \
Neth Heart J. 2008;16(6):191-196. Validated in Backus BE, Six AJ, Kelder JC, et al. A prospective \
validation of the HEART score for chest pain patients at the emergency department. Int J Cardiol. \
2013;168(3):2153-2158.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1007/BF03086144",
};

/// History (clinical suspicion) component - a subjective gestalt of how
/// suspicious the presentation is for an acute coronary syndrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum History {
    /// Slightly suspicious (0 points).
    SlightlySuspicious,
    /// Moderately suspicious (1 point).
    ModeratelySuspicious,
    /// Highly suspicious (2 points).
    HighlySuspicious,
}

impl History {
    fn points(self) -> u8 {
        match self {
            History::SlightlySuspicious => 0,
            History::ModeratelySuspicious => 1,
            History::HighlySuspicious => 2,
        }
    }
}

/// ECG component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ecg {
    /// Normal (0 points).
    Normal,
    /// Non-specific repolarisation disturbance, e.g. LBBB, LVH, digoxin effect,
    /// or unchanged repolarisation abnormalities (1 point).
    NonSpecificRepolarisation,
    /// Significant ST deviation not due to LBBB, LVH, or digoxin (2 points).
    SignificantStDeviation,
}

impl Ecg {
    fn points(self) -> u8 {
        match self {
            Ecg::Normal => 0,
            Ecg::NonSpecificRepolarisation => 1,
            Ecg::SignificantStDeviation => 2,
        }
    }
}

/// Risk-factor burden for coronary artery disease.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskFactors {
    /// No known risk factors (0 points).
    None,
    /// One or two risk factors (1 point).
    OneOrTwo,
    /// Three or more risk factors, OR a history of atherosclerotic disease,
    /// which scores 2 regardless of the count (2 points).
    ThreeOrMoreOrAtherosclerosis,
}

impl RiskFactors {
    fn points(self) -> u8 {
        match self {
            RiskFactors::None => 0,
            RiskFactors::OneOrTwo => 1,
            RiskFactors::ThreeOrMoreOrAtherosclerosis => 2,
        }
    }
}

/// Initial troponin relative to the assay's upper reference (normal) limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Troponin {
    /// At or below the normal limit (0 points).
    Normal,
    /// One to three times the normal limit (1 point).
    OneToThreeTimes,
    /// Greater than three times the normal limit (2 points).
    OverThreeTimes,
}

impl Troponin {
    fn points(self) -> u8 {
        match self {
            Troponin::Normal => 0,
            Troponin::OneToThreeTimes => 1,
            Troponin::OverThreeTimes => 2,
        }
    }
}

/// HEART Score inputs. Age is numeric; the three age bands are derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartInput {
    /// Clinical suspicion from the history (subjective gestalt).
    pub history: History,
    /// ECG findings.
    pub ecg: Ecg,
    /// Age in years.
    pub age: u8,
    /// Coronary risk-factor burden (or history of atherosclerotic disease).
    pub risk_factors: RiskFactors,
    /// Initial troponin relative to the assay's normal limit.
    pub troponin: Troponin,
}

/// Management band by total score (Backus BE et al. 2013).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Score 0-3: low risk (~1.7% MACE); consider discharge.
    Low,
    /// Score 4-6: moderate risk (~16.6% MACE); admit for observation.
    Moderate,
    /// Score 7-10: high risk (~50.1% MACE); early invasive strategy.
    High,
}

impl RiskBand {
    fn slug(self) -> &'static str {
        match self {
            RiskBand::Low => "low",
            RiskBand::Moderate => "moderate",
            RiskBand::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartOutcome {
    /// Total score (0-10).
    pub score: u8,
    /// Points contributed by the age band (0, 1, or 2).
    pub age_points: u8,
    pub band: RiskBand,
    pub interpretation: String,
}

fn age_points(age: u8) -> u8 {
    if age >= 65 {
        2
    } else if age >= 45 {
        1
    } else {
        0
    }
}

/// Pure scoring.
pub fn compute(input: &HeartInput) -> Result<HeartOutcome, CalcError> {
    let age_points = age_points(input.age);

    let score = input.history.points()
        + input.ecg.points()
        + age_points
        + input.risk_factors.points()
        + input.troponin.points();

    let band = if score <= 3 {
        RiskBand::Low
    } else if score <= 6 {
        RiskBand::Moderate
    } else {
        RiskBand::High
    };

    let interpretation = match band {
        RiskBand::Low => format!(
            "Score {score}: low risk (~1.7% 6-week MACE). Supports discharge from the ED with \
appropriate safety-netting (Backus BE et al. 2013)."
        ),
        RiskBand::Moderate => format!(
            "Score {score}: moderate risk (~16.6% 6-week MACE). Admit for clinical observation and \
further investigation (Backus BE et al. 2013)."
        ),
        RiskBand::High => format!(
            "Score {score}: high risk (~50.1% 6-week MACE). Supports an early invasive strategy \
(Backus BE et al. 2013)."
        ),
    };

    Ok(HeartOutcome {
        score,
        age_points,
        band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &HeartInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("history".into(), json!(input.history.points()));
    working.insert("ecg".into(), json!(input.ecg.points()));
    working.insert("age".into(), json!(o.age_points));
    working.insert("risk_factors".into(), json!(input.risk_factors.points()));
    working.insert("troponin".into(), json!(input.troponin.points()));
    working.insert("total_score".into(), json!(o.score));
    working.insert("risk_band".into(), json!(o.band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Heart;

impl Calculator for Heart {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "HEART Score (ED Chest Pain)"
    }

    fn description(&self) -> &'static str {
        "6-week MACE risk for emergency department chest pain, guiding discharge versus admission \
versus early invasive management (Six AJ et al. 2008)."
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
            "title": "HeartInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["history", "ecg", "age", "risk_factors", "troponin"],
            "properties": {
                "history": {
                    "type": "string",
                    "enum": ["slightly_suspicious", "moderately_suspicious", "highly_suspicious"],
                    "description": "Clinical suspicion from the history: slightly_suspicious=0, moderately_suspicious=1, highly_suspicious=2",
                    "definition": {
                        "concept": "History / clinical suspicion (H)",
                        "statement": "A subjective overall impression of how suspicious the presentation is for an acute coronary syndrome, weighing the character, radiation, duration, and reproducibility of the pain together with accompanying symptoms.",
                        "caveats": "This is a deliberate clinical gestalt, not a checklist. 'Highly suspicious' implies mainly classical features (e.g. heavy retrosternal pain, radiation, relief by nitrates, accompanying sweating/nausea); 'slightly suspicious' implies mainly non-specific features. Inter-rater variability is the main weakness of the HEART score.",
                        "source": { "citation": "Six AJ, Backus BE, Kelder JC. Neth Heart J. 2008;16(6):191-196.", "url": "https://doi.org/10.1007/BF03086144" },
                        "status": "draft"
                    }
                },
                "ecg": {
                    "type": "string",
                    "enum": ["normal", "non_specific_repolarisation", "significant_st_deviation"],
                    "description": "ECG: normal=0, non_specific_repolarisation=1 (e.g. LBBB, LVH, digoxin, unchanged repolarisation changes), significant_st_deviation=2 (ST deviation not due to LBBB/LVH/digoxin)"
                },
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (<45 scores 0, 45-64 scores 1, 65+ scores 2)"
                },
                "risk_factors": {
                    "type": "string",
                    "enum": ["none", "one_or_two", "three_or_more_or_atherosclerosis"],
                    "description": "Coronary risk-factor burden: none=0, one_or_two=1, three_or_more_or_atherosclerosis=2",
                    "definition": {
                        "concept": "Risk factors (R)",
                        "statement": "Count of risk factors for coronary artery disease, banded as none, one-to-two, or three-or-more. A documented history of atherosclerotic disease scores the maximum 2 regardless of the count.",
                        "includes": [
                            "Hypertension",
                            "Hypercholesterolaemia",
                            "Diabetes mellitus",
                            "Obesity (BMI > 30)",
                            "Current or recent smoking (within ~1 month)",
                            "Positive family history of premature coronary disease",
                            "History of atherosclerotic disease (prior MI, PCI/CABG, stroke/TIA, or peripheral arterial disease) - auto-scores 2"
                        ],
                        "caveats": "A history of atherosclerotic disease short-circuits the count and scores 2 even if it is the only risk factor present. Count the modifiable factors otherwise.",
                        "source": { "citation": "Six AJ, Backus BE, Kelder JC. Neth Heart J. 2008;16(6):191-196.", "url": "https://doi.org/10.1007/BF03086144" },
                        "status": "draft"
                    }
                },
                "troponin": {
                    "type": "string",
                    "enum": ["normal", "one_to_three_times", "over_three_times"],
                    "description": "Initial troponin vs the assay's normal limit: normal (<=limit)=0, one_to_three_times=1, over_three_times (>3x)=2"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: HeartInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HeartInput {
        HeartInput {
            history: History::SlightlySuspicious,
            ecg: Ecg::Normal,
            age: 40,
            risk_factors: RiskFactors::None,
            troponin: Troponin::Normal,
        }
    }

    #[test]
    fn age_bands() {
        assert_eq!(age_points(44), 0);
        assert_eq!(age_points(45), 1);
        assert_eq!(age_points(64), 1);
        assert_eq!(age_points(65), 2);
    }

    #[test]
    fn all_zero_is_low() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.band, RiskBand::Low);
    }

    #[test]
    fn worked_example_moderate() {
        // 60yo, moderately suspicious history (1), non-specific ECG (1),
        // age 45-64 (1), 1-2 risk factors (1), troponin 1-3x (1) = 5 -> moderate.
        let i = HeartInput {
            history: History::ModeratelySuspicious,
            ecg: Ecg::NonSpecificRepolarisation,
            age: 60,
            risk_factors: RiskFactors::OneOrTwo,
            troponin: Troponin::OneToThreeTimes,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 5);
        assert_eq!(o.band, RiskBand::Moderate);
        assert!(o.interpretation.contains("moderate risk"));
    }

    #[test]
    fn maximum_score_is_ten_and_high() {
        let i = HeartInput {
            history: History::HighlySuspicious,
            ecg: Ecg::SignificantStDeviation,
            age: 70,
            risk_factors: RiskFactors::ThreeOrMoreOrAtherosclerosis,
            troponin: Troponin::OverThreeTimes,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 10);
        assert_eq!(o.band, RiskBand::High);
    }

    #[test]
    fn band_boundary_three_is_low() {
        // history 2 + age 1 (age 50) = 3 -> still low.
        let mut i = base();
        i.history = History::HighlySuspicious;
        i.age = 50;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 3);
        assert_eq!(o.band, RiskBand::Low);
    }

    #[test]
    fn band_boundary_four_is_moderate() {
        // history 2 + age 1 (age 50) + risk 1 = 4 -> moderate.
        let mut i = base();
        i.history = History::HighlySuspicious;
        i.age = 50;
        i.risk_factors = RiskFactors::OneOrTwo;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.band, RiskBand::Moderate);
    }

    #[test]
    fn band_boundary_six_is_moderate() {
        // history 2 + ecg 2 + age 2 (age 70) = 6 -> moderate.
        let mut i = base();
        i.history = History::HighlySuspicious;
        i.ecg = Ecg::SignificantStDeviation;
        i.age = 70;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 6);
        assert_eq!(o.band, RiskBand::Moderate);
    }

    #[test]
    fn band_boundary_seven_is_high() {
        // history 2 + ecg 2 + age 2 (age 70) + risk 1 = 7 -> high.
        let mut i = base();
        i.history = History::HighlySuspicious;
        i.ecg = Ecg::SignificantStDeviation;
        i.age = 70;
        i.risk_factors = RiskFactors::OneOrTwo;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 7);
        assert_eq!(o.band, RiskBand::High);
    }

    #[test]
    fn invalid_enum_is_rejected() {
        let value = json!({
            "history": "very_suspicious",
            "ecg": "normal",
            "age": 50,
            "risk_factors": "none",
            "troponin": "normal"
        });
        assert!(matches!(
            Heart.calculate(&value),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn missing_field_is_rejected() {
        let value = json!({
            "history": "slightly_suspicious",
            "ecg": "normal",
            "age": 50,
            "troponin": "normal"
        });
        assert!(matches!(
            Heart.calculate(&value),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn risk_factor_definition_notes_atherosclerosis_autoscores() {
        let schema = Heart.input_schema();
        let includes = &schema["properties"]["risk_factors"]["definition"]["includes"];
        let last = includes.as_array().unwrap().last().unwrap().as_str().unwrap();
        assert!(last.contains("atherosclerotic disease"));
        assert!(last.contains("auto-scores 2"));
    }

    #[test]
    fn history_definition_marks_subjectivity() {
        let schema = Heart.input_schema();
        let caveats = schema["properties"]["history"]["definition"]["caveats"]
            .as_str()
            .unwrap();
        assert!(caveats.contains("gestalt"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "history": "moderately_suspicious",
            "ecg": "non_specific_repolarisation",
            "age": 60,
            "risk_factors": "one_or_two",
            "troponin": "one_to_three_times"
        });
        let typed = HeartInput {
            history: History::ModeratelySuspicious,
            ecg: Ecg::NonSpecificRepolarisation,
            age: 60,
            risk_factors: RiskFactors::OneOrTwo,
            troponin: Troponin::OneToThreeTimes,
        };
        let dynamic = Heart.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(5));
    }
}
