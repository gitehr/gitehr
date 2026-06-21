//! HAS-BLED - bleeding risk in atrial fibrillation on anticoagulation.
//!
//! The bleeding-risk counterpart to CHA2DS2-VASc: the two are weighed together
//! when deciding whether to anticoagulate (NICE NG196). HAS-BLED does not by
//! itself contraindicate anticoagulation - a high score flags modifiable
//! bleeding risk factors to address and signals the need for closer review.
//!
//! Like CHA2DS2-VASc, this is a showcase for the input-definition system
//! (`spec/calculator-input-definitions.md`): several criteria are
//! clinician-asserted predicates with thresholds that are easy to get subtly
//! wrong, and two of the acronym's letters cover *two* independent points each:
//! - "A" is abnormal renal function AND/OR abnormal liver function (up to 2).
//! - "D" is drugs (antiplatelets/NSAIDs) AND/OR harmful alcohol use (up to 2).
//!
//! The hypertension criterion is deliberately stricter than the CHA2DS2-VASc
//! one: here it means *uncontrolled* hypertension with systolic BP >160 mmHg,
//! not merely a diagnosis of or treatment for hypertension.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "hasbled";

/// Primary citation.
pub const REFERENCE: &str =
    "Pisters R, Lane DA, Nieuwlaat R, de Vos CB, Crijns HJM, Lip GYH. A novel user-friendly score \
(HAS-BLED) to assess 1-year risk of major bleeding in patients with atrial fibrillation: the Euro \
Heart Survey. Chest. 2010;138(5):1093-1100. Used alongside CHA2DS2-VASc per NICE NG196.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1378/chest.10-0134",
};

/// Score at and above which bleeding risk is "high" and warrants caution and
/// regular review of modifiable risk factors.
pub const HIGH_RISK_CUTOFF: u8 = 3;

/// HAS-BLED inputs. Every criterion is a clinician-asserted boolean, scoring 1
/// point each; "A" and "D" each split into two independently-scored criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HasBledInput {
    /// Hypertension (H): UNCONTROLLED, systolic BP >160 mmHg.
    pub hypertension_uncontrolled: bool,
    /// Abnormal renal function (A): dialysis, transplant, or creatinine >=200 umol/L.
    pub abnormal_renal_function: bool,
    /// Abnormal liver function (A): cirrhosis, or bilirubin >2x ULN with AST/ALT/ALP >3x ULN.
    pub abnormal_liver_function: bool,
    /// Prior stroke (S).
    pub stroke: bool,
    /// Bleeding history or predisposition (B): prior major bleed, bleeding diathesis, anaemia.
    pub bleeding_history: bool,
    /// Labile INRs (L): unstable/high INRs, or time in therapeutic range <60%.
    pub labile_inr: bool,
    /// Elderly (E): age >65 years.
    pub elderly_over_65: bool,
    /// Drugs (D): concomitant antiplatelet agents or NSAIDs.
    pub drugs_antiplatelet_nsaid: bool,
    /// Alcohol (D): harmful use, >=8 units (drinks) per week.
    pub alcohol_excess: bool,
}

/// Bleeding-risk band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    /// Score 0-2: low/moderate bleeding risk.
    Low,
    /// Score >=3: high bleeding risk - caution and regular review.
    High,
}

