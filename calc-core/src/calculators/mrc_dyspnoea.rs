//! MRC Dyspnoea Scale - a five-point grade of breathlessness.
//!
//! The Medical Research Council (MRC) breathlessness scale (Fletcher, 1959)
//! grades the disability that breathlessness causes during day-to-day
//! activities, on a single ordinal scale from grade 1 (breathless only on
//! strenuous exertion) to grade 5 (too breathless to leave the house). It is
//! the version in routine UK use, recommended by NICE (NG115) and the British
//! Thoracic Society for COPD assessment.
//!
//! This is the classic MRC scale (grades 1-5). A modified form (mMRC) renumbers
//! the same descriptors 0-4; mMRC grade = MRC grade - 1. The two are otherwise
//! identical in wording, so a grade quoted without a scale is ambiguous - this
//! calculator takes the classic 1-5 grade.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "mrc_dyspnoea";

/// Distribution licence: the MRC Dyspnoea Scale is published by the Medical
/// Research Council and made freely available for use, asking only that the
/// source is acknowledged.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free to use - published by the Medical Research Council; reproduction permitted with acknowledgement of the source",
    source_url: "https://www.ukri.org/councils/mrc/facilities-and-resources/find-an-mrc-facility-or-resource/mrc-dyspnoea-scale/",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Fletcher CM, Elmes PC, Fairbairn AS, Wood CH. The significance of respiratory symptoms and \
the diagnosis of chronic bronchitis in a working population. Br Med J. 1959;2(5147):257-266. \
doi:10.1136/bmj.2.5147.257";

/// Lowest valid MRC grade.
pub const MIN_GRADE: u8 = 1;

/// Highest valid MRC grade.
pub const MAX_GRADE: u8 = 5;

/// Input: a single MRC breathlessness grade (1-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MrcDyspnoeaInput {
    /// The MRC Dyspnoea Scale grade, an integer from 1 (least) to 5 (most).
    pub grade: u8,
}

