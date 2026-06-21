//! IPSS - International Prostate Symptom Score (AUA Symptom Index).
//!
//! Seven lower-urinary-tract symptom items, each rated 0-5 over the **last
//! month**, summed to a 0-35 symptom score with standard bands (mild 0-7,
//! moderate 8-19, severe 20-35). The IPSS adds a single quality-of-life item
//! (0-6) that is reported alongside but, by convention, **not** added to the
//! 0-35 symptom total.
//!
//! The scoring algorithm is implemented from the primary literature
//! (Barry et al., J Urol 1992); it is the published method, distinct from the
//! verbatim official questionnaire wording distributed by the Mapi Research
//! Trust. The item descriptors here are paraphrased clinical labels, not the
//! licensed questionnaire text.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "ipss";

/// Distribution licence for the algorithm. The IPSS/AUA-SI *scoring method*
/// (sum of seven 0-5 items, bands at 7/19) is a published clinical algorithm,
/// implemented here from the primary literature and not subject to copyright.
/// The verbatim official questionnaire wording is separately copyrighted and
/// distributed by the Mapi Research Trust; it is **not** reproduced here.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - scoring algorithm implemented from the primary literature (Barry et al., J Urol 1992); verbatim official questionnaire wording is separately licensed via Mapi Research Trust and is not reproduced",
    source_url: "https://eprovide.mapi-trust.org/instruments/international-prostate-symptom-score",
};

/// Primary citation.
pub const REFERENCE: &str = "Barry MJ, Fowler FJ Jr, O'Leary MP, et al. The American Urological \
Association Symptom Index for Benign Prostatic Hyperplasia. J Urol. 1992;148(5):1549-1557. \
doi:10.1016/S0022-5347(17)36966-5";

/// Number of symptom items contributing to the 0-35 total.
pub const ITEM_COUNT: usize = 7;

/// Maximum value for a single symptom item.
pub const ITEM_MAX: u8 = 5;

/// Maximum value for the quality-of-life item.
pub const QOL_MAX: u8 = 6;

/// The seven IPSS symptom responses, each 0-5, in canonical AUA-SI order, plus
/// the optional quality-of-life item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpssInput {
    /// Seven symptom responses (Q1-Q7), each 0-5, over the last month:
    /// Q1 incomplete emptying, Q2 frequency, Q3 intermittency, Q4 urgency,
    /// Q5 weak stream, Q6 straining, Q7 nocturia.
    pub responses: Vec<u8>,
    /// Optional quality-of-life item (0 = delighted, 6 = terrible). Reported but
    /// **not** added to the 0-35 symptom total.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_of_life: Option<u8>,
}

/// Symptom severity band implied by the 0-35 total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
}

