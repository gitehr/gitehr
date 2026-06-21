//! CHA2DS2-VASc - stroke risk in non-valvular atrial fibrillation.
//!
//! Guides anticoagulation decisions (NICE NG196). The score is the flagship for
//! the input-definition system (`spec/calculator-input-definitions.md`): several
//! criteria are clinician-asserted predicates whose TRUE/FALSE conditions are
//! easy to get subtly wrong, most notably "vascular disease", which is arterial
//! disease and explicitly excludes venous thromboembolism.
//!
//! Two clinical subtleties are encoded here:
//! - Age is a single input mapping to the two mutually-exclusive bands (65-74 = 1
//!   point, >=75 = 2 points), so contradictory age inputs are impossible.
//! - Female sex contributes a point, but as an age-dependent risk *modifier*: a
//!   score of 1 arising from sex alone is low risk and does not by itself warrant
//!   anticoagulation (NICE NG196).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "cha2ds2vasc";

/// Primary citation.
pub const REFERENCE: &str =
    "Lip GYH, Nieuwlaat R, Pisters R, et al. Refining clinical risk stratification for predicting \
stroke and thromboembolism in atrial fibrillation using a novel risk factor-based approach: the \
Euro Heart Survey on Atrial Fibrillation. Chest. 2010;137(2):263-272. Thresholds per NICE NG196.";

/// Distribution licence: the score is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1378/chest.09-1584",
};

/// Sex, which both contributes a point (female) and modifies interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// CHA2DS2-VASc inputs. Age is numeric; the two age bands are derived.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cha2ds2VascInput {
    /// Age in years.
    pub age: u8,
    pub sex: Sex,
    /// Congestive heart failure / moderate-to-severe LV systolic dysfunction (C).
    pub congestive_heart_failure: bool,
    /// Hypertension (H).
    pub hypertension: bool,
    /// Diabetes mellitus (D).
    pub diabetes: bool,
    /// Prior stroke, TIA, or systemic (arterial) thromboembolism (S2, 2 points).
    pub stroke_tia_thromboembolism: bool,
    /// Vascular disease: prior MI, peripheral arterial disease, or aortic plaque (V).
    pub vascular_disease: bool,
}

/// Anticoagulation recommendation band (NICE NG196).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    /// Low risk: anticoagulation not recommended.
    NotRecommended,
    /// Consider anticoagulation (men with a score of 1).
    Consider,
    /// Offer anticoagulation (score 2 or above), weighing bleeding risk.
    Offer,
}

