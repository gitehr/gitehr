//! ASRS-v1.1 — Adult ADHD Self-Report Scale screener.
//!
//! 18-item WHO-validated screener. Part A (items 1–6) is the validated screen;
//! Part B (items 7–18) provides additional clinical detail and does not affect
//! the screen result. Ported verbatim from the web calculator
//! (`calc-web/calculators/adhd-questionnaire-asrs111.html`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "asrs";

/// Primary citation (matches the payload dispatched by the web calculator).
pub const REFERENCE: &str =
    "Kessler RC et al. (2005). Psychol Med. 35(2):245-56. doi:10.1017/S0033291704002892";

/// Number of items in the questionnaire.
pub const ITEM_COUNT: usize = 18;

/// The 18 frequency responses, each 0 (Never) – 4 (Very Often), in question
/// order Q1–Q18.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsrsInput {
    pub responses: Vec<u8>,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrsOutcome {
    /// Count of Part A items meeting their frequency threshold (0–6).
    pub part_a_positive: u8,
    /// Sum of Part A item scores (0–24).
    pub part_a_total: u16,
    /// Sum of Part B item scores (0–48).
    pub part_b_total: u16,
    /// Overall total (0–72).
    pub total: u16,
    /// True if ≥ 4 Part A items are positive.
    pub screen_positive: bool,
    /// Per-item positivity for Part A items 1–6.
    pub part_a_item_positive: [bool; 6],
    pub interpretation: String,
}

/// Part A item threshold: items 1–3 (index 0–2) positive at ≥ 2 (Sometimes),
/// items 4–6 (index 3–5) positive at ≥ 3 (Often).
fn part_a_item_positive(index: usize, score: u8) -> bool {
    if index < 3 {
        score >= 2
    } else {
        score >= 3
    }
}

fn interpret(part_a_positive: u8) -> String {
    if part_a_positive >= 4 {
        format!(
            "Positive screen: {part_a_positive}/6 Part A items meet the frequency threshold. \
These symptoms are highly consistent with adult ADHD. A formal diagnostic \
assessment by a qualified clinician is recommended. This result is not a \
diagnosis of ADHD."
        )
    } else {
        format!(
            "Negative screen: {part_a_positive}/6 Part A items meet the frequency threshold \
(4 required for a positive screen). Reported symptoms are less consistent with \
adult ADHD, though clinical judgement should always be applied. If clinical \
concern persists, further assessment is warranted."
        )
    }
}

/// Pure scoring. Mirrors the web calculator's `scoreAll` + `interpret`.
pub fn compute(input: &AsrsInput) -> Result<AsrsOutcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }
    for (i, &v) in input.responses.iter().enumerate() {
        if v > 4 {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {v} is out of range 0–4",
                i + 1
            )));
        }
    }

    let mut part_a_item_positive_arr = [false; 6];
    let mut part_a_positive = 0u8;
    let mut part_a_total = 0u16;
    for i in 0..6 {
        let v = input.responses[i];
        part_a_total += v as u16;
        let pos = part_a_item_positive(i, v);
        part_a_item_positive_arr[i] = pos;
        if pos {
            part_a_positive += 1;
        }
    }

    let part_b_total: u16 = input.responses[6..ITEM_COUNT].iter().map(|&v| v as u16).sum();
    let total = part_a_total + part_b_total;
    let screen_positive = part_a_positive >= 4;

    Ok(AsrsOutcome {
        part_a_positive,
        part_a_total,
        part_b_total,
        total,
        screen_positive,
        part_a_item_positive: part_a_item_positive_arr,
        interpretation: interpret(part_a_positive),
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &AsrsInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "part_a_screen_result".into(),
        json!(if o.screen_positive { "POSITIVE" } else { "NEGATIVE" }),
    );
    working.insert("part_a_positive_item_count".into(), json!(o.part_a_positive));
    working.insert("part_a_total_score".into(), json!(o.part_a_total));
    working.insert("part_b_total_score".into(), json!(o.part_b_total));
    working.insert("total_score".into(), json!(o.total));
    working.insert("answers".into(), json!(input.responses));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.part_a_positive),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Asrs;

impl Calculator for Asrs {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "ASRS-v1.1 Adult ADHD Screener"
    }

    fn description(&self) -> &'static str {
        "18-item WHO-validated screener for adult ADHD; Part A (items 1–6) is the validated screen."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "AsrsInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "18 frequency responses (Q1–Q18), each 0=Never, 1=Rarely, 2=Sometimes, 3=Often, 4=Very Often",
                    "items": { "type": "integer", "minimum": 0, "maximum": 4 },
                    "minItems": 18,
                    "maxItems": 18
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AsrsInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 18]) -> AsrsInput {
        AsrsInput { responses: v.to_vec() }
    }

    #[test]
    fn all_zero_is_negative() {
        let o = compute(&responses([0; 18])).unwrap();
        assert_eq!(o.part_a_positive, 0);
        assert_eq!(o.total, 0);
        assert!(!o.screen_positive);
    }

    #[test]
    fn item_thresholds_differ_between_q1_3_and_q4_6() {
        // Q1–3 positive at 2, Q4–6 need 3: a row of all-2s gives exactly 3 positives.
        let mut v = [0u8; 18];
        v[0] = 2;
        v[1] = 2;
        v[2] = 2; // three positives (>= 2)
        v[3] = 2;
        v[4] = 2;
        v[5] = 2; // not positive (need >= 3)
        let o = compute(&responses(v)).unwrap();
        assert_eq!(o.part_a_positive, 3);
        assert!(!o.screen_positive);
    }

    #[test]
    fn four_positives_is_a_positive_screen() {
        let mut v = [0u8; 18];
        v[0] = 2;
        v[1] = 2;
        v[2] = 2;
        v[3] = 3; // fourth positive (>= 3)
        let o = compute(&responses(v)).unwrap();
        assert_eq!(o.part_a_positive, 4);
        assert!(o.screen_positive);
    }

    #[test]
    fn totals_split_part_a_and_b() {
        let o = compute(&responses([4; 18])).unwrap();
        assert_eq!(o.part_a_total, 24);
        assert_eq!(o.part_b_total, 48);
        assert_eq!(o.total, 72);
        assert_eq!(o.part_a_positive, 6);
        assert!(o.screen_positive);
    }

    #[test]
    fn wrong_length_is_rejected() {
        assert!(compute(&AsrsInput { responses: vec![0; 17] }).is_err());
        assert!(compute(&AsrsInput { responses: vec![0; 19] }).is_err());
    }

    #[test]
    fn out_of_range_is_rejected() {
        let mut v = vec![0u8; 18];
        v[5] = 5;
        assert!(compute(&AsrsInput { responses: v }).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [2, 2, 2, 3, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let dynamic = Asrs.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.working["part_a_screen_result"], json!("POSITIVE"));
    }
}
