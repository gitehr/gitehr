//! Padua Prediction Score - VTE risk in hospitalised medical inpatients.
//!
//! An 11-item weighted point score (Barbar et al. J Thromb Haemost 2010) that
//! stratifies the risk of venous thromboembolism (VTE) in non-surgical
//! hospitalised patients and guides whether pharmacological thromboprophylaxis
//! should be considered (NICE NG89). A total of 4 or more is high risk; below 4
//! is low risk.
//!
//! Several criteria are clinician-asserted predicates whose TRUE/FALSE
//! conditions are easy to get subtly wrong, so the ambiguous ones carry
//! `definition` blocks: "active cancer" (local/distant metastases and/or
//! chemo/radiotherapy in the preceding 6 months), "reduced mobility" (bedrest
//! with bathroom privileges for at least 3 days), and "previous VTE", which
//! explicitly EXCLUDES superficial vein thrombosis.
//!
//! As with CHA2DS2-VASc, age is a single numeric input and the >=70 point is
//! derived, so a contradictory age input is impossible.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "padua";

/// Primary citation.
pub const REFERENCE: &str =
    "Barbar S, Noventa F, Rossetto V, et al. A risk assessment model for the identification of \
hospitalized medical patients at risk for venous thromboembolism: the Padua Prediction Score. \
J Thromb Haemost. 2010;8(11):2450-2457. Threshold and prophylaxis guidance per NICE NG89.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1111/j.1538-7836.2010.04044.x",
};

/// The high-risk threshold: a total score of this value or above.
const HIGH_RISK_THRESHOLD: u8 = 4;

/// Padua Prediction Score inputs. Age is numeric; the >=70 point is derived.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaduaInput {
    /// Age in years.
    pub age: u8,
    /// Active cancer (3 points).
    pub active_cancer: bool,
    /// Previous VTE, excluding superficial vein thrombosis (3 points).
    pub previous_vte: bool,
    /// Reduced mobility - bedrest with bathroom privileges for >=3 days (3 points).
    pub reduced_mobility: bool,
    /// Known thrombophilic condition (3 points).
    pub thrombophilia: bool,
    /// Recent (<=1 month) trauma and/or surgery (2 points).
    pub recent_trauma_or_surgery: bool,
    /// Heart and/or respiratory failure (1 point).
    pub heart_or_respiratory_failure: bool,
    /// Acute myocardial infarction or ischaemic stroke (1 point).
    pub acute_mi_or_ischaemic_stroke: bool,
    /// Acute infection and/or rheumatological disorder (1 point).
    pub acute_infection_or_rheumatological: bool,
    /// Obesity, BMI >= 30 (1 point).
    pub obesity: bool,
    /// Ongoing hormonal treatment (1 point).
    pub ongoing_hormonal_treatment: bool,
}

/// VTE risk band (Barbar et al. 2010; thromboprophylaxis guidance per NICE NG89).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Score < 4: low risk of VTE.
    Low,
    /// Score >= 4: high risk of VTE.
    High,
}

