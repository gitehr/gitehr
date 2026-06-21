//! NPI - Nottingham Prognostic Index, prognosis in primary operable breast cancer.
//!
//! A linear index combining three pathological factors of the resected primary
//! tumour: invasive tumour size, lymph node stage, and histological grade.
//!
//!   NPI = (0.2 x tumour_size_cm) + lymph_node_stage + histological_grade
//!
//! where tumour size is the largest diameter of the invasive tumour in cm,
//! lymph node stage is 1 (node-negative), 2 (1-3 nodes involved), or 3 (>=4
//! nodes involved), and histological grade is the Nottingham/Bloom-Richardson
//! grade 1, 2, or 3. The index stratifies patients into prognostic groups with
//! markedly different survival.
//!
//! Two non-obvious points are encoded here. First, lymph node *stage* is a
//! 1-3 ordinal coding of the node count, not the raw number of positive nodes,
//! so the count must be translated before it is passed in. Second, the original
//! three-group scheme (good/moderate/poor) was later refined into a five-group
//! scheme that splits the moderate band into Moderate I and Moderate II; this
//! implementation reports the five-group classification, whose boundaries
//! (2.4, 3.4, 4.4, 5.4) are the widely cited cut-points.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "npi";

/// Distribution licence: the NPI is a published clinical method, implemented
/// here from the primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature (Nottingham Prognostic Index)",
    source_url: "https://doi.org/10.1007/BF01840834",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Galea MH, Blamey RW, Elston CE, Ellis IO. The Nottingham Prognostic Index in primary breast \
cancer. Breast Cancer Res Treat. 1992;22(3):207-219. doi:10.1007/BF01840834";

/// Weight applied to the invasive tumour size (cm) in the index.
pub const SIZE_WEIGHT: f64 = 0.2;

/// Inputs to the Nottingham Prognostic Index.
///
/// `node_stage` and `grade` are the ordinal 1-3 codings, not raw counts; see the
/// schema `definition` blocks for the mapping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NpiInput {
    /// Largest diameter of the invasive tumour, in centimetres.
    pub tumour_size_cm: f64,
    /// Lymph node stage: 1 = node-negative, 2 = 1-3 nodes involved,
    /// 3 = >=4 nodes involved.
    pub node_stage: u8,
    /// Histological (Nottingham/Bloom-Richardson) grade: 1, 2, or 3.
    pub grade: u8,
}

/// Five-group prognostic classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrognosticGroup {
    /// Excellent prognosis: NPI <= 2.4.
    Excellent,
    /// Good prognosis: 2.4 < NPI <= 3.4.
    Good,
    /// Moderate I prognosis: 3.4 < NPI <= 4.4.
    ModerateI,
    /// Moderate II prognosis: 4.4 < NPI <= 5.4.
    ModerateII,
    /// Poor prognosis: NPI > 5.4.
    Poor,
}

impl PrognosticGroup {
    /// Classify an NPI value into the five-group scheme.
    ///
    /// Cut-points are 2.4, 3.4, 4.4, 5.4; each boundary value falls in the lower
    /// (better) band (i.e. the upper bound is inclusive).
    fn from_npi(npi: f64) -> Self {
        if npi <= 2.4 {
            PrognosticGroup::Excellent
        } else if npi <= 3.4 {
            PrognosticGroup::Good
        } else if npi <= 4.4 {
            PrognosticGroup::ModerateI
        } else if npi <= 5.4 {
            PrognosticGroup::ModerateII
        } else {
            PrognosticGroup::Poor
        }
    }

