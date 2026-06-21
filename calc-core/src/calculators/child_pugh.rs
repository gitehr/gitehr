//! Child-Pugh - severity of chronic liver disease (cirrhosis).
//!
//! The Child-Turcotte-Pugh score grades cirrhosis from five parameters, each
//! scored 1-3, for a total of 5-15 mapping to classes A (5-6,
//! well-compensated), B (7-9, significant functional compromise), and C (10-15,
//! decompensated). Used for prognosis and, historically, surgical risk.
//!
//! Two of the five parameters carry unit traps. Bilirubin and albumin are
//! accepted in both conventional and SI units, which differ by large factors:
//! UK laboratories report bilirubin in umol/L and albumin in g/L, while the
//! original criteria are stated in mg/dL and g/dL. The unit is therefore a
//! required input rather than assumed - a wrong label silently shifts the
//! sub-score and can change the class. The remaining three parameters
//! (INR, ascites, encephalopathy) are graded categories whose boundaries are
//! easy to get subtly wrong, so each carries a `definition` block.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "child_pugh";

/// Distribution licence: the Child-Pugh score is a published clinical method,
/// implemented here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1002/bjs.1800600817",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Pugh RNH, Murray-Lyon IM, Dawson JL, Pietroni MC, Williams R. Transection of the oesophagus \
for bleeding oesophageal varices. Br J Surg. 1973;60(8):646-649. doi:10.1002/bjs.1800600817";

/// umol/L per mg/dL for bilirubin (molar mass 584.66 g/mol): 1 mg/dL = 17.1 umol/L.
pub const BILIRUBIN_UMOL_PER_MGDL: f64 = 17.1;

/// Unit the total bilirubin value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BilirubinUnit {
    #[serde(rename = "mg/dL")]
    MgDl,
    #[serde(rename = "umol/L")]
    UmolL,
}

/// Unit the serum albumin value is expressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlbuminUnit {
    #[serde(rename = "g/dL")]
    GDl,
    #[serde(rename = "g/L")]
    GL,
}

/// Severity of ascites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ascites {
    /// No ascites (1 point).
    None,
    /// Mild, or controlled by diuretics (2 points).
    Mild,
    /// Moderate-to-severe, or refractory to diuretics (3 points).
    ModerateSevere,
}

/// Severity of hepatic encephalopathy (West Haven grades).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encephalopathy {
    /// No encephalopathy (1 point).
    None,
    /// Grade 1-2, or suppressed with medication (2 points).
    Grade1To2,
    /// Grade 3-4, or refractory (3 points).
    Grade3To4,
}

/// Child-Pugh inputs. Bilirubin and albumin are numeric with an explicit unit;
/// ascites and encephalopathy are graded categories.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChildPughInput {
    /// Total bilirubin, in the unit given by `bilirubin_unit`.
    pub bilirubin: f64,
    pub bilirubin_unit: BilirubinUnit,
    /// Serum albumin, in the unit given by `albumin_unit`.
    pub albumin: f64,
    pub albumin_unit: AlbuminUnit,
    /// International normalised ratio (prothrombin time).
    pub inr: f64,
    pub ascites: Ascites,
    pub encephalopathy: Encephalopathy,
}

/// Child-Pugh class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Class A: 5-6 points, well-compensated disease.
    A,
    /// Class B: 7-9 points, significant functional compromise.
    B,
    /// Class C: 10-15 points, decompensated disease.
    C,
}

impl Class {
    fn from_score(score: u8) -> Self {
        if score <= 6 {
            Class::A
        } else if score <= 9 {
            Class::B
        } else {
            Class::C
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Class::A => "A",
            Class::B => "B",
            Class::C => "C",
        }
    }

    fn descriptor(self) -> &'static str {
        match self {
            Class::A => "well-compensated disease",
            Class::B => "significant functional compromise",
            Class::C => "decompensated disease",
        }
    }
}

/// The computed outcome, with each parameter's sub-score retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildPughOutcome {
    /// Total score (5-15).
    pub score: u8,
    pub class: Class,
    pub bilirubin_points: u8,
    pub albumin_points: u8,
    pub inr_points: u8,
    pub ascites_points: u8,
    pub encephalopathy_points: u8,
    pub interpretation: String,
}

