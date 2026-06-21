//! Wells score for pulmonary embolism - pretest probability of acute PE.
//!
//! Stratifies a patient with suspected PE so that low-risk patients can be
//! safely investigated with a D-dimer rather than going straight to CT pulmonary
//! angiography. Two reporting models are in common use:
//!
//! - **Two-tier** (NICE NG158, the "2-level PE Wells score"): "PE likely" at
//!   more than 4 points, "PE unlikely" at 4 or fewer. This is the model that
//!   drives the NICE unlikely + D-dimer pathway, so it is surfaced as the primary
//!   result here.
//! - **Three-tier** (original Wells et al. 2000): low at under 2, moderate at 2
//!   to 6 inclusive, high at over 6.
//!
//! The criteria are weighted, so several carry fractional (1.5) points and the
//! total runs 0-12.5. Points are therefore held as `f64`; the presented total is
//! rounded to one decimal place so the fractional weights never surface a float
//! artefact such as `4.4000000000000004`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "wells_pe";

/// Primary citation.
pub const REFERENCE: &str =
    "Wells PS, Anderson DR, Rodger M, et al. Derivation of a simple clinical model to categorize \
patients probability of pulmonary embolism: increasing the models utility with the SimpliRED \
D-dimer. Thromb Haemost. 2000;83(3):416-420. Two-level thresholds per NICE NG158.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1055/s-0037-1613830",
};

/// Wells PE inputs: seven weighted, clinician-asserted boolean criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WellsPeInput {
    /// Clinical signs and symptoms of DVT (minimum leg swelling and pain with
    /// palpation of the deep veins). Weighted 3.
    pub clinical_signs_of_dvt: bool,
    /// PE is the most likely diagnosis, or at least as likely as any alternative.
    /// Weighted 3.
    pub pe_most_likely_diagnosis: bool,
    /// Heart rate greater than 100 beats per minute. Weighted 1.5.
    pub heart_rate_over_100: bool,
    /// Immobilisation for 3 days or more, or surgery in the previous 4 weeks.
    /// Weighted 1.5.
    pub immobilisation_or_surgery: bool,
    /// Previous objectively diagnosed DVT or PE. Weighted 1.5.
    pub previous_dvt_or_pe: bool,
    /// Haemoptysis. Weighted 1.
    pub haemoptysis: bool,
    /// Malignancy: treatment within the last 6 months, or palliative. Weighted 1.
    pub malignancy: bool,
}

/// Two-tier band (NICE NG158 2-level PE Wells score).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoTier {
    /// PE unlikely: 4 points or fewer.
    Unlikely,
    /// PE likely: more than 4 points.
    Likely,
}

impl TwoTier {
    fn slug(self) -> &'static str {
        match self {
            TwoTier::Unlikely => "unlikely",
            TwoTier::Likely => "likely",
        }
    }
}

/// Three-tier band (original Wells et al. 2000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreeTier {
    /// Low probability: under 2 points.
    Low,
    /// Moderate probability: 2 to 6 points inclusive.
    Moderate,
    /// High probability: over 6 points.
    High,
}

