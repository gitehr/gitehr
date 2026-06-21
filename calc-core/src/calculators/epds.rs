//! EPDS - Edinburgh Postnatal Depression Scale.
//!
//! Ten items, each rated 0-3 over the **last seven days**, summed to a 0-30
//! score. A score of **10 or more** suggests possible depression and **13 or
//! more** suggests probable depression of varying severity (Cox et al. 1987).
//! Item 10 (thoughts of self-harm) is a clinical-safety item: any non-zero
//! response flags a need for risk assessment, independent of the total score.
//!
//! ## Responses must be pre-scored 0-3 per the EPDS key
//!
//! On the printed EPDS the four answer options for each item run top-to-bottom,
//! but the **scoring direction alternates**: items 1, 2 and 4 are scored 0-1-2-3
//! down the page, whereas items 3, 5, 6, 7, 8, 9 and 10 are **reverse scored**
//! 3-2-1-0. This module takes the *already-scored* 0-3 value for each item (as
//! [`phq9`](super::phq9) does), **not** the literal answer position. The caller
//! is responsible for applying the EPDS key - mapping each ticked answer to its
//! 0-3 value, reversing items 3 and 5-10 - before passing `responses` here. The
//! reverse-scored items are listed in [`REVERSE_SCORED_ITEMS`] and surfaced in
//! the input schema so this contract is explicit.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "epds";

/// Distribution licence: the EPDS may be reproduced free of charge for clinical
/// or research use provided it is copied in full and the source is cited; no
/// further permission is required (Cox, Holden & Sagovsky 1987).
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free to reproduce in full for clinical or research use with the source citation; no further permission required (Cox, Holden & Sagovsky 1987)",
    source_url: "https://doi.org/10.1192/bjp.150.6.782",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Cox JL, Holden JM, Sagovsky R. Detection of postnatal depression: development of the \
10-item Edinburgh Postnatal Depression Scale. Br J Psychiatry. 1987;150:782-786. \
doi:10.1192/bjp.150.6.782";

/// Number of scored items.
pub const ITEM_COUNT: usize = 10;

/// Index of the self-harm safety item (Q10, zero-based).
pub const SELF_HARM_ITEM: usize = 9;

/// Threshold (inclusive) at or above which depression is *possible*.
pub const POSSIBLE_THRESHOLD: u16 = 10;

/// Threshold (inclusive) at or above which depression is *probable*.
pub const PROBABLE_THRESHOLD: u16 = 13;

/// One-based item numbers that are reverse-scored on the printed form (3-2-1-0).
///
/// Informational: `responses` must already be oriented to 0-3 per the EPDS key,
/// so this slice documents which items the caller had to reverse, it does not
/// change the arithmetic here.
pub const REVERSE_SCORED_ITEMS: [u8; 7] = [3, 5, 6, 7, 8, 9, 10];

/// The ten EPDS responses, each already scored 0-3 per the EPDS key, in
/// question order Q1-Q10. See the module docs: items 3 and 5-10 are
/// reverse-scored on the form and must be oriented before being passed here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpdsInput {
    pub responses: Vec<u8>,
}

/// Likelihood band implied by the total score (Cox et al. 1987).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Likelihood {
    /// Below the screening threshold (0-9).
    Unlikely,
    /// Possible depression (10-12).
    Possible,
    /// Probable depression (13-30).
    Probable,
}

impl Likelihood {
    /// Standard EPDS bands: >=10 possible, >=13 probable.
    pub fn from_total(total: u16) -> Self {
        if total >= PROBABLE_THRESHOLD {
            Likelihood::Probable
        } else if total >= POSSIBLE_THRESHOLD {
            Likelihood::Possible
        } else {
            Likelihood::Unlikely
        }
    }

