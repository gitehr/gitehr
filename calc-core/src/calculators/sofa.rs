//! SOFA - Sequential Organ Failure Assessment (Vincent et al., 1996).
//!
//! Grades dysfunction in six organ systems - respiratory, coagulation, hepatic,
//! cardiovascular, neurological, and renal - each scored 0 (normal) to 4 (most
//! abnormal), for a total of 0-24. Higher totals track steeply rising ICU
//! mortality. SOFA also underpins the Sepsis-3 definition of sepsis: a rise of
//! >= 2 points from baseline in a patient with suspected infection.
//!
//! Three clinical subtleties are encoded here:
//! - The respiratory rows for sub-scores 3 and 4 (PaO2/FiO2 < 200 and < 100)
//!   apply ONLY with respiratory support (mechanical ventilation or CPAP).
//!   Without support those low ratios are capped at sub-score 2, so the support
//!   flag is a required input, not an afterthought.
//! - Bilirubin and creatinine are accepted in either conventional (mg/dL) or SI
//!   (umol/L) units. UK laboratories report umol/L while the SOFA table is
//!   defined in mg/dL; the unit is a required input rather than assumed, because
//!   the two differ by ~17x (bilirubin) and ~88x (creatinine) and a wrong label
//!   silently shifts the sub-score.
//! - The cardiovascular axis mixes a blood-pressure threshold with named
//!   vasopressor agents and doses. Rather than ask for every drug and dose, the
//!   caller selects the single highest applicable support level from an enum,
//!   which keeps the contract tractable and unambiguous.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "sofa";

/// Primary citation.
pub const REFERENCE: &str =
    "Vincent JL, Moreno R, Takala J, et al. The SOFA (Sepsis-related Organ Failure Assessment) \
score to describe organ dysfunction/failure. Intensive Care Med. 1996;22(7):707-710. \
doi:10.1007/BF01709751. Sepsis-3: Singer M, et al. JAMA. 2016;315(8):801-810.";

/// Distribution licence: SOFA is a published clinical method, implemented here
/// from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (SOFA, Vincent et al. 1996)",
    source_url: "https://doi.org/10.1007/BF01709751",
};

/// umol/L per mg/dL for bilirubin (molar mass 584.66 g/mol).
pub const BILIRUBIN_UMOL_PER_MGDL: f64 = 17.1;

/// umol/L per mg/dL for creatinine (molar mass 113.12 g/mol).
pub const CREATININE_UMOL_PER_MGDL: f64 = 88.4;

/// Unit a bilirubin value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BilirubinUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// Unit a creatinine value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatinineUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// The single highest applicable cardiovascular support level. The caller picks
/// one; the levels map directly onto SOFA sub-scores 0-4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cardiovascular {
    /// MAP >= 70 mmHg, no vasopressor support. Sub-score 0.
    MapAtLeast70,
    /// MAP < 70 mmHg, no vasopressor support. Sub-score 1.
    MapBelow70,
    /// Dopamine <= 5 mcg/kg/min OR dobutamine at any dose. Sub-score 2.
    DopamineLow,
    /// Dopamine > 5, OR epinephrine <= 0.1, OR norepinephrine <= 0.1
    /// mcg/kg/min. Sub-score 3.
    ModerateVasopressors,
    /// Dopamine > 15, OR epinephrine > 0.1, OR norepinephrine > 0.1
    /// mcg/kg/min. Sub-score 4.
    HighVasopressors,
}

impl Cardiovascular {
    /// SOFA cardiovascular sub-score for this support level.
    fn score(self) -> u8 {
        match self {
            Cardiovascular::MapAtLeast70 => 0,
            Cardiovascular::MapBelow70 => 1,
            Cardiovascular::DopamineLow => 2,
            Cardiovascular::ModerateVasopressors => 3,
            Cardiovascular::HighVasopressors => 4,
        }
    }
}

