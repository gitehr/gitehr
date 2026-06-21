//! GAD-7 - Generalised Anxiety Disorder 7-item scale.
//!
//! Seven items, each rated 0-3 over the **last two weeks**, summed to a 0-21
//! severity score with standard bands (NICE CG113). A total of 10 or more is
//! the validated cut-point for likely generalised anxiety disorder and warrants
//! further assessment.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "gad7";

/// Distribution licence: Pfizer released the GAD-7 into the public domain in
/// 2010; no permission is required to reproduce, translate, display, or
/// distribute.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public domain - released by Pfizer (2010); no permission required to reproduce, translate, display, or distribute",
    source_url: "https://www.pfizer.com/news/press-release/press-release-detail/pfizer_to_offer_free_public_access_to_mental_health_assessment_tools_to_improve_diagnosis_and_patient_care",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Spitzer RL, Kroenke K, Williams JBW, Löwe B. A brief measure for assessing generalized \
anxiety disorder: the GAD-7. Arch Intern Med. 2006;166(10):1092-1097. doi:10.1001/archinte.166.10.1092";

/// Number of items.
pub const ITEM_COUNT: usize = 7;

/// Validated cut-point for likely GAD.
pub const CASE_THRESHOLD: u16 = 10;

/// The seven GAD-7 responses, each 0 (Not at all) - 3 (Nearly every day), in
/// question order Q1-Q7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gad7Input {
    pub responses: Vec<u8>,
}

/// Anxiety severity band implied by the total score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Minimal,
    Mild,
    Moderate,
    Severe,
}

impl Severity {
    /// Standard GAD-7 bands (Spitzer 2006).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=4 => Severity::Minimal,
            5..=9 => Severity::Mild,
            10..=14 => Severity::Moderate,
            _ => Severity::Severe,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Severity::Minimal => "minimal",
            Severity::Mild => "mild",
            Severity::Moderate => "moderate",
            Severity::Severe => "severe",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gad7Outcome {
    /// Total severity score (0-21).
    pub total: u16,
    pub severity: Severity,
    /// True if total >= 10, the validated cut-point for likely GAD.
    pub above_case_threshold: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &Gad7Input) -> Result<Gad7Outcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }
    for (i, &v) in input.responses.iter().enumerate() {
        if v > 3 {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {v} is out of range 0-3",
                i + 1
            )));
        }
    }

    let total: u16 = input.responses.iter().map(|&v| v as u16).sum();
    let severity = Severity::from_total(total);
    let above_case_threshold = total >= CASE_THRESHOLD;

    let mut interpretation =
        format!("Total score {total}/21 indicates {} anxiety symptoms.", severity.label());
    if above_case_threshold {
        interpretation.push_str(
            " At or above the cut-point of 10 for likely generalised anxiety disorder; \
further assessment is warranted.",
        );
    } else {
        interpretation.push_str(" Below the cut-point of 10 for likely generalised anxiety disorder.");
    }
    interpretation.push_str(" GAD-7 supports severity grading; it is not a diagnosis.");

    Ok(Gad7Outcome {
        total,
        severity,
        above_case_threshold,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Gad7Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("severity".into(), json!(o.severity.label()));
    working.insert("above_case_threshold".into(), json!(o.above_case_threshold));
    working.insert("answers".into(), json!(input.responses));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Gad7;

impl Calculator for Gad7 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "GAD-7 Anxiety Severity"
    }

    fn description(&self) -> &'static str {
        "Seven-item generalised anxiety severity score (0-21); a total of 10+ flags likely GAD."
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
            "title": "Gad7Input",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Seven responses (Q1-Q7), each 0=Not at all, 1=Several days, 2=More than half the days, 3=Nearly every day",
                    "items": { "type": "integer", "minimum": 0, "maximum": 3 },
                    "minItems": 7,
                    "maxItems": 7,
                    "definition": {
                        "concept": "GAD-7 item responses",
                        "statement": "Each item rates how often the patient has been bothered by the problem over the LAST 2 WEEKS.",
                        "includes": [
                            "Q1 Feeling nervous, anxious, or on edge",
                            "Q2 Not being able to stop or control worrying",
                            "Q3 Worrying too much about different things",
                            "Q4 Trouble relaxing",
                            "Q5 Being so restless that it is hard to sit still",
                            "Q6 Becoming easily annoyed or irritable",
                            "Q7 Feeling afraid as if something awful might happen"
                        ],
                        "caveats": "The cut-point of 10 maximises sensitivity and specificity for GAD; lower thresholds screen for other anxiety disorders.",
                        "source": {
                            "citation": "Spitzer RL, Kroenke K, Williams JBW, Löwe B. Arch Intern Med. 2006;166(10):1092-1097.",
                            "url": "https://doi.org/10.1001/archinte.166.10.1092"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Gad7Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 7]) -> Gad7Input {
        Gad7Input { responses: v.to_vec() }
    }

    #[test]
    fn all_zero_is_minimal() {
        let o = compute(&responses([0; 7])).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.severity, Severity::Minimal);
        assert!(!o.above_case_threshold);
    }

    #[test]
    fn band_boundaries_match_spitzer() {
        assert_eq!(Severity::from_total(4), Severity::Minimal);
        assert_eq!(Severity::from_total(5), Severity::Mild);
        assert_eq!(Severity::from_total(9), Severity::Mild);
        assert_eq!(Severity::from_total(10), Severity::Moderate);
        assert_eq!(Severity::from_total(14), Severity::Moderate);
        assert_eq!(Severity::from_total(15), Severity::Severe);
        assert_eq!(Severity::from_total(21), Severity::Severe);
    }

    #[test]
    fn cut_point_is_ten() {
        let nine = compute(&responses([3, 3, 3, 0, 0, 0, 0])).unwrap();
        assert_eq!(nine.total, 9);
        assert!(!nine.above_case_threshold);

        let ten = compute(&responses([3, 3, 3, 1, 0, 0, 0])).unwrap();
        assert_eq!(ten.total, 10);
        assert!(ten.above_case_threshold);
        assert!(ten.interpretation.contains("cut-point of 10"));
    }

    #[test]
    fn maximum_score_is_severe() {
        let o = compute(&responses([3; 7])).unwrap();
        assert_eq!(o.total, 21);
        assert_eq!(o.severity, Severity::Severe);
    }

    #[test]
    fn wrong_length_and_range_are_rejected() {
        assert!(compute(&Gad7Input { responses: vec![0; 6] }).is_err());
        assert!(compute(&Gad7Input { responses: vec![0; 8] }).is_err());
        assert!(compute(&Gad7Input { responses: vec![4, 0, 0, 0, 0, 0, 0] }).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [2, 2, 2, 1, 1, 1, 1];
        let dynamic = Gad7.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(10));
        assert_eq!(dynamic.working["above_case_threshold"], json!(true));
    }
}