    fn slug(self) -> &'static str {
        match self {
            PrognosticGroup::Excellent => "excellent",
            PrognosticGroup::Good => "good",
            PrognosticGroup::ModerateI => "moderate-1",
            PrognosticGroup::ModerateII => "moderate-2",
            PrognosticGroup::Poor => "poor",
        }
    }

    fn label(self) -> &'static str {
        match self {
            PrognosticGroup::Excellent => "Excellent",
            PrognosticGroup::Good => "Good",
            PrognosticGroup::ModerateI => "Moderate I",
            PrognosticGroup::ModerateII => "Moderate II",
            PrognosticGroup::Poor => "Poor",
        }
    }

    /// Indicative 5-year survival, as reported in the original Nottingham series.
    fn five_year_survival(self) -> &'static str {
        match self {
            PrognosticGroup::Excellent => "~93%",
            PrognosticGroup::Good => "~85%",
            PrognosticGroup::ModerateI => "~70%",
            PrognosticGroup::ModerateII => "~70%",
            PrognosticGroup::Poor => "~50%",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct NpiOutcome {
    /// The NPI value, rounded to 2 decimal places.
    pub npi: f64,
    pub group: PrognosticGroup,
    pub interpretation: String,
}

/// Round to 2 decimal places (the precision the index is reported to).
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Pure scoring: the Nottingham Prognostic Index.
pub fn compute(input: &NpiInput) -> Result<NpiOutcome, CalcError> {
    if !input.tumour_size_cm.is_finite() || input.tumour_size_cm <= 0.0 {
        return Err(CalcError::InvalidInput(
            "tumour_size_cm must be a positive number".into(),
        ));
    }
    if !(1..=3).contains(&input.node_stage) {
        return Err(CalcError::InvalidInput(
            "node_stage must be 1 (node-negative), 2 (1-3 nodes), or 3 (>=4 nodes)".into(),
        ));
    }
    if !(1..=3).contains(&input.grade) {
        return Err(CalcError::InvalidInput(
            "grade must be 1, 2, or 3".into(),
        ));
    }

    let raw = SIZE_WEIGHT * input.tumour_size_cm + input.node_stage as f64 + input.grade as f64;
    let npi = round2(raw);
    let group = PrognosticGroup::from_npi(npi);

    let interpretation = format!(
        "NPI {npi:.2}: {} prognostic group (indicative 5-year survival {}). NPI = (0.2 x size_cm) \
+ node stage + grade. The five-group scheme uses cut-points 2.4, 3.4, 4.4, 5.4. The index applies \
to primary operable invasive breast cancer and informs prognosis, not treatment selection on its \
own; survival figures are historical cohort estimates and modern tools (for example PREDICT) \
incorporate further factors such as receptor status and adjuvant therapy.",
        group.label(),
        group.five_year_survival()
    );

    Ok(NpiOutcome {
        npi,
        group,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &NpiInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("npi".into(), json!(o.npi));
    working.insert("prognostic_group".into(), json!(o.group.slug()));
    working.insert("size_component".into(), json!(round2(SIZE_WEIGHT * input.tumour_size_cm)));
    working.insert("node_stage".into(), json!(input.node_stage));
    working.insert("grade".into(), json!(input.grade));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.npi),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Npi;

impl Calculator for Npi {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Nottingham Prognostic Index (NPI)"
    }

    fn description(&self) -> &'static str {
        "Prognosis in primary operable breast cancer from invasive tumour size, lymph node stage, and histological grade; reports the prognostic group."
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
            "title": "NpiInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["tumour_size_cm", "node_stage", "grade"],
            "properties": {
                "tumour_size_cm": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Largest diameter of the invasive tumour, in centimetres",
                    "definition": {
                        "concept": "Invasive tumour size",
                        "statement": "The maximum diameter of the invasive component of the tumour, in centimetres. The index weights this by 0.2.",
                        "caveats": "Use centimetres, not millimetres - a 22 mm tumour is 2.2 cm. Use the invasive size, not the whole-lesion size including in-situ disease.",
                        "source": {
                            "citation": "Galea MH et al. Breast Cancer Res Treat. 1992;22(3):207-219.",
                            "url": "https://doi.org/10.1007/BF01840834"
                        },
                        "status": "draft"
                    }
                },
                "node_stage": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 3,
                    "description": "Lymph node stage: 1 = node-negative, 2 = 1-3 nodes involved, 3 = >=4 nodes involved",
                    "definition": {
                        "concept": "Lymph node stage (NPI coding)",
                        "statement": "An ordinal 1-3 coding of nodal involvement: 1 = no positive nodes; 2 = 1 to 3 positive nodes; 3 = 4 or more positive nodes.",
                        "excludes": [
                            "Do NOT pass the raw count of positive nodes: 7 positive nodes is stage 3, not 7. The count must be translated to the 1-3 stage first"
                        ],
                        "source": {
                            "citation": "Galea MH et al. Breast Cancer Res Treat. 1992;22(3):207-219.",
                            "url": "https://doi.org/10.1007/BF01840834"
                        },
                        "status": "draft"
                    }
                },
                "grade": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 3,
                    "description": "Histological (Nottingham/Bloom-Richardson) grade: 1, 2, or 3",
                    "definition": {
                        "concept": "Histological grade",
                        "statement": "The Nottingham modification of the Bloom-Richardson grade: 1 = well-differentiated (low grade); 2 = moderately differentiated; 3 = poorly differentiated (high grade).",
                        "source": {
                            "citation": "Galea MH et al. Breast Cancer Res Treat. 1992;22(3):207-219.",
                            "url": "https://doi.org/10.1007/BF01840834"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: NpiInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(tumour_size_cm: f64, node_stage: u8, grade: u8) -> NpiInput {
        NpiInput {
            tumour_size_cm,
            node_stage,
            grade,
        }
    }

    #[test]
    fn worked_example() {
        // 2.0 cm, node stage 2, grade 2: 0.2*2 + 2 + 2 = 4.4 -> Moderate I (boundary).
        let o = compute(&input(2.0, 2, 2)).unwrap();
        assert_eq!(o.npi, 4.4);
        assert_eq!(o.group, PrognosticGroup::ModerateI);
    }

    #[test]
    fn small_node_negative_grade1_is_excellent() {
        // 1.0 cm, node-negative, grade 1: 0.2 + 1 + 1 = 2.2 -> Excellent.
        let o = compute(&input(1.0, 1, 1)).unwrap();
        assert_eq!(o.npi, 2.2);
        assert_eq!(o.group, PrognosticGroup::Excellent);
    }

    #[test]
    fn large_high_grade_node_positive_is_poor() {
        // 5.0 cm, node stage 3, grade 3: 1.0 + 3 + 3 = 7.0 -> Poor.
        let o = compute(&input(5.0, 3, 3)).unwrap();
        assert_eq!(o.npi, 7.0);
        assert_eq!(o.group, PrognosticGroup::Poor);
    }

    #[test]
    fn rounds_to_two_dp() {
        // 3.3 cm: 0.66 + 2 + 2 = 4.66 -> Moderate II.
        let o = compute(&input(3.3, 2, 2)).unwrap();
        assert_eq!(o.npi, 4.66);
        assert_eq!(o.group, PrognosticGroup::ModerateII);
    }

    #[test]
    fn group_boundaries_are_inclusive_upper() {
        // Each cut-point value falls in the lower (better) band.
        assert_eq!(PrognosticGroup::from_npi(2.4), PrognosticGroup::Excellent);
        assert_eq!(PrognosticGroup::from_npi(2.41), PrognosticGroup::Good);
        assert_eq!(PrognosticGroup::from_npi(3.4), PrognosticGroup::Good);
        assert_eq!(PrognosticGroup::from_npi(3.41), PrognosticGroup::ModerateI);
        assert_eq!(PrognosticGroup::from_npi(4.4), PrognosticGroup::ModerateI);
        assert_eq!(PrognosticGroup::from_npi(4.41), PrognosticGroup::ModerateII);
        assert_eq!(PrognosticGroup::from_npi(5.4), PrognosticGroup::ModerateII);
        assert_eq!(PrognosticGroup::from_npi(5.41), PrognosticGroup::Poor);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(compute(&input(0.0, 1, 1)).is_err());
        assert!(compute(&input(-1.0, 1, 1)).is_err());
        assert!(compute(&input(f64::NAN, 1, 1)).is_err());
        assert!(compute(&input(2.0, 0, 1)).is_err());
        assert!(compute(&input(2.0, 4, 1)).is_err());
        assert!(compute(&input(2.0, 1, 0)).is_err());
        assert!(compute(&input(2.0, 1, 4)).is_err());
    }

    #[test]
    fn working_records_components() {
        let r = build_response(&input(2.0, 2, 2)).unwrap();
        assert_eq!(r.working["npi"], json!(4.4));
        assert_eq!(r.working["prognostic_group"], json!("moderate-1"));
        assert_eq!(r.working["size_component"], json!(0.4));
        assert_eq!(r.working["node_stage"], json!(2));
        assert_eq!(r.working["grade"], json!(2));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "tumour_size_cm": 2.0, "node_stage": 2, "grade": 2 });
        let dynamic = Npi.calculate(&value).unwrap();
        let typed = build_response(&input(2.0, 2, 2)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(4.4));
    }

    #[test]
    fn schema_flags_node_count_trap() {
        let schema = Npi.input_schema();
        let def = &schema["properties"]["node_stage"]["definition"];
        assert!(def["excludes"][0]
            .as_str()
            .unwrap()
            .contains("raw count"));
    }
}