/// Bilirubin sub-score, on a value already normalised to mg/dL.
/// <2 -> 1; 2-3 inclusive -> 2; >3 -> 3.
fn bilirubin_points(mgdl: f64) -> u8 {
    if mgdl < 2.0 {
        1
    } else if mgdl <= 3.0 {
        2
    } else {
        3
    }
}

/// Albumin sub-score, on a value already normalised to g/dL.
/// >3.5 -> 1; 2.8-3.5 inclusive -> 2; <2.8 -> 3.
fn albumin_points(gdl: f64) -> u8 {
    if gdl > 3.5 {
        1
    } else if gdl >= 2.8 {
        2
    } else {
        3
    }
}

/// INR sub-score. <1.7 -> 1; 1.7-2.3 inclusive -> 2; >2.3 -> 3.
fn inr_points(inr: f64) -> u8 {
    if inr < 1.7 {
        1
    } else if inr <= 2.3 {
        2
    } else {
        3
    }
}

fn ascites_points(ascites: Ascites) -> u8 {
    match ascites {
        Ascites::None => 1,
        Ascites::Mild => 2,
        Ascites::ModerateSevere => 3,
    }
}

fn encephalopathy_points(enc: Encephalopathy) -> u8 {
    match enc {
        Encephalopathy::None => 1,
        Encephalopathy::Grade1To2 => 2,
        Encephalopathy::Grade3To4 => 3,
    }
}