impl Severity {
    /// Standard IPSS bands (Barry 1992).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=7 => Severity::Mild,
            8..=19 => Severity::Moderate,
            _ => Severity::Severe,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Severity::Mild => "mild",
            Severity::Moderate => "moderate",
            Severity::Severe => "severe",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpssOutcome {
    /// Total symptom score (0-35) from the seven symptom items.
    pub total: u16,
    pub severity: Severity,
    /// The quality-of-life item (0-6), echoed back if supplied; not part of the total.
    pub quality_of_life: Option<u8>,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &IpssInput) -> Result<IpssOutcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }
    for (i, &v) in input.responses.iter().enumerate() {
        if v > ITEM_MAX {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {v} is out of range 0-{ITEM_MAX}",
                i + 1
            )));
        }
    }
    if let Some(qol) = input.quality_of_life
        && qol > QOL_MAX
    {
        return Err(CalcError::InvalidInput(format!(
            "quality_of_life = {qol} is out of range 0-{QOL_MAX}"
        )));
    }

    let total: u16 = input.responses.iter().map(|&v| v as u16).sum();
    let severity = Severity::from_total(total);

    let mut interpretation = format!(
        "Symptom score {total}/35 indicates {} lower urinary tract symptoms.",
        severity.label()
    );
    if let Some(qol) = input.quality_of_life {
        interpretation.push_str(&format!(
            " Quality-of-life rating is {qol}/6 (0 = delighted, 6 = terrible); this item is \
reported separately and is not part of the 0-35 symptom score."
        ));
    }
    interpretation.push_str(" IPSS grades symptom severity to guide management; it is not a diagnosis.");

    Ok(IpssOutcome {
        total,
        severity,
        quality_of_life: input.quality_of_life,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &IpssInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("symptom_score".into(), json!(o.total));
    working.insert("severity".into(), json!(o.severity.label()));
    working.insert("answers".into(), json!(input.responses));
    if let Some(qol) = o.quality_of_life {
        working.insert("quality_of_life".into(), json!(qol));
    }

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Ipss;

impl Calculator for Ipss {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "IPSS - International Prostate Symptom Score"
    }

    fn description(&self) -> &'static str {
        "Seven-item lower urinary tract symptom score (0-35) for benign prostatic hyperplasia; \
bands mild 0-7, moderate 8-19, severe 20-35, with an optional quality-of-life item (0-6)."
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
            "title": "IpssInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Seven symptom responses (Q1-Q7), each 0=Not at all to 5=Almost always (Q7 nocturia: number of times per night, 0-5)",
                    "items": { "type": "integer", "minimum": 0, "maximum": 5 },
                    "minItems": 7,
                    "maxItems": 7,
                    "definition": {
                        "concept": "IPSS / AUA-SI symptom item responses",
                        "statement": "Each item rates how often the patient has been bothered by the symptom over the LAST MONTH.",
                        "includes": [
                            "Q1 Incomplete emptying",
                            "Q2 Frequency",
                            "Q3 Intermittency",
                            "Q4 Urgency",
                            "Q5 Weak stream",
                            "Q6 Straining",
                            "Q7 Nocturia"
                        ],
                        "caveats": "Bands are mild 0-7, moderate 8-19, severe 20-35. The quality-of-life item is reported but not added to the 0-35 symptom total.",
                        "source": {
                            "citation": "Barry MJ, Fowler FJ Jr, O'Leary MP, et al. J Urol. 1992;148(5):1549-1557.",
                            "url": "https://doi.org/10.1016/S0022-5347(17)36966-5"
                        },
                        "status": "draft"
                    }
                },
                "quality_of_life": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 6,
                    "description": "Optional quality-of-life due to urinary symptoms (0=delighted to 6=terrible); reported but not added to the symptom score"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: IpssInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 7]) -> IpssInput {
        IpssInput { responses: v.to_vec(), quality_of_life: None }
    }

    #[test]
    fn all_zero_is_mild() {
        let o = compute(&responses([0; 7])).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.severity, Severity::Mild);
        assert_eq!(o.quality_of_life, None);
    }

    #[test]
    fn worked_example_sums_and_bands() {
        // 2+3+1+4+2+0+5 = 17 -> moderate.
        let o = compute(&responses([2, 3, 1, 4, 2, 0, 5])).unwrap();
        assert_eq!(o.total, 17);
        assert_eq!(o.severity, Severity::Moderate);
        assert!(o.interpretation.contains("17/35"));
        assert!(o.interpretation.contains("moderate"));
    }

    #[test]
    fn band_boundaries_match_barry() {
        assert_eq!(Severity::from_total(0), Severity::Mild);
        assert_eq!(Severity::from_total(7), Severity::Mild);
        assert_eq!(Severity::from_total(8), Severity::Moderate);
        assert_eq!(Severity::from_total(19), Severity::Moderate);
        assert_eq!(Severity::from_total(20), Severity::Severe);
        assert_eq!(Severity::from_total(35), Severity::Severe);
    }

    #[test]
    fn maximum_score_is_severe() {
        let o = compute(&responses([5; 7])).unwrap();
        assert_eq!(o.total, 35);
        assert_eq!(o.severity, Severity::Severe);
    }

    #[test]
    fn quality_of_life_is_reported_not_scored() {
        let input = IpssInput { responses: vec![1; 7], quality_of_life: Some(6) };
        let o = compute(&input).unwrap();
        // Total is unchanged by the QoL item (7 symptom points only).
        assert_eq!(o.total, 7);
        assert_eq!(o.severity, Severity::Mild);
        assert_eq!(o.quality_of_life, Some(6));
        assert!(o.interpretation.contains("Quality-of-life rating is 6/6"));
        assert!(o.interpretation.contains("not part of the 0-35"));

        let r = build_response(&input).unwrap();
        // Result remains the symptom score, not symptom + QoL.
        assert_eq!(r.result, json!(7));
        assert_eq!(r.working["quality_of_life"], json!(6));
        assert!(!r.working.contains_key("quality_of_life_in_total"));
    }

    #[test]
    fn quality_of_life_absent_is_omitted_from_working() {
        let r = build_response(&responses([1; 7])).unwrap();
        assert!(!r.working.contains_key("quality_of_life"));
    }

    #[test]
    fn wrong_length_and_range_are_rejected() {
        assert!(compute(&IpssInput { responses: vec![0; 6], quality_of_life: None }).is_err());
        assert!(compute(&IpssInput { responses: vec![0; 8], quality_of_life: None }).is_err());
        assert!(compute(&IpssInput { responses: vec![6, 0, 0, 0, 0, 0, 0], quality_of_life: None }).is_err());
    }

    #[test]
    fn out_of_range_quality_of_life_is_rejected() {
        let input = IpssInput { responses: vec![0; 7], quality_of_life: Some(7) };
        assert!(compute(&input).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [2, 3, 1, 4, 2, 0, 5];
        let dynamic = Ipss
            .calculate(&json!({ "responses": arr, "quality_of_life": 3 }))
            .unwrap();
        let typed = build_response(&IpssInput {
            responses: arr.to_vec(),
            quality_of_life: Some(3),
        })
        .unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(17));
        assert_eq!(dynamic.working["severity"], json!("moderate"));
        assert_eq!(dynamic.working["quality_of_life"], json!(3));
    }

    #[test]
    fn dynamic_calculate_works_without_quality_of_life() {
        let arr = [1, 1, 1, 1, 1, 1, 1];
        let dynamic = Ipss.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(7));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Ipss.calculate(&json!({ "responses": "nope" })).is_err());
    }
}