impl RiskBand {
    fn slug(self) -> &'static str {
        match self {
            RiskBand::Low => "low",
            RiskBand::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaduaOutcome {
    /// Total score (0-20).
    pub score: u8,
    /// Points contributed by the age criterion (0 or 1).
    pub age_points: u8,
    pub risk_band: RiskBand,
    pub interpretation: String,
}

/// Points contributed by the age criterion: 1 if age >= 70, else 0.
fn age_points(age: u8) -> u8 {
    u8::from(age >= 70)
}

/// Pure scoring.
pub fn compute(input: &PaduaInput) -> Result<PaduaOutcome, CalcError> {
    let age_points = age_points(input.age);

    let score = 3 * u8::from(input.active_cancer)
        + 3 * u8::from(input.previous_vte)
        + 3 * u8::from(input.reduced_mobility)
        + 3 * u8::from(input.thrombophilia)
        + 2 * u8::from(input.recent_trauma_or_surgery)
        + age_points
        + u8::from(input.heart_or_respiratory_failure)
        + u8::from(input.acute_mi_or_ischaemic_stroke)
        + u8::from(input.acute_infection_or_rheumatological)
        + u8::from(input.obesity)
        + u8::from(input.ongoing_hormonal_treatment);

    let risk_band = if score >= HIGH_RISK_THRESHOLD {
        RiskBand::High
    } else {
        RiskBand::Low
    };

    let interpretation = match risk_band {
        RiskBand::High => format!(
            "Score {score}: high risk of VTE (score >=4). Consider pharmacological \
thromboprophylaxis unless contraindicated, weighing bleeding risk (NICE NG89)."
        ),
        RiskBand::Low => format!(
            "Score {score}: low risk of VTE (score <4). Pharmacological thromboprophylaxis is not \
routinely indicated on this score alone (NICE NG89)."
        ),
    };

    Ok(PaduaOutcome {
        score,
        age_points,
        risk_band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &PaduaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("active_cancer".into(), json!(3 * u8::from(input.active_cancer)));
    working.insert("previous_vte".into(), json!(3 * u8::from(input.previous_vte)));
    working.insert("reduced_mobility".into(), json!(3 * u8::from(input.reduced_mobility)));
    working.insert("thrombophilia".into(), json!(3 * u8::from(input.thrombophilia)));
    working.insert(
        "recent_trauma_or_surgery".into(),
        json!(2 * u8::from(input.recent_trauma_or_surgery)),
    );
    working.insert("age_points".into(), json!(o.age_points));
    working.insert(
        "heart_or_respiratory_failure".into(),
        json!(u8::from(input.heart_or_respiratory_failure)),
    );
    working.insert(
        "acute_mi_or_ischaemic_stroke".into(),
        json!(u8::from(input.acute_mi_or_ischaemic_stroke)),
    );
    working.insert(
        "acute_infection_or_rheumatological".into(),
        json!(u8::from(input.acute_infection_or_rheumatological)),
    );
    working.insert("obesity".into(), json!(u8::from(input.obesity)));
    working.insert(
        "ongoing_hormonal_treatment".into(),
        json!(u8::from(input.ongoing_hormonal_treatment)),
    );
    working.insert("risk_band".into(), json!(o.risk_band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Padua;

impl Calculator for Padua {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Padua Prediction Score (VTE risk)"
    }

    fn description(&self) -> &'static str {
        "VTE risk in hospitalised medical inpatients, guiding thromboprophylaxis (NICE NG89)."
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
            "title": "PaduaInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "active_cancer", "previous_vte", "reduced_mobility", "thrombophilia",
                "recent_trauma_or_surgery", "heart_or_respiratory_failure",
                "acute_mi_or_ischaemic_stroke", "acute_infection_or_rheumatological",
                "obesity", "ongoing_hormonal_treatment"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (>=70 scores 1)"
                },
                "active_cancer": {
                    "type": "boolean",
                    "description": "Active cancer - local/distant metastases and/or chemo/radiotherapy in the past 6 months (3 points)",
                    "definition": {
                        "concept": "Active cancer",
                        "statement": "The patient has local or distant metastases, and/or has received chemotherapy or radiotherapy within the previous 6 months.",
                        "includes": ["Known local or distant metastases", "Chemotherapy in the preceding 6 months", "Radiotherapy in the preceding 6 months"],
                        "excludes": ["Cancer in long-term remission with no treatment in the past 6 months and no metastases"],
                        "snomedEcl": "<< 363346000 |Malignant neoplastic disease (disorder)|",
                        "source": { "citation": "Barbar S et al. J Thromb Haemost. 2010;8(11):2450-2457.", "url": "https://doi.org/10.1111/j.1538-7836.2010.04044.x" },
                        "status": "draft"
                    }
                },
                "previous_vte": {
                    "type": "boolean",
                    "description": "Previous VTE, EXCLUDING superficial vein thrombosis (3 points)",
                    "definition": {
                        "concept": "Previous venous thromboembolism",
                        "statement": "A history of deep vein thrombosis or pulmonary embolism.",
                        "includes": ["Prior deep vein thrombosis (DVT)", "Prior pulmonary embolism (PE)"],
                        "excludes": ["Superficial vein thrombosis does NOT count"],
                        "snomedEcl": "<< 128053003 |Deep venous thrombosis (disorder)| OR << 59282003 |Pulmonary embolism (disorder)|",
                        "source": { "citation": "Barbar S et al. J Thromb Haemost. 2010;8(11):2450-2457.", "url": "https://doi.org/10.1111/j.1538-7836.2010.04044.x" },
                        "status": "draft"
                    }
                },
                "reduced_mobility": {
                    "type": "boolean",
                    "description": "Reduced mobility - bedrest with bathroom privileges for >=3 days (3 points)",
                    "definition": {
                        "concept": "Reduced mobility",
                        "statement": "Anticipated bedrest with bathroom privileges, due to the patient's limitations or on physician order, for at least 3 days.",
                        "includes": ["Bedrest with bathroom privileges expected to last >=3 days"],
                        "excludes": ["Independently mobile patients", "Short-lived immobility expected to resolve before 3 days"],
                        "source": { "citation": "Barbar S et al. J Thromb Haemost. 2010;8(11):2450-2457.", "url": "https://doi.org/10.1111/j.1538-7836.2010.04044.x" },
                        "status": "draft"
                    }
                },
                "thrombophilia": {
                    "type": "boolean",
                    "description": "Known thrombophilic condition (3 points)",
                    "definition": {
                        "concept": "Thrombophilic condition",
                        "statement": "A known inherited or acquired thrombophilia.",
                        "includes": ["Antithrombin, protein C or protein S deficiency", "Factor V Leiden", "Prothrombin G20210A mutation", "Antiphospholipid syndrome"],
                        "snomedEcl": "<< 439001009 |Acquired thrombophilia (disorder)| OR << 234467004 |Inherited thrombophilia (disorder)|",
                        "source": { "citation": "Barbar S et al. J Thromb Haemost. 2010;8(11):2450-2457.", "url": "https://doi.org/10.1111/j.1538-7836.2010.04044.x" },
                        "status": "draft"
                    }
                },
                "recent_trauma_or_surgery": {
                    "type": "boolean",
                    "description": "Recent (<=1 month) trauma and/or surgery (2 points)"
                },
                "heart_or_respiratory_failure": {
                    "type": "boolean",
                    "description": "Heart and/or respiratory failure (1 point)"
                },
                "acute_mi_or_ischaemic_stroke": {
                    "type": "boolean",
                    "description": "Acute myocardial infarction or ischaemic stroke (1 point)"
                },
                "acute_infection_or_rheumatological": {
                    "type": "boolean",
                    "description": "Acute infection and/or rheumatological disorder (1 point)"
                },
                "obesity": {
                    "type": "boolean",
                    "description": "Obesity, BMI >= 30 (1 point)"
                },
                "ongoing_hormonal_treatment": {
                    "type": "boolean",
                    "description": "Ongoing hormonal treatment (1 point)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: PaduaInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(age: u8) -> PaduaInput {
        PaduaInput {
            age,
            active_cancer: false,
            previous_vte: false,
            reduced_mobility: false,
            thrombophilia: false,
            recent_trauma_or_surgery: false,
            heart_or_respiratory_failure: false,
            acute_mi_or_ischaemic_stroke: false,
            acute_infection_or_rheumatological: false,
            obesity: false,
            ongoing_hormonal_treatment: false,
        }
    }

    #[test]
    fn age_derivation() {
        assert_eq!(age_points(69), 0);
        assert_eq!(age_points(70), 1);
        assert_eq!(age_points(85), 1);
    }

    #[test]
    fn all_false_young_is_zero_low() {
        let o = compute(&base(40)).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk_band, RiskBand::Low);
    }

    #[test]
    fn worked_example_active_cancer_age_obesity() {
        // 72yo, active cancer (3) + age>=70 (1) + obesity (1) = 5 -> high.
        let mut i = base(72);
        i.active_cancer = true;
        i.obesity = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 5);
        assert_eq!(o.age_points, 1);
        assert_eq!(o.risk_band, RiskBand::High);
        assert!(o.interpretation.contains("high risk"));
    }

    #[test]
    fn threshold_boundary_at_four() {
        // Score exactly 3 (single 3-point item) is low; one more point tips to high.
        let mut low = base(40);
        low.active_cancer = true;
        let lo = compute(&low).unwrap();
        assert_eq!(lo.score, 3);
        assert_eq!(lo.risk_band, RiskBand::Low);

        // 3 + 1 (obesity) = 4 -> high.
        let mut high = low;
        high.obesity = true;
        let hi = compute(&high).unwrap();
        assert_eq!(hi.score, 4);
        assert_eq!(hi.risk_band, RiskBand::High);
    }

    #[test]
    fn age_alone_does_not_reach_threshold() {
        // age>=70 contributes a single point: low risk on age alone.
        let o = compute(&base(80)).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.risk_band, RiskBand::Low);
    }

    #[test]
    fn maximum_score_is_twenty() {
        let i = PaduaInput {
            age: 80,
            active_cancer: true,
            previous_vte: true,
            reduced_mobility: true,
            thrombophilia: true,
            recent_trauma_or_surgery: true,
            heart_or_respiratory_failure: true,
            acute_mi_or_ischaemic_stroke: true,
            acute_infection_or_rheumatological: true,
            obesity: true,
            ongoing_hormonal_treatment: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 20);
        assert_eq!(o.risk_band, RiskBand::High);
    }

    #[test]
    fn three_point_items_weighted_correctly() {
        let mut i = base(40);
        i.previous_vte = true;
        assert_eq!(compute(&i).unwrap().score, 3);

        let mut j = base(40);
        j.reduced_mobility = true;
        assert_eq!(compute(&j).unwrap().score, 3);

        let mut k = base(40);
        k.thrombophilia = true;
        assert_eq!(compute(&k).unwrap().score, 3);

        let mut t = base(40);
        t.recent_trauma_or_surgery = true;
        assert_eq!(compute(&t).unwrap().score, 2);
    }

    #[test]
    fn working_map_breaks_out_components() {
        let mut i = base(72);
        i.active_cancer = true;
        i.obesity = true;
        let resp = build_response(&i).unwrap();
        assert_eq!(resp.working["total_score"], json!(5));
        assert_eq!(resp.working["active_cancer"], json!(3));
        assert_eq!(resp.working["age_points"], json!(1));
        assert_eq!(resp.working["obesity"], json!(1));
        assert_eq!(resp.working["previous_vte"], json!(0));
        assert_eq!(resp.working["risk_band"], json!("high"));
    }

    #[test]
    fn previous_vte_definition_excludes_superficial() {
        let schema = Padua.input_schema();
        let excludes = &schema["properties"]["previous_vte"]["definition"]["excludes"];
        assert!(excludes[0]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("superficial"));
    }

    #[test]
    fn validation_rejects_missing_field() {
        // Missing `ongoing_hormonal_treatment`.
        let value = json!({
            "age": 70, "active_cancer": false, "previous_vte": false,
            "reduced_mobility": false, "thrombophilia": false,
            "recent_trauma_or_surgery": false, "heart_or_respiratory_failure": false,
            "acute_mi_or_ischaemic_stroke": false, "acute_infection_or_rheumatological": false,
            "obesity": false
        });
        let err = Padua.calculate(&value).unwrap_err();
        assert!(matches!(err, CalcError::InvalidInput(_)));
    }

    #[test]
    fn validation_rejects_wrong_type() {
        let mut value = json!({
            "age": 70, "active_cancer": "yes", "previous_vte": false,
            "reduced_mobility": false, "thrombophilia": false,
            "recent_trauma_or_surgery": false, "heart_or_respiratory_failure": false,
            "acute_mi_or_ischaemic_stroke": false, "acute_infection_or_rheumatological": false,
            "obesity": false, "ongoing_hormonal_treatment": false
        });
        assert!(Padua.calculate(&value).is_err());
        // A valid version of the same shape succeeds.
        value["active_cancer"] = json!(false);
        assert!(Padua.calculate(&value).is_ok());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 72, "active_cancer": true, "previous_vte": false,
            "reduced_mobility": false, "thrombophilia": false,
            "recent_trauma_or_surgery": false, "heart_or_respiratory_failure": false,
            "acute_mi_or_ischaemic_stroke": false, "acute_infection_or_rheumatological": false,
            "obesity": true, "ongoing_hormonal_treatment": false
        });
        let mut typed = base(72);
        typed.active_cancer = true;
        typed.obesity = true;
        let dynamic = Padua.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(5));
    }
}
