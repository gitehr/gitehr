//! CHALICE - Children's Head injury ALgorithm for the prediction of Important
//! Clinical Events (paediatric head injury CT decision rule).
//!
//! Unlike the numeric scores elsewhere in this crate, CHALICE is a *decision
//! rule*: it does not add points. If ANY single criterion is positive, a CT
//! head scan is recommended, because the presence of any one criterion predicts
//! a clinically significant intracranial injury. The rule was derived with a
//! sensitivity of 98% for clinically significant head injury (Dunning et al,
//! Arch Dis Child 2006), and is reflected in NICE head-injury guidance
//! (CG176, updated NG232).
//!
//! Two criteria are age-dependent and easy to get subtly wrong, so age is taken
//! as a single numeric input from which an `under 1 year` flag is derived:
//! - GCS: the threshold is < 14, but tightens to < 15 for infants under 1 year.
//! - Bruise/swelling/laceration > 5 cm: this criterion applies ONLY to infants
//!   under 1 year; in older children such a finding is not, by itself, a CHALICE
//!   indication.
//!
//! Criteria are grouped as History, Examination, and Mechanism per the original
//! algorithm, and the response lists exactly which criteria were positive.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "chalice";

/// Primary citation.
pub const REFERENCE: &str =
    "Dunning J, Daly JP, Lomas J-P, Lecky F, Batchelor J, Mackway-Jones K. Derivation of the \
children's head injury algorithm for the prediction of important clinical events decision rule for \
head injury in children. Arch Dis Child. 2006;91(11):885-891. Reflected in NICE head injury \
guidance (CG176 / NG232).";

/// Distribution licence: CHALICE is a published clinical decision rule,
/// implemented here from the primary literature, which is not subject to
/// copyright as an algorithm.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1136/adc.2005.083980",
};

/// CHALICE inputs. Age is numeric; the `under 1 year` band is derived for the
/// two age-dependent criteria (the GCS threshold and the bruise/swelling/
/// laceration criterion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChaliceInput {
    /// Age in completed years (0 means under 1 year old).
    pub age_years: u8,

    // --- History ---
    /// Witnessed loss of consciousness of more than 5 minutes' duration.
    pub loss_of_consciousness_over_5_min: bool,
    /// Amnesia (antegrade or retrograde) of more than 5 minutes' duration.
    pub amnesia_over_5_min: bool,
    /// Abnormal drowsiness (exceeding the level expected by the examining doctor).
    pub abnormal_drowsiness: bool,
    /// Three or more discrete episodes of vomiting after the head injury.
    pub vomiting_3_or_more: bool,
    /// Suspicion of non-accidental injury (NAI).
    pub suspected_nai: bool,
    /// Seizure after head injury in a patient with no history of epilepsy.
    pub seizure_no_epilepsy_history: bool,

    // --- Examination ---
    /// GCS below 15 (i.e. 14 or lower). Part of the raw GCS observation; the
    /// rule applies the age-dependent threshold from this and `gcs_below_14`.
    pub gcs_below_15: bool,
    /// GCS below 14 (i.e. 13 or lower). Required for the criterion to fire in a
    /// child of 1 year or older, so GCS exactly 14 does not fire over age 1.
    pub gcs_below_14: bool,
    /// Suspicion of penetrating or depressed skull injury, or a tense fontanelle.
    pub penetrating_or_depressed_skull_injury_or_tense_fontanelle: bool,
    /// Signs of a basal skull fracture (e.g. CSF/blood from ear or nose, panda
    /// eyes, Battle's sign, haemotympanum).
    pub basal_skull_fracture_signs: bool,
    /// Positive focal neurology.
    pub focal_neurology: bool,
    /// Presence of a bruise, swelling, or laceration greater than 5 cm (this
    /// criterion applies only if the child is under 1 year old).
    pub bruise_swelling_laceration_over_5cm: bool,

    // --- Mechanism ---
    /// High-speed road traffic accident as pedestrian, cyclist, or occupant
    /// (greater than 40 mph).
    pub high_speed_rta: bool,
    /// Fall of more than 3 metres in height.
    pub fall_over_3m: bool,
    /// High-speed injury from a projectile or object.
    pub high_speed_projectile: bool,
}