impl Recommendation {
    fn slug(self) -> &'static str {
        match self {
            Recommendation::NotRecommended => "not-recommended",
            Recommendation::Consider => "consider",
            Recommendation::Offer => "offer",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cha2ds2VascOutcome {
    /// Total score (0-9).
    pub score: u8,
    /// Points contributed by the age band (0, 1, or 2).
    pub age_points: u8,
    /// Score excluding the sex point - used to identify "sex-only" low risk.
    pub non_sex_score: u8,
    pub recommendation: Recommendation,
    pub interpretation: String,
}

fn age_points(age: u8) -> u8 {
    if age >= 75 {
        2
    } else if age >= 65 {
        1
    } else {
        0
    }
}

/// Pure scoring.
pub fn compute(input: &Cha2ds2VascInput) -> Result<Cha2ds2VascOutcome, CalcError> {
    let age_points = age_points(input.age);
    let sex_point = u8::from(input.sex == Sex::Female);

    let non_sex_score = age_points
        + u8::from(input.congestive_heart_failure)
        + u8::from(input.hypertension)
        + u8::from(input.diabetes)
        + 2 * u8::from(input.stroke_tia_thromboembolism)
        + u8::from(input.vascular_disease);

    let score = non_sex_score + sex_point;

    // NICE NG196: do not anticoagulate for a score arising only from sex (score 0
    // in men, 1 in women); consider for men with a score of 1; offer at 2+.
    let recommendation = if non_sex_score == 0 {
        Recommendation::NotRecommended
    } else if score == 1 {
        Recommendation::Consider
    } else {
        Recommendation::Offer
    };

    let interpretation = match recommendation {
        Recommendation::NotRecommended => {
            if input.sex == Sex::Female && score == 1 {
                "Score 1 from female sex alone. Female sex is an age-dependent risk modifier, not an \
independent indication: this is low risk and anticoagulation is not recommended (NICE NG196)."
                    .to_string()
            } else {
                "Score 0: low risk. Anticoagulation is not recommended (NICE NG196).".to_string()
            }
        }
        Recommendation::Consider => format!(
            "Score {score}: consider anticoagulation, weighing bleeding risk (e.g. ORBIT or \
HAS-BLED) and patient preference (NICE NG196)."
        ),
        Recommendation::Offer => format!(
            "Score {score}: offer anticoagulation, taking bleeding risk into account (NICE NG196). \
Stroke risk rises with the score."
        ),
    };

    Ok(Cha2ds2VascOutcome {
        score,
        age_points,
        non_sex_score,
        recommendation,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Cha2ds2VascInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("age_points".into(), json!(o.age_points));
    working.insert("congestive_heart_failure".into(), json!(u8::from(input.congestive_heart_failure)));
    working.insert("hypertension".into(), json!(u8::from(input.hypertension)));
    working.insert("diabetes".into(), json!(u8::from(input.diabetes)));
    working.insert("stroke_tia_thromboembolism".into(), json!(2 * u8::from(input.stroke_tia_thromboembolism)));
    working.insert("vascular_disease".into(), json!(u8::from(input.vascular_disease)));
    working.insert("sex_point".into(), json!(u8::from(input.sex == Sex::Female)));
    working.insert("recommendation".into(), json!(o.recommendation.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Cha2ds2Vasc;

impl Calculator for Cha2ds2Vasc {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "CHA2DS2-VASc Stroke Risk (AF)"
    }

    fn description(&self) -> &'static str {
        "Stroke risk in non-valvular atrial fibrillation, guiding anticoagulation (NICE NG196)."
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
            "title": "Cha2ds2VascInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "sex", "congestive_heart_failure", "hypertension",
                "diabetes", "stroke_tia_thromboembolism", "vascular_disease"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (65-74 scores 1, 75+ scores 2)"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex; female scores 1, as an age-dependent risk modifier",
                    "definition": {
                        "concept": "Sex category (Sc)",
                        "statement": "Female sex scores 1 point.",
                        "caveats": "Female sex is a risk modifier, not an independent indication: a score of 1 from sex alone is low risk. Recent schemes (CHA2DS2-VA) drop the sex criterion.",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                },
                "congestive_heart_failure": {
                    "type": "boolean",
                    "description": "Heart failure or moderate-to-severe LV systolic dysfunction (C)",
                    "definition": {
                        "concept": "Congestive heart failure / LV dysfunction (C)",
                        "statement": "Signs/symptoms of heart failure, or objective moderate-to-severe LV systolic dysfunction, or recent decompensated heart failure.",
                        "includes": ["HFrEF and HFpEF", "Recent decompensation requiring hospitalisation", "Moderate-to-severe LV systolic dysfunction on imaging"],
                        "excludes": ["An isolated historical mention with no current evidence or objective dysfunction"],
                        "snomedEcl": "<< 42343007 |Congestive heart failure (disorder)|",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                },
                "hypertension": {
                    "type": "boolean",
                    "description": "Hypertension - diagnosed, treated, or BP consistently >140/90 (H)",
                    "definition": {
                        "concept": "Hypertension (H)",
                        "statement": "A diagnosis of hypertension, treatment with antihypertensives, or resting BP consistently above 140/90 mmHg.",
                        "includes": ["On antihypertensive treatment", "Diagnosed hypertension", "Repeated resting BP >140/90 mmHg"],
                        "excludes": ["A single isolated elevated reading without diagnosis or treatment"],
                        "snomedEcl": "<< 38341003 |Hypertensive disorder, systemic arterial (disorder)|",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                },
                "diabetes": {
                    "type": "boolean",
                    "description": "Diabetes mellitus, type 1 or type 2 (D)",
                    "definition": {
                        "concept": "Diabetes mellitus (D)",
                        "statement": "Established type 1 or type 2 diabetes mellitus.",
                        "includes": ["Type 1 diabetes", "Type 2 diabetes", "On glucose-lowering treatment"],
                        "excludes": ["Pre-diabetes / impaired glucose tolerance does NOT count"],
                        "snomedEcl": "<< 73211009 |Diabetes mellitus (disorder)| MINUS << 714628002 |Prediabetes (finding)|",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                },
                "stroke_tia_thromboembolism": {
                    "type": "boolean",
                    "description": "Prior stroke, TIA, or systemic ARTERIAL thromboembolism (S2, 2 points)",
                    "definition": {
                        "concept": "Stroke / TIA / thromboembolism (S2)",
                        "statement": "Prior ischaemic stroke, transient ischaemic attack, or systemic arterial thromboembolism.",
                        "includes": ["Prior ischaemic stroke", "Transient ischaemic attack (TIA)", "Systemic arterial thromboembolism"],
                        "excludes": ["Venous thromboembolism (DVT or PE) does NOT count - this criterion is arterial"],
                        "snomedEcl": "(<< 230690007 |Cerebrovascular accident (disorder)| OR << 266257000 |Transient ischemic attack (disorder)|) MINUS << 118927008 |Disorder of venous system (disorder)|",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                },
                "vascular_disease": {
                    "type": "boolean",
                    "description": "Vascular disease: prior MI, peripheral arterial disease, or aortic plaque (V)",
                    "definition": {
                        "concept": "Vascular disease (V)",
                        "statement": "Established ARTERIAL vascular disease.",
                        "includes": ["Prior myocardial infarction", "Peripheral arterial disease", "Complex aortic plaque"],
                        "excludes": [
                            "Venous thromboembolism (DVT or PE) does NOT count - this criterion is arterial",
                            "Isolated stable coronary artery disease without prior MI is disputed; see caveats"
                        ],
                        "caveats": "Guidelines differ on whether stable CAD without MI qualifies. Aortic plaque means complex/atheromatous plaque, not incidental calcification.",
                        "snomedEcl": "(<< 22298006 |Myocardial infarction (disorder)| OR << 400047006 |Peripheral vascular disease (disorder)|) MINUS << 118927008 |Disorder of venous system (disorder)|",
                        "source": { "citation": "Lip GYH et al. Chest. 2010;137(2):263-272.", "url": "https://doi.org/10.1378/chest.09-1584" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Cha2ds2VascInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(age: u8, sex: Sex) -> Cha2ds2VascInput {
        Cha2ds2VascInput {
            age,
            sex,
            congestive_heart_failure: false,
            hypertension: false,
            diabetes: false,
            stroke_tia_thromboembolism: false,
            vascular_disease: false,
        }
    }

    #[test]
    fn age_bands() {
        assert_eq!(age_points(64), 0);
        assert_eq!(age_points(65), 1);
        assert_eq!(age_points(74), 1);
        assert_eq!(age_points(75), 2);
    }

    #[test]
    fn worked_example_female_htn_dm() {
        // 70F, HTN, DM: age 1 + female 1 + htn 1 + dm 1 = 4.
        let mut i = base(70, Sex::Female);
        i.hypertension = true;
        i.diabetes = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.recommendation, Recommendation::Offer);
    }

    #[test]
    fn stroke_scores_two() {
        // 80M, prior stroke: age 2 + stroke 2 = 4.
        let mut i = base(80, Sex::Male);
        i.stroke_tia_thromboembolism = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.recommendation, Recommendation::Offer);
    }

    #[test]
    fn female_sex_only_is_low_risk() {
        // 60F, no factors: score 1 from sex alone -> not recommended.
        let o = compute(&base(60, Sex::Female)).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.non_sex_score, 0);
        assert_eq!(o.recommendation, Recommendation::NotRecommended);
        assert!(o.interpretation.contains("female sex alone"));
    }

    #[test]
    fn male_zero_is_low_risk() {
        let o = compute(&base(60, Sex::Male)).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.recommendation, Recommendation::NotRecommended);
    }

    #[test]
    fn male_score_one_is_consider() {
        // 70M, no other factors: age 65-74 = 1 -> consider.
        let o = compute(&base(70, Sex::Male)).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.recommendation, Recommendation::Consider);
    }

    #[test]
    fn maximum_score_is_nine() {
        let i = Cha2ds2VascInput {
            age: 80,
            sex: Sex::Female,
            congestive_heart_failure: true,
            hypertension: true,
            diabetes: true,
            stroke_tia_thromboembolism: true,
            vascular_disease: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 9);
    }

    #[test]
    fn vascular_disease_contributes_one() {
        let mut i = base(60, Sex::Male);
        i.vascular_disease = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.recommendation, Recommendation::Consider);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 70, "sex": "female", "congestive_heart_failure": false,
            "hypertension": true, "diabetes": true,
            "stroke_tia_thromboembolism": false, "vascular_disease": false
        });
        let mut typed = base(70, Sex::Female);
        typed.hypertension = true;
        typed.diabetes = true;
        let dynamic = Cha2ds2Vasc.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(4));
    }

    #[test]
    fn vascular_definition_excludes_vte() {
        let schema = Cha2ds2Vasc.input_schema();
        let excludes = &schema["properties"]["vascular_disease"]["definition"]["excludes"];
        assert!(excludes[0].as_str().unwrap().contains("Venous thromboembolism"));
        let ecl = schema["properties"]["vascular_disease"]["definition"]["snomedEcl"]
            .as_str()
            .unwrap();
        assert!(ecl.contains("MINUS"), "vascular ECL must exclude the venous hierarchy");
    }
}