/// SOFA inputs. Bilirubin and creatinine each carry their own unit; the
/// cardiovascular axis is an enum of support levels; respiratory support gates
/// the two most severe respiratory rows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SofaInput {
    /// PaO2/FiO2 ratio in mmHg (the "P/F ratio").
    pub pao2_fio2: f64,
    /// Whether the patient is receiving respiratory support (mechanical
    /// ventilation or CPAP). Required for the < 200 and < 100 respiratory rows.
    pub respiratory_support: bool,
    /// Platelet count, x10^3/uL (i.e. x10^9/L).
    pub platelets: f64,
    /// Serum bilirubin, in the unit given by `bilirubin_unit`.
    pub bilirubin: f64,
    pub bilirubin_unit: BilirubinUnit,
    /// Highest applicable cardiovascular support level.
    pub cardiovascular: Cardiovascular,
    /// Glasgow Coma Scale total (3-15).
    pub gcs: u8,
    /// Serum creatinine, in the unit given by `creatinine_unit`.
    pub creatinine: f64,
    pub creatinine_unit: CreatinineUnit,
}

/// The computed outcome, with every organ sub-score exposed for transparency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SofaOutcome {
    /// Total SOFA score (0-24).
    pub total: u8,
    pub respiration_score: u8,
    pub coagulation_score: u8,
    pub liver_score: u8,
    pub cardiovascular_score: u8,
    pub cns_score: u8,
    pub renal_score: u8,
    pub interpretation: String,
}

/// Respiration: PaO2/FiO2 in mmHg. Sub-scores 3 and 4 require respiratory
/// support; without it, a ratio below 200 is capped at 2.
fn respiration_score(pao2_fio2: f64, respiratory_support: bool) -> u8 {
    if pao2_fio2 >= 400.0 {
        0
    } else if pao2_fio2 >= 300.0 {
        1
    } else if pao2_fio2 >= 200.0 {
        2
    } else if pao2_fio2 >= 100.0 {
        // < 200: needs support to score 3, else capped at 2.
        if respiratory_support {
            3
        } else {
            2
        }
    } else {
        // < 100: needs support to score 4, else capped at 2.
        if respiratory_support {
            4
        } else {
            2
        }
    }
}

/// Coagulation: platelets, x10^3/uL.
fn coagulation_score(platelets: f64) -> u8 {
    if platelets >= 150.0 {
        0
    } else if platelets >= 100.0 {
        1
    } else if platelets >= 50.0 {
        2
    } else if platelets >= 20.0 {
        3
    } else {
        4
    }
}

/// Liver: bilirubin, mg/dL.
fn liver_score(bilirubin_mgdl: f64) -> u8 {
    if bilirubin_mgdl < 1.2 {
        0
    } else if bilirubin_mgdl < 2.0 {
        1
    } else if bilirubin_mgdl < 6.0 {
        2
    } else if bilirubin_mgdl < 12.0 {
        3
    } else {
        4
    }
}

/// CNS: Glasgow Coma Scale total (3-15).
fn cns_score(gcs: u8) -> u8 {
    match gcs {
        15 => 0,
        13 | 14 => 1,
        10..=12 => 2,
        6..=9 => 3,
        _ => 4, // < 6
    }
}

/// Renal: creatinine, mg/dL. (Urine-output rows are an alternative basis the
/// caller can apply at the bedside; this implementation scores on creatinine.)
fn renal_score(creatinine_mgdl: f64) -> u8 {
    if creatinine_mgdl < 1.2 {
        0
    } else if creatinine_mgdl < 2.0 {
        1
    } else if creatinine_mgdl < 3.5 {
        2
    } else if creatinine_mgdl < 5.0 {
        3
    } else {
        4
    }
}