impl ChaliceInput {
    /// Whether the child is under 1 year old, used by the age-dependent criteria.
    fn under_1_year(&self) -> bool {
        self.age_years < 1
    }

    /// The raw GCS observation crosses the age-dependent CHALICE threshold.
    ///
    /// The threshold is GCS < 14 normally, tightening to GCS < 15 for infants
    /// under 1 year. The caller supplies `gcs_below_15` (true when GCS < 15);
    /// for a child of 1 year or more we additionally require GCS < 14, i.e. the
    /// criterion does not fire on GCS exactly 14.
    fn gcs_criterion(&self) -> bool {
        if self.under_1_year() {
            // Under 1 year: any GCS below 15 fires.
            self.gcs_below_15
        } else {
            // 1 year or older: only GCS below 14 fires. `gcs_below_15` alone is
            // not sufficient, so a separate observation is needed.
            self.gcs_below_14
        }
    }
}

/// A single CHALICE criterion that was found positive, with the group it belongs
/// to, for the response breakdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositiveCriterion {
    /// CHALICE group: "history", "examination", or "mechanism".
    pub group: &'static str,
    /// Machine key of the criterion.
    pub key: &'static str,
    /// Human-readable description of the criterion.
    pub label: &'static str,
}

/// The computed outcome of the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaliceOutcome {
    /// Whether a CT head scan is recommended (any criterion positive).
    pub ct_recommended: bool,
    /// The criteria that were positive, in algorithm order.
    pub positive: Vec<PositiveCriterion>,
    pub interpretation: String,
}