impl ThreeTier {
    fn slug(self) -> &'static str {
        match self {
            ThreeTier::Low => "low",
            ThreeTier::Moderate => "moderate",
            ThreeTier::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct WellsPeOutcome {
    /// Total score, 0.0-12.5, rounded to one decimal place.
    pub score: f64,
    pub two_tier: TwoTier,
    pub three_tier: ThreeTier,
    pub interpretation: String,
}

/// Round to one decimal place, so the 1.5-weighted criteria never surface a
/// binary-float artefact (e.g. 4.4 stored as 4.400000000000001).
fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// Pure scoring.
pub fn compute(input: &WellsPeInput) -> Result<WellsPeOutcome, CalcError> {
    let raw = 3.0 * f64::from(input.clinical_signs_of_dvt)
        + 3.0 * f64::from(input.pe_most_likely_diagnosis)
        + 1.5 * f64::from(input.heart_rate_over_100)
        + 1.5 * f64::from(input.immobilisation_or_surgery)
        + 1.5 * f64::from(input.previous_dvt_or_pe)
        + 1.0 * f64::from(input.haemoptysis)
        + 1.0 * f64::from(input.malignancy);
    let score = round1(raw);

    // NICE NG158 two-level: likely at >4, unlikely at <=4.
    let two_tier = if score > 4.0 {
        TwoTier::Likely
    } else {
        TwoTier::Unlikely
    };

    // Wells 2000 three-level: low <2, moderate 2-6 inclusive, high >6.
    let three_tier = if score < 2.0 {
        ThreeTier::Low
    } else if score <= 6.0 {
        ThreeTier::Moderate
    } else {
        ThreeTier::High
    };

    let interpretation = match two_tier {
        TwoTier::Unlikely => format!(
            "Score {score}: PE unlikely (2-level Wells, 4 or fewer points). Offer a D-dimer; if \
positive, proceed to CT pulmonary angiogram (NICE NG158). Three-tier band: {}.",
            three_tier.slug()
        ),
        TwoTier::Likely => format!(
            "Score {score}: PE likely (2-level Wells, more than 4 points). Offer an immediate CT \
pulmonary angiogram, with interim anticoagulation if imaging is delayed (NICE NG158). Three-tier \
band: {}.",
            three_tier.slug()
        ),
    };

    Ok(WellsPeOutcome {
        score,
        two_tier,
        three_tier,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &WellsPeInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("clinical_signs_of_dvt".into(), json!(3.0 * f64::from(input.clinical_signs_of_dvt)));
    working.insert("pe_most_likely_diagnosis".into(), json!(3.0 * f64::from(input.pe_most_likely_diagnosis)));
    working.insert("heart_rate_over_100".into(), json!(1.5 * f64::from(input.heart_rate_over_100)));
    working.insert("immobilisation_or_surgery".into(), json!(1.5 * f64::from(input.immobilisation_or_surgery)));
    working.insert("previous_dvt_or_pe".into(), json!(1.5 * f64::from(input.previous_dvt_or_pe)));
    working.insert("haemoptysis".into(), json!(1.0 * f64::from(input.haemoptysis)));
    working.insert("malignancy".into(), json!(1.0 * f64::from(input.malignancy)));
    working.insert("total_score".into(), json!(o.score));
    working.insert("two_tier".into(), json!(o.two_tier.slug()));
    working.insert("three_tier".into(), json!(o.three_tier.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct WellsPe;

impl Calculator for WellsPe {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Wells Score for Pulmonary Embolism"
    }

    fn description(&self) -> &'static str {
        "Pretest probability of pulmonary embolism, guiding D-dimer vs CTPA (NICE NG158)."
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
            "title": "WellsPeInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "clinical_signs_of_dvt", "pe_most_likely_diagnosis", "heart_rate_over_100",
                "immobilisation_or_surgery", "previous_dvt_or_pe", "haemoptysis", "malignancy"
            ],
            "properties": {
                "clinical_signs_of_dvt": {
                    "type": "boolean",
                    "description": "Clinical signs and symptoms of DVT (3 points)",
                    "definition": {
                        "concept": "Clinical signs and symptoms of DVT",
                        "statement": "At least leg swelling and pain on palpation of the deep veins.",
                        "includes": ["Unilateral leg swelling", "Pain on deep-vein palpation", "Tenderness localised to the deep venous system"],
                        "excludes": ["Isolated superficial varicose veins", "Bilateral leg oedema with no localising signs"],
                        "snomedEcl": "<< 128053003 |Deep venous thrombosis (disorder)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "pe_most_likely_diagnosis": {
                    "type": "boolean",
                    "description": "PE is the most likely diagnosis, or at least as likely as any alternative (3 points)",
                    "definition": {
                        "concept": "Alternative diagnosis less likely than PE",
                        "statement": "On the treating clinician's judgement, PE is the most likely diagnosis, or no alternative diagnosis is more likely than PE.",
                        "includes": ["PE judged the single most likely cause", "PE judged at least as likely as any competing diagnosis"],
                        "excludes": ["A clear alternative diagnosis (e.g. pneumonia, ACS) judged more likely than PE"],
                        "caveats": "This is a deliberately subjective gestalt item carrying heavy weight (3 points); it depends on the assessing clinician's judgement.",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "heart_rate_over_100": {
                    "type": "boolean",
                    "description": "Heart rate greater than 100 beats per minute (1.5 points)",
                    "definition": {
                        "concept": "Tachycardia",
                        "statement": "Measured heart rate strictly greater than 100 bpm.",
                        "includes": ["Sustained heart rate > 100 bpm"],
                        "excludes": ["A heart rate of exactly 100 bpm does NOT count"],
                        "snomedEcl": "<< 3424008 |Tachycardia (finding)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "immobilisation_or_surgery": {
                    "type": "boolean",
                    "description": "Immobilisation for 3 days or more, OR surgery in the previous 4 weeks (1.5 points)",
                    "definition": {
                        "concept": "Recent immobilisation or surgery",
                        "statement": "Bedrest (except to access the bathroom) for 3 or more consecutive days, or major surgery within the previous 4 weeks.",
                        "includes": ["Bedrest >=3 days", "Major surgery within the last 4 weeks", "Recent immobilisation of a limb in plaster"],
                        "excludes": ["Long-haul travel without bedrest", "Surgery more than 4 weeks ago"],
                        "snomedEcl": "<< 40631008 |Surgical procedure (procedure)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "previous_dvt_or_pe": {
                    "type": "boolean",
                    "description": "Previous objectively diagnosed DVT or PE (1.5 points)",
                    "definition": {
                        "concept": "Previous DVT or PE",
                        "statement": "A prior, objectively diagnosed deep vein thrombosis or pulmonary embolism.",
                        "includes": ["Prior imaging-confirmed DVT", "Prior imaging-confirmed PE"],
                        "excludes": ["A self-reported or undocumented prior clot with no objective confirmation"],
                        "snomedEcl": "<< 128053003 |Deep venous thrombosis (disorder)| OR << 59282003 |Pulmonary embolism (disorder)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "haemoptysis": {
                    "type": "boolean",
                    "description": "Haemoptysis (1 point)",
                    "definition": {
                        "concept": "Haemoptysis",
                        "statement": "Coughing up of blood or blood-stained sputum.",
                        "includes": ["Frank haemoptysis", "Blood-streaked sputum"],
                        "excludes": ["Haematemesis", "Epistaxis or blood from the upper airway tracking down"],
                        "snomedEcl": "<< 66857006 |Hemoptysis (finding)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                },
                "malignancy": {
                    "type": "boolean",
                    "description": "Malignancy with treatment within 6 months, or palliative (1 point)",
                    "definition": {
                        "concept": "Active malignancy",
                        "statement": "Active cancer: treatment ongoing or within the last 6 months, or receiving palliative care.",
                        "includes": ["Cancer treated within the last 6 months", "Cancer on palliative care", "Currently on chemotherapy or radiotherapy"],
                        "excludes": ["Cancer in remission with treatment completed more than 6 months ago", "Non-melanoma skin cancer treated locally"],
                        "snomedEcl": "<< 363346000 |Malignant neoplastic disease (disorder)|",
                        "source": { "citation": "Wells PS et al. Thromb Haemost. 2000;83(3):416-420.", "url": "https://doi.org/10.1055/s-0037-1613830" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WellsPeInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> WellsPeInput {
        WellsPeInput {
            clinical_signs_of_dvt: false,
            pe_most_likely_diagnosis: false,
            heart_rate_over_100: false,
            immobilisation_or_surgery: false,
            previous_dvt_or_pe: false,
            haemoptysis: false,
            malignancy: false,
        }
    }

    #[test]
    fn all_false_is_zero_low_unlikely() {
        let o = compute(&none()).unwrap();
        assert_eq!(o.score, 0.0);
        assert_eq!(o.two_tier, TwoTier::Unlikely);
        assert_eq!(o.three_tier, ThreeTier::Low);
    }

    #[test]
    fn maximum_score_is_twelve_point_five() {
        let i = WellsPeInput {
            clinical_signs_of_dvt: true,
            pe_most_likely_diagnosis: true,
            heart_rate_over_100: true,
            immobilisation_or_surgery: true,
            previous_dvt_or_pe: true,
            haemoptysis: true,
            malignancy: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 12.5);
        assert_eq!(o.two_tier, TwoTier::Likely);
        assert_eq!(o.three_tier, ThreeTier::High);
    }

    #[test]
    fn fractional_weights_round_cleanly() {
        // Three 1.5-weight items = 4.5, which in raw binary float would be exact
        // here but the rounding guard must keep it tidy regardless.
        let mut i = none();
        i.heart_rate_over_100 = true;
        i.immobilisation_or_surgery = true;
        i.previous_dvt_or_pe = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4.5);
        // No float artefact in the stored value.
        assert_eq!(format!("{}", o.score), "4.5");
    }

    #[test]
    fn two_tier_boundary_at_four() {
        // Exactly 4 is unlikely; just over 4 is likely.
        let mut at_four = none();
        at_four.pe_most_likely_diagnosis = true; // 3
        at_four.haemoptysis = true; // 1 -> 4.0
        let o = compute(&at_four).unwrap();
        assert_eq!(o.score, 4.0);
        assert_eq!(o.two_tier, TwoTier::Unlikely);

        let mut over_four = none();
        over_four.pe_most_likely_diagnosis = true; // 3
        over_four.heart_rate_over_100 = true; // 1.5 -> 4.5
        let o = compute(&over_four).unwrap();
        assert_eq!(o.score, 4.5);
        assert_eq!(o.two_tier, TwoTier::Likely);
    }

    #[test]
    fn three_tier_boundaries() {
        // 1.5 (one 1.5-weight item) -> low (<2).
        let mut i = none();
        i.heart_rate_over_100 = true;
        assert_eq!(compute(&i).unwrap().three_tier, ThreeTier::Low);

        // 2.0 (two 1-weight items) -> moderate (lower edge inclusive).
        let mut i = none();
        i.haemoptysis = true;
        i.malignancy = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 2.0);
        assert_eq!(o.three_tier, ThreeTier::Moderate);

        // 6.0 (two 3-weight items) -> moderate (upper edge inclusive).
        let mut i = none();
        i.clinical_signs_of_dvt = true;
        i.pe_most_likely_diagnosis = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 6.0);
        assert_eq!(o.three_tier, ThreeTier::Moderate);

        // 6.5 (3 + 3 + 0.5? no: 3 + 1.5 + 1.5 + ...). Build >6 -> high.
        let mut i = none();
        i.clinical_signs_of_dvt = true; // 3
        i.pe_most_likely_diagnosis = true; // 3
        i.haemoptysis = true; // 1 -> 7.0
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 7.0);
        assert_eq!(o.three_tier, ThreeTier::High);
    }

    #[test]
    fn working_breakdown_sums_to_total() {
        let mut i = none();
        i.clinical_signs_of_dvt = true; // 3
        i.heart_rate_over_100 = true; // 1.5
        i.malignancy = true; // 1 -> 5.5
        let resp = build_response(&i).unwrap();
        assert_eq!(resp.working["total_score"], json!(5.5));
        assert_eq!(resp.working["clinical_signs_of_dvt"], json!(3.0));
        assert_eq!(resp.working["heart_rate_over_100"], json!(1.5));
        assert_eq!(resp.working["malignancy"], json!(1.0));
        assert_eq!(resp.working["two_tier"], json!("likely"));
        assert_eq!(resp.working["three_tier"], json!("moderate"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "clinical_signs_of_dvt": true,
            "pe_most_likely_diagnosis": false,
            "heart_rate_over_100": true,
            "immobilisation_or_surgery": false,
            "previous_dvt_or_pe": false,
            "haemoptysis": false,
            "malignancy": false
        });
        let mut typed = none();
        typed.clinical_signs_of_dvt = true;
        typed.heart_rate_over_100 = true;
        let dynamic = WellsPe.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(4.5));
    }

    #[test]
    fn rejects_malformed_input() {
        let value = json!({ "clinical_signs_of_dvt": "yes" });
        assert!(WellsPe.calculate(&value).is_err());
    }

    #[test]
    fn schema_lists_all_seven_criteria() {
        let schema = WellsPe.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 7);
    }
}
