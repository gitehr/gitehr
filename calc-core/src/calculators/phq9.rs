//! PHQ-9 - Patient Health Questionnaire depression severity measure.
//!
//! Nine items, each rated 0-3 over the **last two weeks**, summed to a 0-27
//! severity score with standard bands (NICE NG222 monitoring use). Item 9
//! (thoughts of self-harm) is a clinical-safety item: any non-zero response
//! flags a need for risk assessment, independent of the total score.
//!
//! The optional tenth functional-impairment question ("how difficult have these
//! problems made it ...") is **not** part of the 0-27 score and is not modelled
//! here.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "phq9";

/// Distribution licence: Pfizer released the PHQ family into the public domain
/// in 2010; no permission is required to reproduce, translate, display, or
/// distribute.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public domain - released by Pfizer (2010); no permission required to reproduce, translate, display, or distribute",
    source_url: "https://www.pfizer.com/news/press-release/press-release-detail/pfizer_to_offer_free_public_access_to_mental_health_assessment_tools_to_improve_diagnosis_and_patient_care",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Kroenke K, Spitzer RL, Williams JBW. The PHQ-9: validity of a brief depression \
severity measure. J Gen Intern Med. 2001;16(9):606-613. doi:10.1046/j.1525-1497.2001.016009606.x";

/// Number of scored items.
pub const ITEM_COUNT: usize = 9;

/// Index of the self-harm safety item (Q9, zero-based).
pub const SELF_HARM_ITEM: usize = 8;

/// The nine PHQ-9 responses, each 0 (Not at all) - 3 (Nearly every day), in
/// question order Q1-Q9.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phq9Input {
    pub responses: Vec<u8>,
}

/// Depression severity band implied by the total score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    None,
    Mild,
    Moderate,
    ModeratelySevere,
    Severe,
}