/// Pure evaluation of the decision rule.
pub fn compute(input: &ChaliceInput) -> Result<ChaliceOutcome, CalcError> {
    let mut positive: Vec<PositiveCriterion> = Vec::new();

    let mut add = |on: bool, group: &'static str, key: &'static str, label: &'static str| {
        if on {
            positive.push(PositiveCriterion { group, key, label });
        }
    };

    // History.
    add(
        input.loss_of_consciousness_over_5_min,
        "history",
        "loss_of_consciousness_over_5_min",
        "Witnessed loss of consciousness > 5 minutes",
    );
    add(
        input.amnesia_over_5_min,
        "history",
        "amnesia_over_5_min",
        "Amnesia (antegrade or retrograde) > 5 minutes",
    );
    add(
        input.abnormal_drowsiness,
        "history",
        "abnormal_drowsiness",
        "Abnormal drowsiness",
    );
    add(
        input.vomiting_3_or_more,
        "history",
        "vomiting_3_or_more",
        ">= 3 discrete episodes of vomiting",
    );
    add(
        input.suspected_nai,
        "history",
        "suspected_nai",
        "Suspicion of non-accidental injury",
    );
    add(
        input.seizure_no_epilepsy_history,
        "history",
        "seizure_no_epilepsy_history",
        "Seizure after head injury with no history of epilepsy",
    );

    // Examination.
    add(
        input.gcs_criterion(),
        "examination",
        "gcs",
        if input.under_1_year() {
            "GCS < 15 (under 1 year old)"
        } else {
            "GCS < 14"
        },
    );
    add(
        input.penetrating_or_depressed_skull_injury_or_tense_fontanelle,
        "examination",
        "penetrating_or_depressed_skull_injury_or_tense_fontanelle",
        "Suspected penetrating or depressed skull injury, or tense fontanelle",
    );
    add(
        input.basal_skull_fracture_signs,
        "examination",
        "basal_skull_fracture_signs",
        "Signs of a basal skull fracture",
    );
    add(
        input.focal_neurology,
        "examination",
        "focal_neurology",
        "Positive focal neurology",
    );
    // Age-dependent: only counts for infants under 1 year old.
    add(
        input.under_1_year() && input.bruise_swelling_laceration_over_5cm,
        "examination",
        "bruise_swelling_laceration_over_5cm",
        "Bruise, swelling, or laceration > 5 cm (under 1 year old)",
    );

    // Mechanism.
    add(
        input.high_speed_rta,
        "mechanism",
        "high_speed_rta",
        "High-speed road traffic accident (> 40 mph)",
    );
    add(
        input.fall_over_3m,
        "mechanism",
        "fall_over_3m",
        "Fall of > 3 m in height",
    );
    add(
        input.high_speed_projectile,
        "mechanism",
        "high_speed_projectile",
        "High-speed injury from a projectile or object",
    );

    let ct_recommended = !positive.is_empty();

    let interpretation = if ct_recommended {
        let labels: Vec<&str> = positive.iter().map(|c| c.label).collect();
        format!(
            "CT head recommended: {} CHALICE criterion/criteria positive ({}). Any positive \
criterion predicts a clinically significant intracranial injury, so a CT head scan is \
recommended (Dunning et al, Arch Dis Child 2006; NICE NG232).",
            positive.len(),
            labels.join("; ")
        )
    } else {
        "CT not indicated by CHALICE: no criterion is positive. The rule does not predict a \
clinically significant intracranial injury; observe and use clinical judgement, reassessing if \
the child deteriorates (Dunning et al, Arch Dis Child 2006; NICE NG232)."
            .to_string()
    };

    Ok(ChaliceOutcome {
        ct_recommended,
        positive,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &ChaliceInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("ct_recommended".into(), json!(o.ct_recommended));
    working.insert("under_1_year".into(), json!(input.under_1_year()));
    working.insert("positive_count".into(), json!(o.positive.len()));
    working.insert(
        "positive_criteria".into(),
        json!(o
            .positive
            .iter()
            .map(|c| json!({ "group": c.group, "key": c.key, "label": c.label }))
            .collect::<Vec<_>>()),
    );

    let result = if o.ct_recommended {
        "ct-recommended"
    } else {
        "ct-not-indicated"
    };

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(result),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Chalice;

impl Calculator for Chalice {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "CHALICE Paediatric Head Injury Rule"
    }

    fn description(&self) -> &'static str {
        "Decision rule for CT head in children after head injury: any positive criterion predicts a \
clinically significant intracranial injury and a CT head scan is recommended (Dunning et al 2006; \
NICE NG232)."
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
            "title": "ChaliceInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age_years",
                "loss_of_consciousness_over_5_min", "amnesia_over_5_min", "abnormal_drowsiness",
                "vomiting_3_or_more", "suspected_nai", "seizure_no_epilepsy_history",
                "gcs_below_15", "gcs_below_14",
                "penetrating_or_depressed_skull_injury_or_tense_fontanelle",
                "basal_skull_fracture_signs", "focal_neurology",
                "bruise_swelling_laceration_over_5cm",
                "high_speed_rta", "fall_over_3m", "high_speed_projectile"
            ],
            "properties": {
                "age_years": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 16,
                    "description": "Age in completed years (0 = under 1 year old). CHALICE applies to children under 16."
                },
                "loss_of_consciousness_over_5_min": {
                    "type": "boolean",
                    "description": "Witnessed loss of consciousness > 5 minutes (History)"
                },
                "amnesia_over_5_min": {
                    "type": "boolean",
                    "description": "Amnesia, antegrade or retrograde, > 5 minutes (History)"
                },
                "abnormal_drowsiness": {
                    "type": "boolean",
                    "description": "Abnormal drowsiness, exceeding the level expected (History)"
                },
                "vomiting_3_or_more": {
                    "type": "boolean",
                    "description": ">= 3 discrete episodes of vomiting after the injury (History)"
                },
                "suspected_nai": {
                    "type": "boolean",
                    "description": "Suspicion of non-accidental injury (History)"
                },
                "seizure_no_epilepsy_history": {
                    "type": "boolean",
                    "description": "Seizure after head injury in a patient with no history of epilepsy (History)"
                },
                "gcs_below_15": {
                    "type": "boolean",
                    "description": "GCS below 15 (i.e. GCS 14 or lower). Used with age for the GCS criterion.",
                    "definition": {
                        "concept": "GCS criterion (age-dependent)",
                        "statement": "The GCS criterion fires at GCS < 14, but tightens to GCS < 15 for infants under 1 year old.",
                        "caveats": "Supply gcs_below_15 (true when GCS < 15) AND gcs_below_14 (true when GCS < 14). For a child under 1 year, gcs_below_15 alone fires the criterion; for a child 1 year or older, gcs_below_14 is required, so GCS exactly 14 does NOT fire.",
                        "source": { "citation": "Dunning J et al. Arch Dis Child. 2006;91(11):885-891.", "url": "https://doi.org/10.1136/adc.2005.083980" },
                        "status": "draft"
                    }
                },
                "gcs_below_14": {
                    "type": "boolean",
                    "description": "GCS below 14 (i.e. GCS 13 or lower). Used for children 1 year or older.",
                    "definition": {
                        "concept": "GCS criterion (age-dependent)",
                        "statement": "For a child of 1 year or more the GCS criterion requires GCS < 14.",
                        "caveats": "Must be consistent with gcs_below_15: if gcs_below_14 is true then gcs_below_15 must also be true. The rule applies the age-dependent threshold from these two flags plus age_years.",
                        "source": { "citation": "Dunning J et al. Arch Dis Child. 2006;91(11):885-891.", "url": "https://doi.org/10.1136/adc.2005.083980" },
                        "status": "draft"
                    }
                },
                "penetrating_or_depressed_skull_injury_or_tense_fontanelle": {
                    "type": "boolean",
                    "description": "Suspected penetrating or depressed skull injury, or tense fontanelle (Examination)"
                },
                "basal_skull_fracture_signs": {
                    "type": "boolean",
                    "description": "Signs of a basal skull fracture, e.g. CSF/blood from ear or nose, panda eyes, Battle's sign, haemotympanum (Examination)"
                },
                "focal_neurology": {
                    "type": "boolean",
                    "description": "Positive focal neurology (Examination)"
                },
                "bruise_swelling_laceration_over_5cm": {
                    "type": "boolean",
                    "description": "Bruise, swelling, or laceration > 5 cm. Only counts if the child is under 1 year old.",
                    "definition": {
                        "concept": "Scalp injury > 5 cm (under 1 year only)",
                        "statement": "Presence of a bruise, swelling, or laceration greater than 5 cm is a CHALICE criterion ONLY for infants under 1 year old.",
                        "caveats": "For a child of 1 year or older this finding does not, by itself, satisfy CHALICE: the rule ignores this flag unless age_years is 0.",
                        "source": { "citation": "Dunning J et al. Arch Dis Child. 2006;91(11):885-891.", "url": "https://doi.org/10.1136/adc.2005.083980" },
                        "status": "draft"
                    }
                },
                "high_speed_rta": {
                    "type": "boolean",
                    "description": "High-speed road traffic accident as pedestrian, cyclist, or occupant (> 40 mph) (Mechanism)"
                },
                "fall_over_3m": {
                    "type": "boolean",
                    "description": "Fall of > 3 m in height (Mechanism)"
                },
                "high_speed_projectile": {
                    "type": "boolean",
                    "description": "High-speed injury from a projectile or object (Mechanism)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ChaliceInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A child with no positive criteria. `age_years` lets each test target the
    /// age-dependent behaviour.
    fn base(age_years: u8) -> ChaliceInput {
        ChaliceInput {
            age_years,
            loss_of_consciousness_over_5_min: false,
            amnesia_over_5_min: false,
            abnormal_drowsiness: false,
            vomiting_3_or_more: false,
            suspected_nai: false,
            seizure_no_epilepsy_history: false,
            gcs_below_15: false,
            gcs_below_14: false,
            penetrating_or_depressed_skull_injury_or_tense_fontanelle: false,
            basal_skull_fracture_signs: false,
            focal_neurology: false,
            bruise_swelling_laceration_over_5cm: false,
            high_speed_rta: false,
            fall_over_3m: false,
            high_speed_projectile: false,
        }
    }

    #[test]
    fn no_criteria_means_ct_not_indicated() {
        let o = compute(&base(5)).unwrap();
        assert!(!o.ct_recommended);
        assert!(o.positive.is_empty());
        assert!(o.interpretation.contains("CT not indicated"));
    }

    #[test]
    fn each_history_criterion_triggers_ct() {
        // Apply each history flag in isolation and confirm it alone fires.
        let setters: Vec<(&str, fn(&mut ChaliceInput))> = vec![
            ("loss_of_consciousness_over_5_min", |i| i.loss_of_consciousness_over_5_min = true),
            ("amnesia_over_5_min", |i| i.amnesia_over_5_min = true),
            ("abnormal_drowsiness", |i| i.abnormal_drowsiness = true),
            ("vomiting_3_or_more", |i| i.vomiting_3_or_more = true),
            ("suspected_nai", |i| i.suspected_nai = true),
            ("seizure_no_epilepsy_history", |i| i.seizure_no_epilepsy_history = true),
        ];
        for (key, set) in setters {
            let mut i = base(5);
            set(&mut i);
            let o = compute(&i).unwrap();
            assert!(o.ct_recommended, "{key} should recommend CT");
            assert_eq!(o.positive.len(), 1, "{key} should be the only positive");
            assert_eq!(o.positive[0].key, key);
            assert_eq!(o.positive[0].group, "history");
        }
    }

    #[test]
    fn each_examination_criterion_triggers_ct() {
        // GCS handled separately; bruise handled separately (age-dependent).
        let setters: Vec<(&str, fn(&mut ChaliceInput))> = vec![
            (
                "penetrating_or_depressed_skull_injury_or_tense_fontanelle",
                |i| i.penetrating_or_depressed_skull_injury_or_tense_fontanelle = true,
            ),
            ("basal_skull_fracture_signs", |i| i.basal_skull_fracture_signs = true),
            ("focal_neurology", |i| i.focal_neurology = true),
        ];
        for (key, set) in setters {
            let mut i = base(5);
            set(&mut i);
            let o = compute(&i).unwrap();
            assert!(o.ct_recommended, "{key} should recommend CT");
            assert_eq!(o.positive.len(), 1);
            assert_eq!(o.positive[0].key, key);
            assert_eq!(o.positive[0].group, "examination");
        }
    }

    #[test]
    fn each_mechanism_criterion_triggers_ct() {
        let setters: Vec<(&str, fn(&mut ChaliceInput))> = vec![
            ("high_speed_rta", |i| i.high_speed_rta = true),
            ("fall_over_3m", |i| i.fall_over_3m = true),
            ("high_speed_projectile", |i| i.high_speed_projectile = true),
        ];
        for (key, set) in setters {
            let mut i = base(5);
            set(&mut i);
            let o = compute(&i).unwrap();
            assert!(o.ct_recommended, "{key} should recommend CT");
            assert_eq!(o.positive.len(), 1);
            assert_eq!(o.positive[0].key, key);
            assert_eq!(o.positive[0].group, "mechanism");
        }
    }

    #[test]
    fn gcs_threshold_is_14_for_older_child() {
        // 5-year-old with GCS 14: gcs_below_15 true, gcs_below_14 false -> no fire.
        let mut i = base(5);
        i.gcs_below_15 = true;
        i.gcs_below_14 = false;
        assert!(!compute(&i).unwrap().ct_recommended);

        // GCS 13: gcs_below_14 true -> fires.
        i.gcs_below_14 = true;
        let o = compute(&i).unwrap();
        assert!(o.ct_recommended);
        assert_eq!(o.positive[0].key, "gcs");
        assert!(o.positive[0].label.contains("GCS < 14"));
    }

    #[test]
    fn gcs_threshold_is_15_for_infant() {
        // Under 1 year with GCS 14: gcs_below_15 true alone -> fires.
        let mut i = base(0);
        i.gcs_below_15 = true;
        i.gcs_below_14 = false;
        let o = compute(&i).unwrap();
        assert!(o.ct_recommended);
        assert_eq!(o.positive[0].key, "gcs");
        assert!(o.positive[0].label.contains("under 1 year"));
    }

    #[test]
    fn bruise_over_5cm_only_counts_under_1_year() {
        // 5-year-old: the bruise flag is ignored.
        let mut older = base(5);
        older.bruise_swelling_laceration_over_5cm = true;
        assert!(!compute(&older).unwrap().ct_recommended);

        // Under 1 year: the same flag fires.
        let mut infant = base(0);
        infant.bruise_swelling_laceration_over_5cm = true;
        let o = compute(&infant).unwrap();
        assert!(o.ct_recommended);
        assert_eq!(o.positive[0].key, "bruise_swelling_laceration_over_5cm");
    }

    #[test]
    fn multiple_positives_are_all_listed() {
        let mut i = base(0);
        i.suspected_nai = true;
        i.bruise_swelling_laceration_over_5cm = true;
        i.fall_over_3m = true;
        let o = compute(&i).unwrap();
        assert!(o.ct_recommended);
        assert_eq!(o.positive.len(), 3);
        let keys: Vec<&str> = o.positive.iter().map(|c| c.key).collect();
        assert!(keys.contains(&"suspected_nai"));
        assert!(keys.contains(&"bruise_swelling_laceration_over_5cm"));
        assert!(keys.contains(&"fall_over_3m"));
    }

    #[test]
    fn build_response_result_string() {
        let mut i = base(5);
        i.focal_neurology = true;
        let r = build_response(&i).unwrap();
        assert_eq!(r.result, json!("ct-recommended"));
        assert_eq!(r.working["ct_recommended"], json!(true));
        assert_eq!(r.working["positive_count"], json!(1));

        let clear = build_response(&base(5)).unwrap();
        assert_eq!(clear.result, json!("ct-not-indicated"));
        assert_eq!(clear.working["ct_recommended"], json!(false));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age_years": 0,
            "loss_of_consciousness_over_5_min": false,
            "amnesia_over_5_min": false,
            "abnormal_drowsiness": false,
            "vomiting_3_or_more": false,
            "suspected_nai": false,
            "seizure_no_epilepsy_history": false,
            "gcs_below_15": true,
            "gcs_below_14": false,
            "penetrating_or_depressed_skull_injury_or_tense_fontanelle": false,
            "basal_skull_fracture_signs": false,
            "focal_neurology": false,
            "bruise_swelling_laceration_over_5cm": true,
            "high_speed_rta": false,
            "fall_over_3m": false,
            "high_speed_projectile": false
        });
        let mut typed = base(0);
        typed.gcs_below_15 = true;
        typed.bruise_swelling_laceration_over_5cm = true;
        let dynamic = Chalice.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!("ct-recommended"));
    }

    #[test]
    fn schema_documents_age_dependent_criteria() {
        let schema = Chalice.input_schema();
        let gcs = &schema["properties"]["gcs_below_15"]["definition"];
        assert!(gcs["caveats"].as_str().unwrap().contains("under 1 year"));
        let bruise = &schema["properties"]["bruise_swelling_laceration_over_5cm"]["definition"];
        assert!(bruise["statement"].as_str().unwrap().contains("under 1 year"));
    }
}
