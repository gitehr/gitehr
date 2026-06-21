//! Wells score (DVT) - clinical pre-test probability of deep vein thrombosis.
//!
//! Ten clinician-asserted criteria, each worth +1 point except the last
//! ("alternative diagnosis at least as likely as DVT"), which scores -2. The
//! total runs from -2 to 9.
//!
//! Two interpretations are encoded:
//! - The two-level stratification used by NICE NG158 (DVT *likely* at a score
//!   of 2 or more; *unlikely* at 1 or less), which drives the choice between a
//!   proximal leg vein ultrasound and a D-dimer test.
//! - The older three-level stratification from the original derivation (low
//!   <=0, moderate 1-2, high >=3), surfaced as a caveat for context.
//!
//! Several criteria carry input definitions because their TRUE/FALSE conditions
//! are easy to get subtly wrong: "active cancer" has a time window, the calf
//! measurement has a fixed anatomical landmark, and the -2 "alternative
//! diagnosis" item is a clinical gestalt rather than a tick-box finding.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "wells_dvt";

/// Primary citation.
pub const REFERENCE: &str =
    "Wells PS, Anderson DR, Rodger M, et al. Evaluation of D-dimer in the diagnosis of suspected \
deep-vein thrombosis. N Engl J Med. 2003;349(13):1227-1235. Two-level interpretation per NICE \
NG158.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1056/NEJMoa023153",
};

/// Wells DVT inputs: ten clinician-asserted boolean criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WellsDvtInput {
    /// Active cancer: treatment ongoing or within the last 6 months, or palliative.
    pub active_cancer: bool,
    /// Paralysis, paresis, or recent plaster immobilisation of the lower limb.
    pub paralysis_paresis_immobilisation: bool,
    /// Recently bedridden >=3 days, or major surgery within 12 weeks under anaesthesia.
    pub bedridden_or_major_surgery: bool,
    /// Localised tenderness along the distribution of the deep venous system.
    pub localised_tenderness: bool,
    /// Entire leg swollen.
    pub entire_leg_swollen: bool,
    /// Calf swelling >3 cm vs the asymptomatic leg, measured 10 cm below the tibial tuberosity.
    pub calf_swelling_over_3cm: bool,
    /// Pitting oedema confined to the symptomatic leg.
    pub pitting_oedema: bool,
    /// Collateral superficial (non-varicose) veins.
    pub collateral_superficial_veins: bool,
    /// Previously documented DVT.
    pub previously_documented_dvt: bool,
    /// Alternative diagnosis at least as likely as DVT (subtracts 2 points).
    pub alternative_diagnosis_as_likely: bool,
}

/// Two-level probability band (NICE NG158).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probability {
    /// DVT unlikely: score 1 or less.
    Unlikely,
    /// DVT likely: score 2 or more.
    Likely,
}