    /// Stable slug for the `working` breakdown.
    pub fn slug(self) -> &'static str {
        match self {
            Likelihood::Unlikely => "unlikely",
            Likelihood::Possible => "possible",
            Likelihood::Probable => "probable",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Likelihood::Unlikely => "depression unlikely",
            Likelihood::Possible => "possible depression",
            Likelihood::Probable => "probable depression",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpdsOutcome {
    /// Total score (0-30).
    pub total: u16,
    pub likelihood: Likelihood,
    /// True if Q10 (self-harm) scored 1 or more - flags a risk assessment.
    pub self_harm_flag: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &EpdsInput) -> Result<EpdsOutcome, CalcError> {
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
    let likelihood = Likelihood::from_total(total);
    let self_harm_flag = input.responses[SELF_HARM_ITEM] >= 1;

    let mut interpretation = format!(
        "Total score {total}/30 indicates {} (threshold >=10 possible, >=13 probable).",
        likelihood.label()
    );
    if self_harm_flag {
        interpretation.push_str(
            " Item 10 (thoughts of self-harm) is positive: a suicide-risk assessment is \
indicated regardless of the total score.",
        );
    }
    interpretation
        .push_str(" The EPDS is a screening aid for clinical judgement; it is not a diagnosis.");

    Ok(EpdsOutcome {
        total,
        likelihood,
        self_harm_flag,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &EpdsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("likelihood".into(), json!(o.likelihood.slug()));
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
pub struct Epds;

impl Calculator for Epds {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Edinburgh Postnatal Depression Scale (EPDS)"
    }

    fn description(&self) -> &'static str {
        "Ten-item perinatal depression screen (0-30); >=10 possible, >=13 probable; item 10 flags self-harm risk."
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
            "title": "EpdsInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Ten responses (Q1-Q10), each ALREADY SCORED 0-3 per the EPDS key. On the form items 1,2,4 score 0-1-2-3 top-to-bottom while items 3,5,6,7,8,9,10 are reverse scored 3-2-1-0; orient them before submitting.",
                    "items": { "type": "integer", "minimum": 0, "maximum": 3 },
                    "minItems": 10,
                    "maxItems": 10,
                    "definition": {
                        "concept": "EPDS item responses",
                        "statement": "Each item rates how the mother has felt over the LAST 7 DAYS, scored 0-3 per the EPDS key.",
                        "includes": [
                            "Q1 I have been able to laugh and see the funny side of things",
                            "Q2 I have looked forward with enjoyment to things",
                            "Q3 I have blamed myself unnecessarily when things went wrong (reverse scored)",
                            "Q4 I have been anxious or worried for no good reason",
                            "Q5 I have felt scared or panicky for no very good reason (reverse scored)",
                            "Q6 Things have been getting on top of me (reverse scored)",
                            "Q7 I have been so unhappy that I have had difficulty sleeping (reverse scored)",
                            "Q8 I have felt sad or miserable (reverse scored)",
                            "Q9 I have been so unhappy that I have been crying (reverse scored)",
                            "Q10 The thought of harming myself has occurred to me (reverse scored)"
                        ],
                        "excludes": [
                            "Responses must be the 0-3 scored value, NOT the literal answer position; reverse-scored items 3,5,6,7,8,9,10 must be oriented by the caller"
                        ],
                        "caveats": "Q10 is a safety item: any non-zero score warrants suicide-risk assessment irrespective of the total.",
                        "source": {
                            "citation": "Cox JL, Holden JM, Sagovsky R. Br J Psychiatry. 1987;150:782-786.",
                            "url": "https://doi.org/10.1192/bjp.150.6.782"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: EpdsInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 10]) -> EpdsInput {
        EpdsInput { responses: v.to_vec() }
    }

    #[test]
    fn all_zero_is_unlikely() {
        let o = compute(&responses([0; 10])).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.likelihood, Likelihood::Unlikely);
        assert!(!o.self_harm_flag);
    }

    #[test]
    fn band_boundaries_match_cox() {
        assert_eq!(Likelihood::from_total(9), Likelihood::Unlikely);
        assert_eq!(Likelihood::from_total(10), Likelihood::Possible);
        assert_eq!(Likelihood::from_total(12), Likelihood::Possible);
        assert_eq!(Likelihood::from_total(13), Likelihood::Probable);
        assert_eq!(Likelihood::from_total(30), Likelihood::Probable);
    }

    #[test]
    fn maximum_score_is_probable() {
        let o = compute(&responses([3; 10])).unwrap();
        assert_eq!(o.total, 30);
        assert_eq!(o.likelihood, Likelihood::Probable);
        assert!(o.self_harm_flag);
    }

    #[test]
    fn self_harm_flag_is_independent_of_total() {
        // Low total (3) but Q10 positive must still flag.
        let o = compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 0, 3])).unwrap();
        assert_eq!(o.total, 3);
        assert_eq!(o.likelihood, Likelihood::Unlikely);
        assert!(o.self_harm_flag);
        assert!(o.interpretation.contains("suicide-risk assessment"));
    }

    #[test]
    fn possible_band_without_self_harm() {
        // Spread 11 across items 1-9, Q10 zero.
        let o = compute(&responses([2, 2, 2, 2, 1, 1, 1, 0, 0, 0])).unwrap();
        assert_eq!(o.total, 11);
        assert_eq!(o.likelihood, Likelihood::Possible);
        assert!(!o.self_harm_flag);
    }

    #[test]
    fn wrong_length_and_range_are_rejected() {
        assert!(compute(&EpdsInput { responses: vec![0; 9] }).is_err());
        assert!(compute(&EpdsInput { responses: vec![0; 11] }).is_err());
        assert!(compute(&EpdsInput {
            responses: vec![4, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        })
        .is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [1, 1, 2, 1, 2, 2, 1, 2, 1, 0];
        let dynamic = Epds.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(13));
        assert_eq!(dynamic.working["likelihood"], json!("probable"));
        assert_eq!(dynamic.working["self_harm_item_flag"], json!(false));
    }

    #[test]
    fn schema_carries_input_definition() {
        let schema = Epds.input_schema();
        let def = &schema["properties"]["responses"]["definition"];
        assert!(def["excludes"][0]
            .as_str()
            .unwrap()
            .contains("NOT the literal answer position"));
        assert_eq!(def["status"], json!("draft"));
    }

    #[test]
    fn reverse_scored_items_documented() {
        assert_eq!(REVERSE_SCORED_ITEMS, [3, 5, 6, 7, 8, 9, 10]);
    }
}