impl Risk {
    fn slug(self) -> &'static str {
        match self {
            Risk::Low => "low",
            Risk::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasBledOutcome {
    /// Total score (0-9).
    pub score: u8,
    pub risk: Risk,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &HasBledInput) -> Result<HasBledOutcome, CalcError> {
    let score = u8::from(input.hypertension_uncontrolled)
        + u8::from(input.abnormal_renal_function)
        + u8::from(input.abnormal_liver_function)
        + u8::from(input.stroke)
        + u8::from(input.bleeding_history)
        + u8::from(input.labile_inr)
        + u8::from(input.elderly_over_65)
        + u8::from(input.drugs_antiplatelet_nsaid)
        + u8::from(input.alcohol_excess);

    let risk = if score >= HIGH_RISK_CUTOFF {
        Risk::High
    } else {
        Risk::Low
    };

    let interpretation = match risk {
        Risk::Low => format!(
            "Score {score}: low bleeding risk. A HAS-BLED score below 3 does not preclude \
anticoagulation; weigh against stroke risk (CHA2DS2-VASc) per NICE NG196."
        ),
        Risk::High => format!(
            "Score {score}: HIGH bleeding risk (>=3). This does not by itself contraindicate \
anticoagulation - it flags the need for caution, correction of modifiable bleeding risk factors \
(e.g. uncontrolled hypertension, labile INR, concomitant antiplatelets/NSAIDs, harmful alcohol \
use), and more regular review, weighed against stroke risk (CHA2DS2-VASc) per NICE NG196."
        ),
    };

    Ok(HasBledOutcome {
        score,
        risk,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &HasBledInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert(
        "hypertension_uncontrolled".into(),
        json!(u8::from(input.hypertension_uncontrolled)),
    );
    working.insert(
        "abnormal_renal_function".into(),
        json!(u8::from(input.abnormal_renal_function)),
    );
    working.insert(
        "abnormal_liver_function".into(),
        json!(u8::from(input.abnormal_liver_function)),
    );
    working.insert("stroke".into(), json!(u8::from(input.stroke)));
    working.insert(
        "bleeding_history".into(),
        json!(u8::from(input.bleeding_history)),
    );
    working.insert("labile_inr".into(), json!(u8::from(input.labile_inr)));
    working.insert(
        "elderly_over_65".into(),
        json!(u8::from(input.elderly_over_65)),
    );
    working.insert(
        "drugs_antiplatelet_nsaid".into(),
        json!(u8::from(input.drugs_antiplatelet_nsaid)),
    );
    working.insert(
        "alcohol_excess".into(),
        json!(u8::from(input.alcohol_excess)),
    );
    working.insert("risk".into(), json!(o.risk.slug()));
    working.insert("high_risk_cutoff".into(), json!(HIGH_RISK_CUTOFF));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct HasBled;

impl Calculator for HasBled {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "HAS-BLED Bleeding Risk (AF)"
    }

    fn description(&self) -> &'static str {
        "Bleeding risk in atrial fibrillation on anticoagulation, used alongside CHA2DS2-VASc (NICE NG196)."
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
            "title": "HasBledInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "hypertension_uncontrolled", "abnormal_renal_function",
                "abnormal_liver_function", "stroke", "bleeding_history",
                "labile_inr", "elderly_over_65", "drugs_antiplatelet_nsaid",
                "alcohol_excess"
            ],
            "properties": {
                "hypertension_uncontrolled": {
                    "type": "boolean",
                    "description": "Uncontrolled hypertension, systolic BP >160 mmHg (H)",
                    "definition": {
                        "concept": "Uncontrolled hypertension (H)",
                        "statement": "Uncontrolled hypertension, defined as systolic blood pressure greater than 160 mmHg.",
                        "includes": ["Systolic BP >160 mmHg (uncontrolled)"],
                        "excludes": [
                            "This is NOT the same as the CHA2DS2-VASc hypertension criterion: controlled or merely treated/diagnosed hypertension with systolic BP <=160 mmHg does NOT score here",
                            "Elevated diastolic alone does not define this criterion; the threshold is systolic >160 mmHg"
                        ],
                        "caveats": "HAS-BLED hypertension is specifically UNCONTROLLED (systolic >160 mmHg). Treating it to target removes the point and is a key modifiable risk factor.",
                        "snomedEcl": "<< 38341003 |Hypertensive disorder, systemic arterial (disorder)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "abnormal_renal_function": {
                    "type": "boolean",
                    "description": "Abnormal renal function: dialysis, transplant, or creatinine >=200 umol/L (A, 1 of up to 2)",
                    "definition": {
                        "concept": "Abnormal renal function (A)",
                        "statement": "Chronic dialysis, renal transplantation, or a serum creatinine of at least 200 umol/L (>=2.26 mg/dL).",
                        "includes": ["Chronic dialysis", "Renal transplant", "Serum creatinine >=200 umol/L (>=2.26 mg/dL)"],
                        "excludes": [
                            "Creatinine below 200 umol/L without dialysis or transplant does NOT score",
                            "Use the umol/L threshold (>=200), not an eGFR cut-off"
                        ],
                        "caveats": "Abnormal renal function and abnormal liver function are SEPARATE criteria under the single 'A' of the acronym; together they can contribute up to 2 points.",
                        "snomedEcl": "<< 90688005 |Chronic renal failure syndrome (disorder)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "abnormal_liver_function": {
                    "type": "boolean",
                    "description": "Abnormal liver function: cirrhosis, or bilirubin >2x ULN with AST/ALT/ALP >3x ULN (A, 1 of up to 2)",
                    "definition": {
                        "concept": "Abnormal liver function (A)",
                        "statement": "Chronic hepatic disease (e.g. cirrhosis) or biochemical evidence of significant hepatic derangement: bilirubin greater than 2x the upper limit of normal in association with AST/ALT/alkaline phosphatase greater than 3x the upper limit of normal.",
                        "includes": ["Cirrhosis", "Bilirubin >2x ULN AND AST/ALT/ALP >3x ULN"],
                        "excludes": [
                            "Isolated mildly deranged liver enzymes below these multiples do NOT score",
                            "Bilirubin >2x ULN alone, without the transaminase/ALP derangement, does NOT score (and vice versa) unless cirrhosis is present"
                        ],
                        "caveats": "Abnormal liver function and abnormal renal function are SEPARATE criteria under the single 'A' of the acronym; together they can contribute up to 2 points.",
                        "snomedEcl": "<< 235856003 |Disorder of liver (disorder)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "stroke": {
                    "type": "boolean",
                    "description": "Prior stroke (S)",
                    "definition": {
                        "concept": "Stroke history (S)",
                        "statement": "A history of stroke, particularly prior ischaemic stroke (and haemorrhagic stroke, which especially raises bleeding risk).",
                        "includes": ["Prior ischaemic stroke", "Prior haemorrhagic stroke"],
                        "excludes": ["TIA without infarction is generally not counted for this bleeding criterion (unlike the CHA2DS2-VASc stroke criterion, which includes TIA)"],
                        "snomedEcl": "<< 230690007 |Cerebrovascular accident (disorder)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "bleeding_history": {
                    "type": "boolean",
                    "description": "Bleeding history or predisposition: prior major bleed, bleeding diathesis, anaemia (B)",
                    "definition": {
                        "concept": "Bleeding history or predisposition (B)",
                        "statement": "A history of major bleeding, a bleeding tendency or predisposition (e.g. bleeding diathesis), or anaemia.",
                        "includes": ["Prior major bleed", "Bleeding diathesis / predisposition", "Anaemia"],
                        "excludes": ["Trivial bruising or a single minor self-limiting bleed without predisposition"],
                        "snomedEcl": "<< 131148009 |Bleeding (finding)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "labile_inr": {
                    "type": "boolean",
                    "description": "Labile INRs: unstable/high INRs, or time in therapeutic range <60% (L)",
                    "definition": {
                        "concept": "Labile INR (L)",
                        "statement": "Unstable or high INRs, or a low time in therapeutic range (TTR) under 60%, in a patient on a vitamin K antagonist (e.g. warfarin).",
                        "includes": ["Time in therapeutic range <60%", "Frequently unstable or high INRs"],
                        "excludes": ["Not applicable to patients on a DOAC, who have no INR to monitor; score 0 for this criterion in that case"],
                        "caveats": "This criterion is specific to vitamin K antagonist therapy. Poor TTR is a modifiable factor.",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "elderly_over_65": {
                    "type": "boolean",
                    "description": "Elderly: age over 65 years (E)",
                    "definition": {
                        "concept": "Elderly (E)",
                        "statement": "Age greater than 65 years.",
                        "includes": ["Age >65 years (i.e. 66 and over)"],
                        "excludes": ["Note the threshold is strictly >65, distinct from the CHA2DS2-VASc age bands (which begin at 65 and add a second point at 75)"],
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "drugs_antiplatelet_nsaid": {
                    "type": "boolean",
                    "description": "Drugs predisposing to bleeding: antiplatelets or NSAIDs (D, 1 of up to 2)",
                    "definition": {
                        "concept": "Drugs predisposing to bleeding (D)",
                        "statement": "Concomitant use of antiplatelet agents (e.g. aspirin, clopidogrel) or NSAIDs.",
                        "includes": ["Aspirin", "Clopidogrel or other antiplatelet agents", "NSAIDs"],
                        "excludes": ["The anticoagulant itself is not what this criterion captures; it is ADDITIONAL bleeding-predisposing drugs"],
                        "caveats": "Drugs and harmful alcohol use are SEPARATE criteria under the single 'D' of the acronym; together they can contribute up to 2 points. Concomitant antiplatelet/NSAID use is a key modifiable factor.",
                        "snomedEcl": "<< 372587000 |Platelet aggregation inhibitor (substance)| OR << 372665008 |Nonsteroidal anti-inflammatory agent (substance)|",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                },
                "alcohol_excess": {
                    "type": "boolean",
                    "description": "Harmful alcohol use: >=8 units (drinks) per week (D, 1 of up to 2)",
                    "definition": {
                        "concept": "Harmful alcohol use (D)",
                        "statement": "Harmful alcohol consumption, at or above 8 units (drinks) per week.",
                        "includes": ["Alcohol intake >=8 units/drinks per week"],
                        "excludes": ["Occasional or light drinking below 8 units per week does NOT score"],
                        "caveats": "Harmful alcohol use and bleeding-predisposing drugs are SEPARATE criteria under the single 'D' of the acronym; together they can contribute up to 2 points.",
                        "source": { "citation": "Pisters R et al. Chest. 2010;138(5):1093-1100.", "url": "https://doi.org/10.1378/chest.10-0134" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: HasBledInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> HasBledInput {
        HasBledInput {
            hypertension_uncontrolled: false,
            abnormal_renal_function: false,
            abnormal_liver_function: false,
            stroke: false,
            bleeding_history: false,
            labile_inr: false,
            elderly_over_65: false,
            drugs_antiplatelet_nsaid: false,
            alcohol_excess: false,
        }
    }

    #[test]
    fn all_false_is_zero_low() {
        let o = compute(&none()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.risk, Risk::Low);
    }

    #[test]
    fn each_criterion_scores_one() {
        for set in [
            |i: &mut HasBledInput| i.hypertension_uncontrolled = true,
            |i: &mut HasBledInput| i.abnormal_renal_function = true,
            |i: &mut HasBledInput| i.abnormal_liver_function = true,
            |i: &mut HasBledInput| i.stroke = true,
            |i: &mut HasBledInput| i.bleeding_history = true,
            |i: &mut HasBledInput| i.labile_inr = true,
            |i: &mut HasBledInput| i.elderly_over_65 = true,
            |i: &mut HasBledInput| i.drugs_antiplatelet_nsaid = true,
            |i: &mut HasBledInput| i.alcohol_excess = true,
        ] {
            let mut i = none();
            set(&mut i);
            assert_eq!(compute(&i).unwrap().score, 1);
        }
    }

    #[test]
    fn renal_and_liver_each_score_giving_two() {
        // The single "A" letter covers two independent points.
        let mut i = none();
        i.abnormal_renal_function = true;
        i.abnormal_liver_function = true;
        assert_eq!(compute(&i).unwrap().score, 2);
    }

    #[test]
    fn drugs_and_alcohol_each_score_giving_two() {
        // The single "D" letter covers two independent points.
        let mut i = none();
        i.drugs_antiplatelet_nsaid = true;
        i.alcohol_excess = true;
        assert_eq!(compute(&i).unwrap().score, 2);
    }

    #[test]
    fn cutoff_three_is_high_risk() {
        let mut i = none();
        i.hypertension_uncontrolled = true;
        i.stroke = true;
        let two = compute(&i).unwrap();
        assert_eq!(two.score, 2);
        assert_eq!(two.risk, Risk::Low);

        i.bleeding_history = true;
        let three = compute(&i).unwrap();
        assert_eq!(three.score, 3);
        assert_eq!(three.risk, Risk::High);
        assert!(three.interpretation.contains("HIGH"));
    }

    #[test]
    fn maximum_score_is_nine() {
        let i = HasBledInput {
            hypertension_uncontrolled: true,
            abnormal_renal_function: true,
            abnormal_liver_function: true,
            stroke: true,
            bleeding_history: true,
            labile_inr: true,
            elderly_over_65: true,
            drugs_antiplatelet_nsaid: true,
            alcohol_excess: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 9);
        assert_eq!(o.risk, Risk::High);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "hypertension_uncontrolled": true,
            "abnormal_renal_function": false,
            "abnormal_liver_function": false,
            "stroke": true,
            "bleeding_history": true,
            "labile_inr": false,
            "elderly_over_65": false,
            "drugs_antiplatelet_nsaid": false,
            "alcohol_excess": false
        });
        let mut typed = none();
        typed.hypertension_uncontrolled = true;
        typed.stroke = true;
        typed.bleeding_history = true;
        let dynamic = HasBled.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(3));
    }

    #[test]
    fn hypertension_definition_distinguishes_from_cha2ds2vasc() {
        let schema = HasBled.input_schema();
        let excludes = &schema["properties"]["hypertension_uncontrolled"]["definition"]["excludes"];
        assert!(excludes[0]
            .as_str()
            .unwrap()
            .contains("CHA2DS2-VASc hypertension criterion"));
    }

    #[test]
    fn renal_definition_notes_the_threshold_and_paired_point() {
        let schema = HasBled.input_schema();
        let def = &schema["properties"]["abnormal_renal_function"]["definition"];
        assert!(def["statement"].as_str().unwrap().contains("200 umol/L"));
        assert!(def["caveats"].as_str().unwrap().contains("up to 2 points"));
    }

    #[test]
    fn drugs_definition_notes_paired_point_with_alcohol() {
        let schema = HasBled.input_schema();
        let def = &schema["properties"]["drugs_antiplatelet_nsaid"]["definition"];
        assert!(def["caveats"].as_str().unwrap().contains("up to 2 points"));
    }
}