/// Pure scoring.
pub fn compute(input: &ChildPughInput) -> Result<ChildPughOutcome, CalcError> {
    if input.bilirubin <= 0.0 || !input.bilirubin.is_finite() {
        return Err(CalcError::InvalidInput(
            "bilirubin must be a positive number".into(),
        ));
    }
    if input.albumin <= 0.0 || !input.albumin.is_finite() {
        return Err(CalcError::InvalidInput(
            "albumin must be a positive number".into(),
        ));
    }
    if input.inr <= 0.0 || !input.inr.is_finite() {
        return Err(CalcError::InvalidInput(
            "INR must be a positive number".into(),
        ));
    }

    // Normalise bilirubin to mg/dL and albumin to g/dL.
    let bilirubin_mgdl = match input.bilirubin_unit {
        BilirubinUnit::MgDl => input.bilirubin,
        BilirubinUnit::UmolL => input.bilirubin / BILIRUBIN_UMOL_PER_MGDL,
    };
    let albumin_gdl = match input.albumin_unit {
        AlbuminUnit::GDl => input.albumin,
        AlbuminUnit::GL => input.albumin / 10.0,
    };

    let bilirubin_points = bilirubin_points(bilirubin_mgdl);
    let albumin_points = albumin_points(albumin_gdl);
    let inr_points = inr_points(input.inr);
    let ascites_points = ascites_points(input.ascites);
    let encephalopathy_points = encephalopathy_points(input.encephalopathy);

    let score = bilirubin_points
        + albumin_points
        + inr_points
        + ascites_points
        + encephalopathy_points;

    let class = Class::from_score(score);

    let interpretation = format!(
        "Child-Pugh score {score}: class {} ({}). Class A 5-6, B 7-9, C 10-15. The class informs \
prognosis and the suitability of interventions (for example, hepatic surgery and TIPS carry \
markedly higher risk in classes B and C). The score does not replace specialist hepatology \
assessment, and ascites and encephalopathy are subjective grades that should be assessed at the \
bedside.",
        class.slug(),
        class.descriptor()
    );

    Ok(ChildPughOutcome {
        score,
        class,
        bilirubin_points,
        albumin_points,
        inr_points,
        ascites_points,
        encephalopathy_points,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &ChildPughInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("bilirubin_points".into(), json!(o.bilirubin_points));
    working.insert("albumin_points".into(), json!(o.albumin_points));
    working.insert("inr_points".into(), json!(o.inr_points));
    working.insert("ascites_points".into(), json!(o.ascites_points));
    working.insert("encephalopathy_points".into(), json!(o.encephalopathy_points));
    working.insert("total_score".into(), json!(o.score));
    working.insert("class".into(), json!(o.class.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct ChildPugh;

impl Calculator for ChildPugh {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Child-Pugh Score (Cirrhosis Severity)"
    }

    fn description(&self) -> &'static str {
        "Severity of chronic liver disease from bilirubin, albumin, INR, ascites, and encephalopathy; reports class A/B/C."
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
            "title": "ChildPughInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "bilirubin", "bilirubin_unit", "albumin", "albumin_unit",
                "inr", "ascites", "encephalopathy"
            ],
            "properties": {
                "bilirubin": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Total bilirubin, in the unit given by bilirubin_unit"
                },
                "bilirubin_unit": {
                    "type": "string",
                    "enum": ["mg/dL", "umol/L"],
                    "description": "Unit of the bilirubin value",
                    "definition": {
                        "concept": "Bilirubin unit",
                        "statement": "The original criteria are stated in mg/dL (<2 -> 1, 2-3 -> 2, >3 -> 3). UK laboratories report umol/L (<34 -> 1, 34-50 -> 2, >50 -> 3); the value is converted internally (1 mg/dL = 17.1 umol/L).",
                        "excludes": [
                            "Do NOT pass a umol/L value while labelling it mg/dL: the two differ by ~17x and the wrong label silently changes the sub-score and can change the class"
                        ],
                        "source": {
                            "citation": "Pugh RNH et al. Br J Surg. 1973;60(8):646-649.",
                            "url": "https://doi.org/10.1002/bjs.1800600817"
                        },
                        "status": "draft"
                    }
                },
                "albumin": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Serum albumin, in the unit given by albumin_unit"
                },
                "albumin_unit": {
                    "type": "string",
                    "enum": ["g/dL", "g/L"],
                    "description": "Unit of the albumin value",
                    "definition": {
                        "concept": "Albumin unit",
                        "statement": "The original criteria are stated in g/dL (>3.5 -> 1, 2.8-3.5 -> 2, <2.8 -> 3). UK laboratories report g/L (>35 -> 1, 28-35 -> 2, <28 -> 3); the value is converted internally (g/L = g/dL x 10).",
                        "excludes": [
                            "Do NOT pass a g/L value (e.g. 35) while labelling it g/dL: the two differ by 10x and the wrong label silently changes the sub-score"
                        ],
                        "source": {
                            "citation": "Pugh RNH et al. Br J Surg. 1973;60(8):646-649.",
                            "url": "https://doi.org/10.1002/bjs.1800600817"
                        },
                        "status": "draft"
                    }
                },
                "inr": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "International normalised ratio (<1.7 -> 1, 1.7-2.3 -> 2, >2.3 -> 3)",
                    "definition": {
                        "concept": "INR / prothrombin time",
                        "statement": "Coagulation is scored by INR: <1.7 -> 1, 1.7-2.3 -> 2, >2.3 -> 3. The original Pugh criterion used prothrombin time prolongation in seconds (<4 -> 1, 4-6 -> 2, >6 -> 3); INR is the modern equivalent used here.",
                        "caveats": "Use the INR, not the prothrombin time in seconds. The two are different scales and are not interchangeable.",
                        "source": {
                            "citation": "Pugh RNH et al. Br J Surg. 1973;60(8):646-649.",
                            "url": "https://doi.org/10.1002/bjs.1800600817"
                        },
                        "status": "draft"
                    }
                },
                "ascites": {
                    "type": "string",
                    "enum": ["none", "mild", "moderate-severe"],
                    "description": "Severity of ascites (none -> 1, mild -> 2, moderate-severe -> 3)",
                    "definition": {
                        "concept": "Ascites grade",
                        "statement": "None -> 1 point; mild (or controlled by diuretics) -> 2 points; moderate-to-severe (or refractory to diuretics) -> 3 points.",
                        "includes": [
                            "Diuretic-controlled ascites scores 2 (mild), not 1 - the response to treatment counts, not the current clinical volume alone",
                            "Refractory or tense ascites scores 3 (moderate-severe)"
                        ],
                        "caveats": "Ascites is graded clinically (and on imaging); it is a subjective assessment. Score the underlying severity including its response to diuretics, not just what is palpable today.",
                        "source": {
                            "citation": "Pugh RNH et al. Br J Surg. 1973;60(8):646-649.",
                            "url": "https://doi.org/10.1002/bjs.1800600817"
                        },
                        "status": "draft"
                    }
                },
                "encephalopathy": {
                    "type": "string",
                    "enum": ["none", "grade1-2", "grade3-4"],
                    "description": "Hepatic encephalopathy grade (none -> 1, grade 1-2 -> 2, grade 3-4 -> 3)",
                    "definition": {
                        "concept": "Hepatic encephalopathy grade (West Haven)",
                        "statement": "None -> 1 point; West Haven grade 1-2 (or suppressed with medication such as lactulose/rifaximin) -> 2 points; grade 3-4 (or refractory) -> 3 points.",
                        "includes": [
                            "Encephalopathy controlled on medication scores 2, not 1 - the underlying grade counts, not the current treated state",
                            "Grade 1-2: mild confusion to disorientation/drowsiness; Grade 3-4: marked confusion/somnolence through to coma"
                        ],
                        "caveats": "Grading is the West Haven criteria. Score the underlying severity including its response to medication, not just the current alertness.",
                        "source": {
                            "citation": "Pugh RNH et al. Br J Surg. 1973;60(8):646-649.",
                            "url": "https://doi.org/10.1002/bjs.1800600817"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: ChildPughInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        bilirubin: f64,
        bilirubin_unit: BilirubinUnit,
        albumin: f64,
        albumin_unit: AlbuminUnit,
        inr: f64,
        ascites: Ascites,
        encephalopathy: Encephalopathy,
    ) -> ChildPughInput {
        ChildPughInput {
            bilirubin,
            bilirubin_unit,
            albumin,
            albumin_unit,
            inr,
            ascites,
            encephalopathy,
        }
    }

    /// All five parameters at their lowest (1-point) values -> score 5, class A.
    fn all_ones() -> ChildPughInput {
        input(
            1.0,
            BilirubinUnit::MgDl,
            4.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        )
    }

    #[test]
    fn minimum_score_is_class_a() {
        let o = compute(&all_ones()).unwrap();
        assert_eq!(o.score, 5);
        assert_eq!(o.class, Class::A);
    }

    #[test]
    fn maximum_score_is_class_c() {
        let i = input(
            5.0,
            BilirubinUnit::MgDl,
            2.0,
            AlbuminUnit::GDl,
            3.0,
            Ascites::ModerateSevere,
            Encephalopathy::Grade3To4,
        );
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 15);
        assert_eq!(o.class, Class::C);
    }

    #[test]
    fn worked_example_class_b() {
        // bilirubin 2.5 mg/dL -> 2; albumin 3.0 g/dL -> 2; INR 1.8 -> 2;
        // mild ascites -> 2; no encephalopathy -> 1. Total 9, class B.
        let i = input(
            2.5,
            BilirubinUnit::MgDl,
            3.0,
            AlbuminUnit::GDl,
            1.8,
            Ascites::Mild,
            Encephalopathy::None,
        );
        let o = compute(&i).unwrap();
        assert_eq!(o.bilirubin_points, 2);
        assert_eq!(o.albumin_points, 2);
        assert_eq!(o.inr_points, 2);
        assert_eq!(o.ascites_points, 2);
        assert_eq!(o.encephalopathy_points, 1);
        assert_eq!(o.score, 9);
        assert_eq!(o.class, Class::B);
    }

    #[test]
    fn class_boundaries() {
        assert_eq!(Class::from_score(5), Class::A);
        assert_eq!(Class::from_score(6), Class::A);
        assert_eq!(Class::from_score(7), Class::B);
        assert_eq!(Class::from_score(9), Class::B);
        assert_eq!(Class::from_score(10), Class::C);
        assert_eq!(Class::from_score(15), Class::C);
    }

    #[test]
    fn bilirubin_boundaries_mgdl() {
        assert_eq!(bilirubin_points(1.9), 1);
        assert_eq!(bilirubin_points(2.0), 2);
        assert_eq!(bilirubin_points(3.0), 2);
        assert_eq!(bilirubin_points(3.1), 3);
    }

    #[test]
    fn albumin_boundaries_gdl() {
        assert_eq!(albumin_points(3.6), 1);
        assert_eq!(albumin_points(3.5), 2);
        assert_eq!(albumin_points(2.8), 2);
        assert_eq!(albumin_points(2.7), 3);
    }

    #[test]
    fn inr_boundaries() {
        assert_eq!(inr_points(1.6), 1);
        assert_eq!(inr_points(1.7), 2);
        assert_eq!(inr_points(2.3), 2);
        assert_eq!(inr_points(2.4), 3);
    }

    #[test]
    fn bilirubin_unit_equivalence() {
        // 2.5 mg/dL == 42.75 umol/L; both must give the same score and sub-score.
        let mgdl = compute(&input(
            2.5,
            BilirubinUnit::MgDl,
            4.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        let umol = compute(&input(
            2.5 * BILIRUBIN_UMOL_PER_MGDL,
            BilirubinUnit::UmolL,
            4.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        assert_eq!(mgdl.bilirubin_points, 2);
        assert_eq!(mgdl.score, umol.score);
        assert_eq!(mgdl.bilirubin_points, umol.bilirubin_points);
    }

    #[test]
    fn bilirubin_umol_thresholds() {
        // SI thresholds: <34 -> 1, 34-50 -> 2, >50 -> 3.
        let low = compute(&input(
            30.0,
            BilirubinUnit::UmolL,
            4.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        assert_eq!(low.bilirubin_points, 1);
        let high = compute(&input(
            60.0,
            BilirubinUnit::UmolL,
            4.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        assert_eq!(high.bilirubin_points, 3);
    }

    #[test]
    fn albumin_unit_equivalence() {
        // 3.0 g/dL == 30 g/L; both must give the same albumin sub-score (2).
        let gdl = compute(&input(
            1.0,
            BilirubinUnit::MgDl,
            3.0,
            AlbuminUnit::GDl,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        let gl = compute(&input(
            1.0,
            BilirubinUnit::MgDl,
            30.0,
            AlbuminUnit::GL,
            1.0,
            Ascites::None,
            Encephalopathy::None,
        ))
        .unwrap();
        assert_eq!(gdl.albumin_points, 2);
        assert_eq!(gdl.score, gl.score);
        assert_eq!(gdl.albumin_points, gl.albumin_points);
    }

    #[test]
    fn rejects_bad_input() {
        let mut i = all_ones();
        i.bilirubin = 0.0;
        assert!(compute(&i).is_err());

        let mut i = all_ones();
        i.albumin = -1.0;
        assert!(compute(&i).is_err());

        let mut i = all_ones();
        i.inr = 0.0;
        assert!(compute(&i).is_err());

        let mut i = all_ones();
        i.bilirubin = f64::NAN;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "bilirubin": 2.5,
            "bilirubin_unit": "mg/dL",
            "albumin": 3.0,
            "albumin_unit": "g/dL",
            "inr": 1.8,
            "ascites": "mild",
            "encephalopathy": "none"
        });
        let typed = input(
            2.5,
            BilirubinUnit::MgDl,
            3.0,
            AlbuminUnit::GDl,
            1.8,
            Ascites::Mild,
            Encephalopathy::None,
        );
        let dynamic = ChildPugh.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(9));
    }

    #[test]
    fn schema_flags_unit_traps() {
        let schema = ChildPugh.input_schema();
        let bili = &schema["properties"]["bilirubin_unit"]["definition"];
        assert!(bili["excludes"][0].as_str().unwrap().contains("17x"));
        let alb = &schema["properties"]["albumin_unit"]["definition"];
        assert!(alb["excludes"][0].as_str().unwrap().contains("10x"));
    }

    #[test]
    fn schema_documents_treated_grades() {
        let schema = ChildPugh.input_schema();
        let ascites = &schema["properties"]["ascites"]["definition"];
        assert!(ascites["includes"][0]
            .as_str()
            .unwrap()
            .contains("Diuretic-controlled"));
        let enc = &schema["properties"]["encephalopathy"]["definition"];
        assert!(enc["includes"][0]
            .as_str()
            .unwrap()
            .contains("controlled on medication"));
    }
}
