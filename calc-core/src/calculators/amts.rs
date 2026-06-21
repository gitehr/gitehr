//! AMTS - Abbreviated Mental Test Score (Hodkinson 1972).
//!
//! A 10-item bedside screen for cognitive impairment in older patients, each
//! item scoring 1 point for a correct answer for a total of 0-10. A score below
//! 8 (i.e. 7 or less) suggests cognitive impairment - which may be dementia,
//! delirium, or another cause - at the time of testing and warrants further,
//! more formal assessment. It is a screen, not a diagnosis, and does not
//! distinguish dementia from delirium.
//!
//! Each item is a clinician-asserted predicate: TRUE means the patient answered
//! that item correctly. The address item (Q3) is given at the start and recalled
//! at the end, so its TRUE condition is correct recall, not correct repetition.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "amts";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature. The 10-item test carries no proprietary
/// licence and is reproduced freely in clinical practice and guidance.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1093/ageing/1.4.233",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Hodkinson HM. Evaluation of a mental test score for assessment of mental impairment in the \
elderly. Age Ageing. 1972;1(4):233-238. doi:10.1093/ageing/1.4.233";

/// Number of items.
pub const ITEM_COUNT: u8 = 10;

/// Highest total score that suggests cognitive impairment (a score of 7 or
/// less, i.e. below 8, is the commonly applied cut-off).
pub const IMPAIRMENT_CUTOFF: u8 = 7;

/// The ten AMTS items. Each field is TRUE when the patient answered that item
/// correctly, scoring 1 point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmtsInput {
    /// Q1: states their age correctly.
    pub age: bool,
    /// Q2: states the time to the nearest hour correctly.
    pub time: bool,
    /// Q3: recalls, at the end of the test, an address given at the start
    /// (e.g. "42 West Street").
    pub address_recall: bool,
    /// Q4: states the current year correctly.
    pub year: bool,
    /// Q5: names the present place (e.g. the hospital) correctly.
    pub place: bool,
    /// Q6: recognises two persons (e.g. doctor and nurse).
    pub recognise_two_persons: bool,
    /// Q7: states their date of birth correctly.
    pub date_of_birth: bool,
    /// Q8: names the year World War I started (1914), or another agreed
    /// well-known date.
    pub year_ww1: bool,
    /// Q9: names the present monarch / head of state correctly.
    pub monarch: bool,
    /// Q10: counts backwards from 20 to 1 without error.
    pub count_backwards: bool,
}