/// The descriptor for a given grade.
fn descriptor(grade: u8) -> &'static str {
    match grade {
        1 => "Not troubled by breathlessness except on strenuous exercise",
        2 => "Short of breath when hurrying on the level or walking up a slight hill",
        3 => "Walks slower than contemporaries on the level because of breathlessness, or has to stop for breath when walking at own pace",
        4 => "Stops for breath after walking about 100 metres or after a few minutes on the level",
        5 => "Too breathless to leave the house, or breathless when dressing or undressing",
        _ => unreachable!("grade is validated to 1-5 before this is called"),
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct MrcDyspnoeaOutcome {
    /// The MRC grade (echoed back, 1-5).
    pub grade: u8,
    /// The grade's breathlessness descriptor.
    pub descriptor: &'static str,
    /// Clinical interpretation.
    pub interpretation: String,
}

/// Pure scoring: validate the grade and attach its descriptor.
pub fn compute(input: &MrcDyspnoeaInput) -> Result<MrcDyspnoeaOutcome, CalcError> {
    if input.grade < MIN_GRADE || input.grade > MAX_GRADE {
        return Err(CalcError::InvalidInput(format!(
            "grade must be an integer from {MIN_GRADE} to {MAX_GRADE}, got {}",
            input.grade
        )));
    }

    let descriptor = descriptor(input.grade);
    let interpretation = format!(
        "MRC Dyspnoea grade {} of 5: {}. The MRC scale grades the disability breathlessness causes, \
not lung function; it does not by itself diagnose or stage a disease. This is the classic MRC scale \
(1-5); the modified mMRC scale numbers the same descriptors 0-4 (mMRC = MRC grade - 1), so confirm \
which scale is meant when a grade is recorded.",
        input.grade, descriptor
    );

    Ok(MrcDyspnoeaOutcome {
        grade: input.grade,
        descriptor,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &MrcDyspnoeaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("grade".into(), json!(o.grade));
    working.insert("descriptor".into(), json!(o.descriptor));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.grade),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct MrcDyspnoea;

impl Calculator for MrcDyspnoea {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "MRC Dyspnoea Scale"
    }

    fn description(&self) -> &'static str {
        "Grades breathlessness-related disability on the classic MRC 1-5 scale (Fletcher 1959; NICE/BTS UK usage)."
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
            "title": "MrcDyspnoeaInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["grade"],
            "properties": {
                "grade": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 5,
                    "description": "MRC Dyspnoea grade (classic 1-5 scale). 1 = not troubled by breathlessness except on strenuous exercise; 2 = short of breath when hurrying on the level or walking up a slight hill; 3 = walks slower than contemporaries on the level because of breathlessness, or stops for breath when walking at own pace; 4 = stops for breath after walking about 100 metres or after a few minutes on the level; 5 = too breathless to leave the house, or breathless when dressing or undressing.",
                    "definition": {
                        "concept": "MRC Dyspnoea Scale grade",
                        "statement": "A single ordinal grade (1-5) describing how much breathlessness limits everyday activity, from breathless only on strenuous exertion (1) to too breathless to leave the house (5).",
                        "caveats": "This is the classic MRC scale (1-5). The modified mMRC scale numbers the same descriptors 0-4, where mMRC grade = MRC grade - 1; the two are easily confused, so confirm which scale a recorded grade refers to.",
                        "excludes": [
                            "Do NOT pass an mMRC (0-4) grade here: 0 is out of range, and grades 1-4 would each be read one step too severe relative to the classic scale"
                        ],
                        "source": {
                            "citation": "Fletcher CM et al. Br Med J. 1959;2(5147):257-266.",
                            "url": "https://www.ukri.org/councils/mrc/facilities-and-resources/find-an-mrc-facility-or-resource/mrc-dyspnoea-scale/"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: MrcDyspnoeaInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(grade: u8) -> MrcDyspnoeaInput {
        MrcDyspnoeaInput { grade }
    }

    #[test]
    fn grade_1_interpretation() {
        let o = compute(&input(1)).unwrap();
        assert_eq!(o.grade, 1);
        assert_eq!(
            o.descriptor,
            "Not troubled by breathlessness except on strenuous exercise"
        );
    }

    #[test]
    fn grade_2_interpretation() {
        let o = compute(&input(2)).unwrap();
        assert_eq!(o.grade, 2);
        assert_eq!(
            o.descriptor,
            "Short of breath when hurrying on the level or walking up a slight hill"
        );
    }

    #[test]
    fn grade_3_interpretation() {
        let o = compute(&input(3)).unwrap();
        assert_eq!(o.grade, 3);
        assert!(o.descriptor.contains("Walks slower than contemporaries"));
    }

    #[test]
    fn grade_4_interpretation() {
        let o = compute(&input(4)).unwrap();
        assert_eq!(o.grade, 4);
        assert!(o.descriptor.contains("100 metres"));
    }

    #[test]
    fn grade_5_interpretation() {
        let o = compute(&input(5)).unwrap();
        assert_eq!(o.grade, 5);
        assert!(o.descriptor.contains("Too breathless to leave the house"));
    }

    #[test]
    fn rejects_out_of_range() {
        // 0 is the mMRC floor, not valid on the classic scale.
        assert!(compute(&input(0)).is_err());
        assert!(compute(&input(6)).is_err());
        assert!(compute(&input(100)).is_err());
    }

    #[test]
    fn result_echoes_grade() {
        let r = build_response(&input(3)).unwrap();
        assert_eq!(r.result, json!(3));
        assert_eq!(r.calculator, NAME);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "grade": 4 });
        let dynamic = MrcDyspnoea.calculate(&value).unwrap();
        let typed = build_response(&input(4)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn dynamic_rejects_out_of_range() {
        assert!(MrcDyspnoea.calculate(&json!({ "grade": 0 })).is_err());
        assert!(MrcDyspnoea.calculate(&json!({ "grade": 9 })).is_err());
    }

    #[test]
    fn schema_constrains_grade_and_flags_mmrc() {
        let schema = MrcDyspnoea.input_schema();
        let grade = &schema["properties"]["grade"];
        assert_eq!(grade["minimum"], json!(1));
        assert_eq!(grade["maximum"], json!(5));
        let def = &grade["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("mMRC"));
    }
}