/// Pure scoring.
pub fn compute(input: &SofaInput) -> Result<SofaOutcome, CalcError> {
    if !input.pao2_fio2.is_finite() || input.pao2_fio2 <= 0.0 {
        return Err(CalcError::InvalidInput(
            "pao2_fio2 must be a positive number".into(),
        ));
    }
    if !input.platelets.is_finite() || input.platelets < 0.0 {
        return Err(CalcError::InvalidInput(
            "platelets must be a non-negative number".into(),
        ));
    }
    if !input.bilirubin.is_finite() || input.bilirubin < 0.0 {
        return Err(CalcError::InvalidInput(
            "bilirubin must be a non-negative number".into(),
        ));
    }
    if !input.creatinine.is_finite() || input.creatinine < 0.0 {
        return Err(CalcError::InvalidInput(
            "creatinine must be a non-negative number".into(),
        ));
    }
    if input.gcs < 3 || input.gcs > 15 {
        return Err(CalcError::InvalidInput(
            "gcs must be between 3 and 15".into(),
        ));
    }

    // Normalise bilirubin and creatinine to mg/dL, the unit the SOFA table is
    // defined in.
    let bilirubin_mgdl = match input.bilirubin_unit {
        BilirubinUnit::MgDl => input.bilirubin,
        BilirubinUnit::UmolL => input.bilirubin / BILIRUBIN_UMOL_PER_MGDL,
    };
    let creatinine_mgdl = match input.creatinine_unit {
        CreatinineUnit::MgDl => input.creatinine,
        CreatinineUnit::UmolL => input.creatinine / CREATININE_UMOL_PER_MGDL,
    };

    let respiration_score = respiration_score(input.pao2_fio2, input.respiratory_support);
    let coagulation_score = coagulation_score(input.platelets);
    let liver_score = liver_score(bilirubin_mgdl);
    let cardiovascular_score = input.cardiovascular.score();
    let cns_score = cns_score(input.gcs);
    let renal_score = renal_score(creatinine_mgdl);

    let total = respiration_score
        + coagulation_score
        + liver_score
        + cardiovascular_score
        + cns_score
        + renal_score;

    let interpretation = format!(
        "Total SOFA {total}/24 (respiration {respiration_score}, coagulation {coagulation_score}, \
liver {liver_score}, cardiovascular {cardiovascular_score}, CNS {cns_score}, renal {renal_score}). \
A higher score reflects more organ dysfunction, and ICU mortality rises steeply with the total. \
SOFA underpins the Sepsis-3 definition: in a patient with suspected infection, an acute rise of \
>= 2 points from baseline indicates sepsis. SOFA describes organ dysfunction; it is not a \
diagnosis by itself and should be interpreted alongside the clinical picture and trend."
    );

    Ok(SofaOutcome {
        total,
        respiration_score,
        coagulation_score,
        liver_score,
        cardiovascular_score,
        cns_score,
        renal_score,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &SofaInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("respiration_score".into(), json!(o.respiration_score));
    working.insert("coagulation_score".into(), json!(o.coagulation_score));
    working.insert("liver_score".into(), json!(o.liver_score));
    working.insert("cardiovascular_score".into(), json!(o.cardiovascular_score));
    working.insert("cns_score".into(), json!(o.cns_score));
    working.insert("renal_score".into(), json!(o.renal_score));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Sofa;

impl Calculator for Sofa {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "SOFA Score (Sequential Organ Failure Assessment)"
    }

    fn description(&self) -> &'static str {
        "Grades dysfunction across six organ systems (0-24); underpins the Sepsis-3 definition \
(rise >= 2 from baseline)."
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
            "title": "SofaInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "pao2_fio2", "respiratory_support", "platelets",
                "bilirubin", "bilirubin_unit", "cardiovascular",
                "gcs", "creatinine", "creatinine_unit"
            ],
            "properties": {
                "pao2_fio2": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "maximum": 700,
                    "description": "PaO2/FiO2 ratio, mmHg (>=400 scores 0; <100 with support scores 4)"
                },
                "respiratory_support": {
                    "type": "boolean",
                    "description": "On mechanical ventilation or CPAP (required for the <200/<100 respiratory rows)",
                    "definition": {
                        "concept": "Respiratory support",
                        "statement": "TRUE if the patient is receiving mechanical ventilation or CPAP. The respiratory sub-scores of 3 (PaO2/FiO2 < 200) and 4 (< 100) apply only with respiratory support.",
                        "excludes": [
                            "Without respiratory support, a PaO2/FiO2 below 200 is capped at sub-score 2: the original SOFA table reserves the 3 and 4 rows for ventilated/CPAP patients"
                        ],
                        "source": { "citation": "Vincent JL et al. Intensive Care Med. 1996;22(7):707-710.", "url": "https://doi.org/10.1007/BF01709751" },
                        "status": "draft"
                    }
                },
                "platelets": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 2000,
                    "description": "Platelet count, x10^3/uL i.e. x10^9/L (>=150 scores 0; <20 scores 4)"
                },
                "bilirubin": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Serum bilirubin value, in the unit given by bilirubin_unit"
                },
                "bilirubin_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the bilirubin value",
                    "definition": {
                        "concept": "Bilirubin unit",
                        "statement": "The SOFA table is defined in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 17.1 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~17x and the wrong label silently shifts the liver sub-score"
                        ],
                        "source": { "citation": "Vincent JL et al. Intensive Care Med. 1996;22(7):707-710.", "url": "https://doi.org/10.1007/BF01709751" },
                        "status": "draft"
                    }
                },
                "cardiovascular": {
                    "type": "string",
                    "enum": [
                        "map-at-least70", "map-below70", "dopamine-low",
                        "moderate-vasopressors", "high-vasopressors"
                    ],
                    "description": "Highest applicable cardiovascular support level (maps to sub-score 0-4)",
                    "definition": {
                        "concept": "Cardiovascular support level",
                        "statement": "The cardiovascular axis combines a mean-arterial-pressure threshold with named vasopressor agents and doses (mcg/kg/min). Select the single HIGHEST applicable level: map-at-least70 (MAP >= 70, score 0); map-below70 (MAP < 70, no pressors, score 1); dopamine-low (dopamine <= 5 OR dobutamine any dose, score 2); moderate-vasopressors (dopamine > 5 OR epinephrine <= 0.1 OR norepinephrine <= 0.1, score 3); high-vasopressors (dopamine > 15 OR epinephrine > 0.1 OR norepinephrine > 0.1, score 4).",
                        "caveats": "Doses are in mcg/kg/min and refer to the rate sustained for at least one hour in the original definition. If more than one level applies, pick the highest.",
                        "source": { "citation": "Vincent JL et al. Intensive Care Med. 1996;22(7):707-710.", "url": "https://doi.org/10.1007/BF01709751" },
                        "status": "draft"
                    }
                },
                "gcs": {
                    "type": "integer",
                    "minimum": 3,
                    "maximum": 15,
                    "description": "Glasgow Coma Scale total (15 scores 0; <6 scores 4)"
                },
                "creatinine": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Serum creatinine value, in the unit given by creatinine_unit"
                },
                "creatinine_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the creatinine value",
                    "definition": {
                        "concept": "Creatinine unit",
                        "statement": "The SOFA table is defined in mg/dL. UK laboratories report umol/L; the value is converted internally (1 mg/dL = 88.4 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~88x and the wrong label silently shifts the renal sub-score"
                        ],
                        "source": { "citation": "Vincent JL et al. Intensive Care Med. 1996;22(7):707-710.", "url": "https://doi.org/10.1007/BF01709751" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: SofaInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wholly normal patient: every organ sub-score is 0.
    fn normal() -> SofaInput {
        SofaInput {
            pao2_fio2: 450.0,
            respiratory_support: false,
            platelets: 250.0,
            bilirubin: 0.5,
            bilirubin_unit: BilirubinUnit::MgDl,
            cardiovascular: Cardiovascular::MapAtLeast70,
            gcs: 15,
            creatinine: 0.8,
            creatinine_unit: CreatinineUnit::MgDl,
        }
    }

    #[test]
    fn all_normal_scores_zero() {
        let o = compute(&normal()).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.respiration_score, 0);
        assert_eq!(o.coagulation_score, 0);
        assert_eq!(o.liver_score, 0);
        assert_eq!(o.cardiovascular_score, 0);
        assert_eq!(o.cns_score, 0);
        assert_eq!(o.renal_score, 0);
    }

    #[test]
    fn worked_multi_organ_example() {
        // Respiration P/F 150 on support (3) + platelets 80 (2) + bilirubin 7.0
        // mg/dL (3) + moderate vasopressors (3) + GCS 11 (2) + creatinine 4.0
        // mg/dL (3) = 16.
        let i = SofaInput {
            pao2_fio2: 150.0,
            respiratory_support: true,
            platelets: 80.0,
            bilirubin: 7.0,
            bilirubin_unit: BilirubinUnit::MgDl,
            cardiovascular: Cardiovascular::ModerateVasopressors,
            gcs: 11,
            creatinine: 4.0,
            creatinine_unit: CreatinineUnit::MgDl,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.respiration_score, 3);
        assert_eq!(o.coagulation_score, 2);
        assert_eq!(o.liver_score, 3);
        assert_eq!(o.cardiovascular_score, 3);
        assert_eq!(o.cns_score, 2);
        assert_eq!(o.renal_score, 3);
        assert_eq!(o.total, 16);
    }

    #[test]
    fn maximum_score_is_24() {
        let i = SofaInput {
            pao2_fio2: 50.0,
            respiratory_support: true,
            platelets: 10.0,
            bilirubin: 15.0,
            bilirubin_unit: BilirubinUnit::MgDl,
            cardiovascular: Cardiovascular::HighVasopressors,
            gcs: 3,
            creatinine: 6.0,
            creatinine_unit: CreatinineUnit::MgDl,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 24);
    }

    #[test]
    fn respiration_boundaries() {
        // Without support, the 3/4 rows are capped at 2.
        assert_eq!(respiration_score(400.0, false), 0);
        assert_eq!(respiration_score(399.0, false), 1);
        assert_eq!(respiration_score(300.0, false), 1);
        assert_eq!(respiration_score(299.0, false), 2);
        assert_eq!(respiration_score(200.0, false), 2);
        assert_eq!(respiration_score(199.0, false), 2, "<200 without support caps at 2");
        assert_eq!(respiration_score(99.0, false), 2, "<100 without support caps at 2");
        // With support, the 3/4 rows apply.
        assert_eq!(respiration_score(199.0, true), 3);
        assert_eq!(respiration_score(100.0, true), 3);
        assert_eq!(respiration_score(99.0, true), 4);
    }

    #[test]
    fn respiratory_support_gates_severe_rows() {
        let mut i = normal();
        i.pao2_fio2 = 90.0;
        i.respiratory_support = false;
        assert_eq!(compute(&i).unwrap().respiration_score, 2);
        i.respiratory_support = true;
        assert_eq!(compute(&i).unwrap().respiration_score, 4);
    }

    #[test]
    fn coagulation_boundaries() {
        assert_eq!(coagulation_score(150.0), 0);
        assert_eq!(coagulation_score(149.0), 1);
        assert_eq!(coagulation_score(100.0), 1);
        assert_eq!(coagulation_score(99.0), 2);
        assert_eq!(coagulation_score(50.0), 2);
        assert_eq!(coagulation_score(49.0), 3);
        assert_eq!(coagulation_score(20.0), 3);
        assert_eq!(coagulation_score(19.0), 4);
    }

    #[test]
    fn liver_boundaries() {
        assert_eq!(liver_score(1.19), 0);
        assert_eq!(liver_score(1.2), 1);
        assert_eq!(liver_score(1.9), 1);
        assert_eq!(liver_score(2.0), 2);
        assert_eq!(liver_score(5.9), 2);
        assert_eq!(liver_score(6.0), 3);
        assert_eq!(liver_score(11.9), 3);
        assert_eq!(liver_score(12.0), 4);
    }

    #[test]
    fn cns_boundaries() {
        assert_eq!(cns_score(15), 0);
        assert_eq!(cns_score(14), 1);
        assert_eq!(cns_score(13), 1);
        assert_eq!(cns_score(12), 2);
        assert_eq!(cns_score(10), 2);
        assert_eq!(cns_score(9), 3);
        assert_eq!(cns_score(6), 3);
        assert_eq!(cns_score(5), 4);
        assert_eq!(cns_score(3), 4);
    }

    #[test]
    fn renal_boundaries() {
        assert_eq!(renal_score(1.19), 0);
        assert_eq!(renal_score(1.2), 1);
        assert_eq!(renal_score(1.9), 1);
        assert_eq!(renal_score(2.0), 2);
        assert_eq!(renal_score(3.4), 2);
        assert_eq!(renal_score(3.5), 3);
        assert_eq!(renal_score(4.9), 3);
        assert_eq!(renal_score(5.0), 4);
    }

    #[test]
    fn cardiovascular_levels_map_to_scores() {
        assert_eq!(Cardiovascular::MapAtLeast70.score(), 0);
        assert_eq!(Cardiovascular::MapBelow70.score(), 1);
        assert_eq!(Cardiovascular::DopamineLow.score(), 2);
        assert_eq!(Cardiovascular::ModerateVasopressors.score(), 3);
        assert_eq!(Cardiovascular::HighVasopressors.score(), 4);
    }

    #[test]
    fn bilirubin_umol_matches_equivalent_mgdl() {
        // 7.0 mg/dL == 119.7 umol/L; both must give liver sub-score 3.
        let mut a = normal();
        a.bilirubin = 7.0;
        a.bilirubin_unit = BilirubinUnit::MgDl;
        let mut b = normal();
        b.bilirubin = 7.0 * BILIRUBIN_UMOL_PER_MGDL;
        b.bilirubin_unit = BilirubinUnit::UmolL;
        assert_eq!(compute(&a).unwrap().liver_score, 3);
        assert_eq!(compute(&a).unwrap().liver_score, compute(&b).unwrap().liver_score);
    }

    #[test]
    fn creatinine_umol_matches_equivalent_mgdl() {
        // 4.0 mg/dL == 353.6 umol/L; both must give renal sub-score 3.
        let mut a = normal();
        a.creatinine = 4.0;
        a.creatinine_unit = CreatinineUnit::MgDl;
        let mut b = normal();
        b.creatinine = 4.0 * CREATININE_UMOL_PER_MGDL;
        b.creatinine_unit = CreatinineUnit::UmolL;
        assert_eq!(compute(&a).unwrap().renal_score, 3);
        assert_eq!(compute(&a).unwrap().renal_score, compute(&b).unwrap().renal_score);
    }

    #[test]
    fn rejects_bad_input() {
        let mut i = normal();
        i.pao2_fio2 = 0.0;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.pao2_fio2 = f64::NAN;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.platelets = -1.0;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.bilirubin = -0.1;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.creatinine = -0.1;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.gcs = 2;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.gcs = 16;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn build_response_carries_subscores_and_reference() {
        let i = SofaInput {
            pao2_fio2: 150.0,
            respiratory_support: true,
            platelets: 80.0,
            bilirubin: 7.0,
            bilirubin_unit: BilirubinUnit::MgDl,
            cardiovascular: Cardiovascular::ModerateVasopressors,
            gcs: 11,
            creatinine: 4.0,
            creatinine_unit: CreatinineUnit::MgDl,
        };
        let r = build_response(&i).unwrap();
        assert_eq!(r.calculator, "sofa");
        assert_eq!(r.result, json!(16));
        for key in [
            "total_score",
            "respiration_score",
            "coagulation_score",
            "liver_score",
            "cardiovascular_score",
            "cns_score",
            "renal_score",
        ] {
            assert!(r.working.contains_key(key), "missing {key}");
        }
        assert_eq!(r.working["renal_score"], json!(3));
        assert!(r.reference.contains("Vincent JL"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "pao2_fio2": 150.0,
            "respiratory_support": true,
            "platelets": 80.0,
            "bilirubin": 7.0,
            "bilirubin_unit": "mg/dL",
            "cardiovascular": "moderate-vasopressors",
            "gcs": 11,
            "creatinine": 4.0,
            "creatinine_unit": "mg/dL"
        });
        let typed = SofaInput {
            pao2_fio2: 150.0,
            respiratory_support: true,
            platelets: 80.0,
            bilirubin: 7.0,
            bilirubin_unit: BilirubinUnit::MgDl,
            cardiovascular: Cardiovascular::ModerateVasopressors,
            gcs: 11,
            creatinine: 4.0,
            creatinine_unit: CreatinineUnit::MgDl,
        };
        let dynamic = Sofa.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(16));
    }

    #[test]
    fn dynamic_calculate_rejects_garbage() {
        assert!(Sofa.calculate(&json!({ "pao2_fio2": "low" })).is_err());
    }

    #[test]
    fn schema_flags_unit_and_support_traps() {
        let schema = Sofa.input_schema();
        let bili = &schema["properties"]["bilirubin_unit"]["definition"]["excludes"][0];
        assert!(bili.as_str().unwrap().contains("17x"));
        let creat = &schema["properties"]["creatinine_unit"]["definition"]["excludes"][0];
        assert!(creat.as_str().unwrap().contains("88x"));
        let support = &schema["properties"]["respiratory_support"]["definition"]["excludes"][0];
        assert!(support.as_str().unwrap().contains("capped at sub-score 2"));
    }
}