impl Severity {
    /// Standard PHQ-9 bands (Kroenke 2001).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=4 => Severity::None,
            5..=9 => Severity::Mild,
            10..=14 => Severity::Moderate,
            15..=19 => Severity::ModeratelySevere,
            _ => Severity::Severe,
        }
    }

    /// Stable slug for the `working` breakdown.
    pub fn slug(self) -> &'static str {
        match self {
            Severity::None => "none-minimal",
            Severity::Mild => "mild",
            Severity::Moderate => "moderate",
            Severity::ModeratelySevere => "moderately-severe",
            Severity::Severe => "severe",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Severity::None => "none-minimal",
            Severity::Mild => "mild",
            Severity::Moderate => "moderate",
            Severity::ModeratelySevere => "moderately severe",
            Severity::Severe => "severe",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phq9Outcome {
    /// Total severity score (0-27).
    pub total: u16,
    pub severity: Severity,
    /// True if Q9 (self-harm) scored 1 or more - flags a risk assessment.
    pub self_harm_flag: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &Phq9Input) -> Result<Phq9Outcome, CalcError> {
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
    let self_harm_flag = input.responses[SELF_HARM_ITEM] >= 1;

    let mut interpretation = format!(
        "Total score {total}/27 indicates {} depressive symptoms.",
        severity.label()
    );
    if self_harm_flag {
        interpretation.push_str(
            " Item 9 (thoughts of self-harm) is positive: a suicide-risk assessment is \
indicated regardless of the total score.",
        );
    }
    interpretation.push_str(" PHQ-9 supports severity grading and monitoring; it is not a diagnosis.");

    Ok(Phq9Outcome {
        total,
        severity,
        self_harm_flag,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Phq9Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("severity".into(), json!(o.severity.slug()));
    working.insert("self_harm_item_flag".into(), json!(o.self_harm_flag));
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
pub struct Phq9;

impl Calculator for Phq9 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "PHQ-9 Depression Severity"
    }

    fn description(&self) -> &'static str {
        "Nine-item depression severity score (0-27) with standard bands; item 9 flags self-harm risk."
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
            "title": "Phq9Input",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Nine responses (Q1-Q9), each 0=Not at all, 1=Several days, 2=More than half the days, 3=Nearly every day",
                    "items": { "type": "integer", "minimum": 0, "maximum": 3 },
                    "minItems": 9,
                    "maxItems": 9,
                    "definition": {
                        "concept": "PHQ-9 item responses",
                        "statement": "Each item rates how often the patient has been bothered by the problem over the LAST 2 WEEKS.",
                        "includes": [
                            "Q1 Little interest or pleasure in doing things",
                            "Q2 Feeling down, depressed, or hopeless",
                            "Q3 Trouble falling or staying asleep, or sleeping too much",
                            "Q4 Feeling tired or having little energy",
                            "Q5 Poor appetite or overeating",
                            "Q6 Feeling bad about yourself, or that you are a failure",
                            "Q7 Trouble concentrating",
                            "Q8 Moving or speaking slowly, or being restless",
                            "Q9 Thoughts that you would be better off dead, or of hurting yourself"
                        ],
                        "excludes": [
                            "The tenth functional-difficulty question is NOT scored and is not part of the 0-27 total"
                        ],
                        "caveats": "Q9 is a safety item: any non-zero score warrants suicide-risk assessment irrespective of the total.",
                        "source": {
                            "citation": "Kroenke K, Spitzer RL, Williams JBW. J Gen Intern Med. 2001;16(9):606-613.",
                            "url": "https://doi.org/10.1046/j.1525-1497.2001.016009606.x"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Phq9Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 9]) -> Phq9Input {
        Phq9Input { responses: v.to_vec() }
    }

    #[test]
    fn all_zero_is_none_minimal() {
        let o = compute(&responses([0; 9])).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.severity, Severity::None);
        assert!(!o.self_harm_flag);
    }

    #[test]
    fn band_boundaries_match_kroenke() {
        assert_eq!(Severity::from_total(4), Severity::None);
        assert_eq!(Severity::from_total(5), Severity::Mild);
        assert_eq!(Severity::from_total(9), Severity::Mild);
        assert_eq!(Severity::from_total(10), Severity::Moderate);
        assert_eq!(Severity::from_total(14), Severity::Moderate);
        assert_eq!(Severity::from_total(15), Severity::ModeratelySevere);
        assert_eq!(Severity::from_total(19), Severity::ModeratelySevere);
        assert_eq!(Severity::from_total(20), Severity::Severe);
        assert_eq!(Severity::from_total(27), Severity::Severe);
    }

    #[test]
    fn maximum_score_is_severe() {
        let o = compute(&responses([3; 9])).unwrap();
        assert_eq!(o.total, 27);
        assert_eq!(o.severity, Severity::Severe);
        assert!(o.self_harm_flag);
    }

    #[test]
    fn self_harm_flag_is_independent_of_total() {
        // Low total (3) but Q9 positive must still flag.
        let o = compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 3])).unwrap();
        assert_eq!(o.total, 3);
        assert_eq!(o.severity, Severity::None);
        assert!(o.self_harm_flag);
        assert!(o.interpretation.contains("suicide-risk assessment"));
    }

    #[test]
    fn wrong_length_and_range_are_rejected() {
        assert!(compute(&Phq9Input { responses: vec![0; 8] }).is_err());
        assert!(compute(&Phq9Input { responses: vec![0; 10] }).is_err());
        assert!(compute(&Phq9Input { responses: vec![4, 0, 0, 0, 0, 0, 0, 0, 0] }).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [2, 2, 1, 1, 1, 0, 1, 0, 0];
        let dynamic = Phq9.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(8));
        assert_eq!(dynamic.working["severity"], json!("mild"));
    }

    #[test]
    fn schema_carries_input_definition() {
        let schema = Phq9.input_schema();
        let def = &schema["properties"]["responses"]["definition"];
        assert!(def["excludes"][0].as_str().unwrap().contains("NOT scored"));
        assert_eq!(def["status"], json!("draft"));
    }
}