impl AmtsInput {
    /// The ten items as `(working-key, scored)` pairs, in question order.
    fn items(&self) -> [(&'static str, bool); 10] {
        [
            ("age", self.age),
            ("time", self.time),
            ("address_recall", self.address_recall),
            ("year", self.year),
            ("place", self.place),
            ("recognise_two_persons", self.recognise_two_persons),
            ("date_of_birth", self.date_of_birth),
            ("year_ww1", self.year_ww1),
            ("monarch", self.monarch),
            ("count_backwards", self.count_backwards),
        ]
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmtsOutcome {
    /// Total score (0-10).
    pub score: u8,
    /// True if the score is at or below the impairment cut-off (7 or less).
    pub suggests_impairment: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &AmtsInput) -> Result<AmtsOutcome, CalcError> {
    let score: u8 = input.items().iter().map(|&(_, ok)| u8::from(ok)).sum();
    let suggests_impairment = score <= IMPAIRMENT_CUTOFF;

    let interpretation = if suggests_impairment {
        format!(
            "Score {score}/{ITEM_COUNT}: below the cut-off of 8, suggesting cognitive impairment \
at the time of testing. This may reflect dementia, delirium, or another cause; the AMTS does not \
distinguish between them. Further, more formal assessment is warranted - the AMTS is a screen, not \
a diagnosis."
        )
    } else {
        format!(
            "Score {score}/{ITEM_COUNT}: at or above the cut-off of 8, not suggestive of cognitive \
impairment on this screen. The AMTS lacks sensitivity for mild impairment, so clinical judgement \
still applies."
        )
    };

    Ok(AmtsOutcome {
        score,
        suggests_impairment,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &AmtsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("suggests_impairment".into(), json!(o.suggests_impairment));
    for (key, ok) in input.items() {
        working.insert(key.into(), json!(u8::from(ok)));
    }

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Amts;

impl Calculator for Amts {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Abbreviated Mental Test Score (AMTS)"
    }

    fn description(&self) -> &'static str {
        "Ten-item bedside cognitive screen (0-10); a score below 8 suggests cognitive impairment."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        let item = |concept: &str, statement: &str| {
            json!({
                "type": "boolean",
                "description": statement,
                "definition": {
                    "concept": concept,
                    "statement": statement,
                    "source": {
                        "citation": "Hodkinson HM. Age Ageing. 1972;1(4):233-238.",
                        "url": "https://doi.org/10.1093/ageing/1.4.233"
                    },
                    "status": "draft"
                }
            })
        };

        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "AmtsInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "time", "address_recall", "year", "place",
                "recognise_two_persons", "date_of_birth", "year_ww1",
                "monarch", "count_backwards"
            ],
            "properties": {
                "age": item("Q1 Age", "Patient states their own age correctly."),
                "time": item("Q2 Time", "Patient states the time to the nearest hour correctly (no clock or watch)."),
                "address_recall": item(
                    "Q3 Address recall",
                    "Patient correctly recalls, at the END of the test, an address given at the START (e.g. \"42 West Street\"). Score correct recall, not correct repetition at the time of giving."
                ),
                "year": item("Q4 Year", "Patient states the current year correctly."),
                "place": item("Q5 Place", "Patient names the present place (e.g. the name of the hospital or building) correctly."),
                "recognise_two_persons": item(
                    "Q6 Recognition of two persons",
                    "Patient recognises two persons present (e.g. doctor and nurse, or relative and carer) - their role, not necessarily their name."
                ),
                "date_of_birth": item("Q7 Date of birth", "Patient states their date of birth correctly."),
                "year_ww1": item("Q8 Year of WW1", "Patient names the year World War I started (1914), or another agreed well-known date."),
                "monarch": item("Q9 Present monarch", "Patient names the present monarch or head of state correctly."),
                "count_backwards": item("Q10 Count backwards", "Patient counts backwards from 20 down to 1 without error.")
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AmtsInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(correct: bool) -> AmtsInput {
        AmtsInput {
            age: correct,
            time: correct,
            address_recall: correct,
            year: correct,
            place: correct,
            recognise_two_persons: correct,
            date_of_birth: correct,
            year_ww1: correct,
            monarch: correct,
            count_backwards: correct,
        }
    }

    #[test]
    fn full_marks_is_ten_and_normal() {
        let o = compute(&all(true)).unwrap();
        assert_eq!(o.score, 10);
        assert!(!o.suggests_impairment);
        assert!(o.interpretation.contains("at or above the cut-off"));
    }

    #[test]
    fn zero_is_impairment() {
        let o = compute(&all(false)).unwrap();
        assert_eq!(o.score, 0);
        assert!(o.suggests_impairment);
        assert!(o.interpretation.contains("below the cut-off"));
    }

    #[test]
    fn worked_example_six_correct() {
        // Correct: age, time, year, place, date_of_birth, monarch = 6/10.
        let mut i = all(false);
        i.age = true;
        i.time = true;
        i.year = true;
        i.place = true;
        i.date_of_birth = true;
        i.monarch = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 6);
        assert!(o.suggests_impairment);
    }

    #[test]
    fn cutoff_is_below_eight() {
        // 7 correct -> impairment; 8 correct -> not.
        let mut seven = all(true);
        seven.monarch = false;
        seven.count_backwards = false;
        seven.year_ww1 = false;
        let o7 = compute(&seven).unwrap();
        assert_eq!(o7.score, 7);
        assert!(o7.suggests_impairment);

        let mut eight = all(true);
        eight.monarch = false;
        eight.count_backwards = false;
        let o8 = compute(&eight).unwrap();
        assert_eq!(o8.score, 8);
        assert!(!o8.suggests_impairment);
    }

    #[test]
    fn missing_and_unknown_fields_are_rejected() {
        // Missing a required field.
        assert!(Amts
            .calculate(&json!({
                "age": true, "time": true, "address_recall": true, "year": true,
                "place": true, "recognise_two_persons": true, "date_of_birth": true,
                "year_ww1": true, "monarch": true
            }))
            .is_err());
        // Unknown field.
        assert!(Amts
            .calculate(&json!({
                "age": true, "time": true, "address_recall": true, "year": true,
                "place": true, "recognise_two_persons": true, "date_of_birth": true,
                "year_ww1": true, "monarch": true, "count_backwards": true,
                "extra": true
            }))
            .is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": true, "time": true, "address_recall": false, "year": true,
            "place": true, "recognise_two_persons": true, "date_of_birth": true,
            "year_ww1": false, "monarch": true, "count_backwards": true
        });
        let mut typed = all(true);
        typed.address_recall = false;
        typed.year_ww1 = false;
        let dynamic = Amts.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(8));
        assert_eq!(dynamic.working["suggests_impairment"], json!(false));
        assert_eq!(dynamic.working["address_recall"], json!(0));
    }
}