impl Probability {
    fn slug(self) -> &'static str {
        match self {
            Probability::Unlikely => "unlikely",
            Probability::Likely => "likely",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WellsDvtOutcome {
    /// Total score (-2 to 9).
    pub score: i8,
    pub probability: Probability,
    pub interpretation: String,
}

/// Pure scoring. The total runs from -2 (alternative diagnosis only) to 9.
pub fn compute(input: &WellsDvtInput) -> Result<WellsDvtOutcome, CalcError> {
    let positives = i8::from(input.active_cancer)
        + i8::from(input.paralysis_paresis_immobilisation)
        + i8::from(input.bedridden_or_major_surgery)
        + i8::from(input.localised_tenderness)
        + i8::from(input.entire_leg_swollen)
        + i8::from(input.calf_swelling_over_3cm)
        + i8::from(input.pitting_oedema)
        + i8::from(input.collateral_superficial_veins)
        + i8::from(input.previously_documented_dvt);

    let alternative = 2 * i8::from(input.alternative_diagnosis_as_likely);
    let score = positives - alternative;

    // NICE NG158 two-level cut-off: likely at 2+, unlikely at 1 or less.
    let probability = if score >= 2 {
        Probability::Likely
    } else {
        Probability::Unlikely
    };

    let interpretation = match probability {
        Probability::Likely => format!(
            "Score {score}: DVT likely (NICE NG158, score 2 or more). Offer a proximal leg vein \
ultrasound scan, with the result available within 4 hours if possible; if it cannot be, offer a \
D-dimer test and interim anticoagulation while awaiting the scan."
        ),
        Probability::Unlikely => format!(
            "Score {score}: DVT unlikely (NICE NG158, score 1 or less). Offer a D-dimer test with \
the result available within 4 hours if possible; a negative D-dimer makes DVT unlikely and rules \
it out without imaging."
        ),
    };

    Ok(WellsDvtOutcome {
        score,
        probability,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &WellsDvtInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("active_cancer".into(), json!(i8::from(input.active_cancer)));
    working.insert(
        "paralysis_paresis_immobilisation".into(),
        json!(i8::from(input.paralysis_paresis_immobilisation)),
    );
    working.insert(
        "bedridden_or_major_surgery".into(),
        json!(i8::from(input.bedridden_or_major_surgery)),
    );
    working.insert(
        "localised_tenderness".into(),
        json!(i8::from(input.localised_tenderness)),
    );
    working.insert(
        "entire_leg_swollen".into(),
        json!(i8::from(input.entire_leg_swollen)),
    );
    working.insert(
        "calf_swelling_over_3cm".into(),
        json!(i8::from(input.calf_swelling_over_3cm)),
    );
    working.insert("pitting_oedema".into(), json!(i8::from(input.pitting_oedema)));
    working.insert(
        "collateral_superficial_veins".into(),
        json!(i8::from(input.collateral_superficial_veins)),
    );
    working.insert(
        "previously_documented_dvt".into(),
        json!(i8::from(input.previously_documented_dvt)),
    );
    working.insert(
        "alternative_diagnosis_as_likely".into(),
        json!(-2 * i8::from(input.alternative_diagnosis_as_likely)),
    );
    working.insert("probability".into(), json!(o.probability.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct WellsDvt;

impl Calculator for WellsDvt {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Wells Score (DVT)"
    }

    fn description(&self) -> &'static str {
        "Clinical pre-test probability of deep vein thrombosis, guiding ultrasound vs D-dimer (NICE NG158)."
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
            "title": "WellsDvtInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "active_cancer", "paralysis_paresis_immobilisation", "bedridden_or_major_surgery",
                "localised_tenderness", "entire_leg_swollen", "calf_swelling_over_3cm",
                "pitting_oedema", "collateral_superficial_veins", "previously_documented_dvt",
                "alternative_diagnosis_as_likely"
            ],
            "properties": {
                "active_cancer": {
                    "type": "boolean",
                    "description": "Active cancer: treatment ongoing or within 6 months, or palliative (+1)",
                    "definition": {
                        "concept": "Active cancer",
                        "statement": "Cancer with treatment ongoing, given within the previous 6 months, or palliative.",
                        "includes": ["On chemotherapy, radiotherapy, or hormonal treatment now", "Treatment within the last 6 months", "Palliative-stage malignancy"],
                        "excludes": ["A cancer in long-term remission with no treatment in the past 6 months"],
                        "snomedEcl": "<< 363346000 |Malignant neoplastic disease (disorder)|",
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "paralysis_paresis_immobilisation": {
                    "type": "boolean",
                    "description": "Paralysis, paresis, or recent plaster immobilisation of the leg (+1)",
                    "definition": {
                        "concept": "Paralysis, paresis, or recent immobilisation",
                        "statement": "Paralysis, paresis, or recent plaster cast immobilisation of the lower extremities.",
                        "includes": ["Lower-limb plaster cast or immobilising splint", "Paralysis or paresis of the leg"],
                        "excludes": ["Immobilisation not involving the lower limb"],
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "bedridden_or_major_surgery": {
                    "type": "boolean",
                    "description": "Recently bedridden >=3 days, or major surgery within 12 weeks under anaesthesia (+1)",
                    "definition": {
                        "concept": "Recent immobility or major surgery",
                        "statement": "Recently bedridden for 3 days or more, OR major surgery within the previous 12 weeks requiring general or regional anaesthesia.",
                        "includes": ["Bedridden for at least 3 consecutive days", "Major surgery in the last 12 weeks under general or regional anaesthesia"],
                        "excludes": ["Minor day-case procedures under local anaesthesia", "Reduced mobility short of being bedridden"],
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "localised_tenderness": {
                    "type": "boolean",
                    "description": "Localised tenderness along the distribution of the deep venous system (+1)",
                    "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" }
                },
                "entire_leg_swollen": {
                    "type": "boolean",
                    "description": "Entire leg swollen (+1)"
                },
                "calf_swelling_over_3cm": {
                    "type": "boolean",
                    "description": "Calf swelling >3 cm vs the asymptomatic leg, measured 10 cm below the tibial tuberosity (+1)",
                    "definition": {
                        "concept": "Calf circumference asymmetry",
                        "statement": "Calf circumference more than 3 cm greater than the asymptomatic leg, measured 10 cm below the tibial tuberosity.",
                        "includes": ["A difference strictly greater than 3 cm at the standard landmark"],
                        "excludes": ["A difference of 3 cm or less", "Comparison made at a different level than 10 cm below the tibial tuberosity"],
                        "caveats": "Both calves are measured at the same fixed landmark (10 cm below the tibial tuberosity); the threshold is strictly greater than 3 cm.",
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "pitting_oedema": {
                    "type": "boolean",
                    "description": "Pitting oedema confined to the symptomatic leg (+1)",
                    "definition": {
                        "concept": "Pitting oedema confined to the symptomatic leg",
                        "statement": "Pitting oedema present only in the symptomatic leg.",
                        "excludes": ["Bilateral oedema (suggests a systemic cause rather than unilateral DVT)"],
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "collateral_superficial_veins": {
                    "type": "boolean",
                    "description": "Collateral superficial veins, non-varicose (+1)",
                    "definition": {
                        "concept": "Collateral superficial veins",
                        "statement": "Collateral (non-varicose) superficial veins.",
                        "excludes": ["Pre-existing varicose veins"],
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "previously_documented_dvt": {
                    "type": "boolean",
                    "description": "Previously documented DVT (+1)",
                    "definition": {
                        "concept": "Previously documented DVT",
                        "statement": "A previous, objectively confirmed deep vein thrombosis.",
                        "includes": ["DVT previously confirmed on imaging"],
                        "excludes": ["A clinically suspected but never objectively confirmed prior DVT"],
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                },
                "alternative_diagnosis_as_likely": {
                    "type": "boolean",
                    "description": "Alternative diagnosis at least as likely as DVT - SUBTRACTS 2 points",
                    "definition": {
                        "concept": "Alternative diagnosis at least as likely as DVT",
                        "statement": "A diagnosis other than DVT (e.g. cellulitis, ruptured Baker's cyst, calf haematoma, superficial thrombophlebitis) is judged at least as likely as DVT.",
                        "caveats": "This is the only item that subtracts (-2). It is a clinical gestalt judgement, not a single examination finding, and it makes the lowest possible total -2.",
                        "source": { "citation": "Wells PS et al. N Engl J Med. 2003;349(13):1227-1235.", "url": "https://doi.org/10.1056/NEJMoa023153" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WellsDvtInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> WellsDvtInput {
        WellsDvtInput {
            active_cancer: false,
            paralysis_paresis_immobilisation: false,
            bedridden_or_major_surgery: false,
            localised_tenderness: false,
            entire_leg_swollen: false,
            calf_swelling_over_3cm: false,
            pitting_oedema: false,
            collateral_superficial_veins: false,
            previously_documented_dvt: false,
            alternative_diagnosis_as_likely: false,
        }
    }

    fn all_nine_positive() -> WellsDvtInput {
        WellsDvtInput {
            active_cancer: true,
            paralysis_paresis_immobilisation: true,
            bedridden_or_major_surgery: true,
            localised_tenderness: true,
            entire_leg_swollen: true,
            calf_swelling_over_3cm: true,
            pitting_oedema: true,
            collateral_superficial_veins: true,
            previously_documented_dvt: true,
            alternative_diagnosis_as_likely: false,
        }
    }

    #[test]
    fn no_criteria_scores_zero_and_unlikely() {
        let o = compute(&none()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.probability, Probability::Unlikely);
    }

    #[test]
    fn maximum_score_is_nine() {
        let o = compute(&all_nine_positive()).unwrap();
        assert_eq!(o.score, 9);
        assert_eq!(o.probability, Probability::Likely);
    }

    #[test]
    fn minimum_score_is_minus_two() {
        let mut i = none();
        i.alternative_diagnosis_as_likely = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, -2);
        assert_eq!(o.probability, Probability::Unlikely);
    }

    #[test]
    fn alternative_diagnosis_subtracts_two() {
        // Three +1 items = 3, minus 2 for alternative diagnosis = 1.
        let mut i = none();
        i.active_cancer = true;
        i.localised_tenderness = true;
        i.entire_leg_swollen = true;
        i.alternative_diagnosis_as_likely = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.probability, Probability::Unlikely);
    }

    #[test]
    fn two_level_threshold_at_two() {
        // Score of 1 is unlikely; score of 2 is likely (NICE NG158).
        let mut one = none();
        one.active_cancer = true;
        assert_eq!(compute(&one).unwrap().score, 1);
        assert_eq!(compute(&one).unwrap().probability, Probability::Unlikely);

        let mut two = none();
        two.active_cancer = true;
        two.localised_tenderness = true;
        assert_eq!(compute(&two).unwrap().score, 2);
        assert_eq!(compute(&two).unwrap().probability, Probability::Likely);
    }

    #[test]
    fn likely_interpretation_mentions_ultrasound() {
        let o = compute(&all_nine_positive()).unwrap();
        assert!(o.interpretation.contains("ultrasound"));
        assert!(o.interpretation.contains("likely"));
    }

    #[test]
    fn unlikely_interpretation_mentions_d_dimer() {
        let o = compute(&none()).unwrap();
        assert!(o.interpretation.contains("D-dimer"));
    }

    #[test]
    fn worked_example_cellulitis_offset() {
        // Tender, swollen calf >3 cm, but cellulitis judged as likely: 2 - 2 = 0.
        let mut i = none();
        i.localised_tenderness = true;
        i.calf_swelling_over_3cm = true;
        i.alternative_diagnosis_as_likely = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.probability, Probability::Unlikely);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "active_cancer": true,
            "paralysis_paresis_immobilisation": false,
            "bedridden_or_major_surgery": false,
            "localised_tenderness": true,
            "entire_leg_swollen": false,
            "calf_swelling_over_3cm": true,
            "pitting_oedema": false,
            "collateral_superficial_veins": false,
            "previously_documented_dvt": false,
            "alternative_diagnosis_as_likely": false
        });
        let mut typed = none();
        typed.active_cancer = true;
        typed.localised_tenderness = true;
        typed.calf_swelling_over_3cm = true;
        let dynamic = WellsDvt.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn working_records_negative_alternative_contribution() {
        let mut i = none();
        i.alternative_diagnosis_as_likely = true;
        let r = build_response(&i).unwrap();
        assert_eq!(r.working["alternative_diagnosis_as_likely"], json!(-2));
        assert_eq!(r.working["total_score"], json!(-2));
        assert_eq!(r.working["probability"], json!("unlikely"));
    }

    #[test]
    fn calf_definition_keeps_landmark_and_threshold() {
        let schema = WellsDvt.input_schema();
        let def = &schema["properties"]["calf_swelling_over_3cm"]["definition"];
        assert!(def["statement"].as_str().unwrap().contains("10 cm below the tibial tuberosity"));
        assert!(def["caveats"].as_str().unwrap().contains("strictly greater than 3 cm"));
    }

    #[test]
    fn rejects_missing_field() {
        let value = json!({ "active_cancer": true });
        assert!(WellsDvt.calculate(&value).is_err());
    }
}
