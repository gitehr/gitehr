//! Gleason score and ISUP/WHO Grade Group for prostate cancer.
//!
//! Prostate adenocarcinoma is graded from the two most prevalent architectural
//! (Gleason) patterns seen in the specimen. The **primary** pattern is the most
//! prevalent, the **secondary** the next most prevalent; each runs 1-5, and
//! their sum is the **Gleason score** (2-10). The 2014 International Society of
//! Urological Pathology (ISUP) consensus, adopted by WHO in 2016, collapses the
//! score into five prognostic **Grade Groups** (1-5).
//!
//! The key subtlety is that Gleason score 7 splits into two Grade Groups by
//! which pattern is primary: 3+4 is Grade Group 2 (favourable intermediate
//! risk) and 4+3 is Grade Group 3 (unfavourable intermediate risk). The sum
//! alone cannot distinguish them, so the mapping is driven by the primary and
//! secondary patterns rather than the score.
//!
//! Modern reporting rarely assigns patterns below 3 in needle biopsies (a
//! benign-looking pattern 1 or 2 focus is generally not reported as carcinoma).
//! Patterns 1-5 are accepted here for completeness, but a score below 6 is
//! flagged as clinically unusual in the interpretation.
//!
//! The Grade Group mapping is a published classification (ISUP 2014 / WHO 2016),
//! implemented here from the primary literature and not subject to copyright.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "gleason";

/// Distribution licence: the ISUP 2014 / WHO 2016 Grade Group mapping is a
/// published classification, implemented here from the primary literature and
/// not subject to copyright.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - Grade Group classification implemented from the primary literature (ISUP 2014 / WHO 2016)",
    source_url: "https://pubmed.ncbi.nlm.nih.gov/26492179/",
};

/// Primary citation.
pub const REFERENCE: &str = "Epstein JI, Egevad L, Amin MB, et al. The 2014 International Society of \
Urological Pathology (ISUP) Consensus Conference on Gleason Grading of Prostatic Carcinoma: \
Definition of Grading Patterns and Proposal for a New Grading System. Am J Surg Pathol. \
2016;40(2):244-252. doi:10.1097/PAS.0000000000000530";

/// Lowest accepted Gleason pattern.
pub const PATTERN_MIN: u8 = 1;

/// Highest accepted Gleason pattern.
pub const PATTERN_MAX: u8 = 5;

/// Inputs to the Gleason grading: the two most prevalent architectural patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GleasonInput {
    /// Primary (most prevalent) Gleason pattern, 1-5.
    pub primary_pattern: u8,
    /// Secondary (next most prevalent) Gleason pattern, 1-5.
    pub secondary_pattern: u8,
}

/// ISUP/WHO Grade Group (1-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradeGroup {
    G1,
    G2,
    G3,
    G4,
    G5,
}

impl GradeGroup {
    /// Map the primary and secondary patterns to a Grade Group.
    ///
    /// Score 7 is resolved by which pattern is primary (3+4 -> G2, 4+3 -> G3),
    /// so this takes the patterns rather than just their sum.
    fn from_patterns(primary: u8, secondary: u8) -> Self {
        let score = primary + secondary;
        if score <= 6 {
            GradeGroup::G1
        } else if score == 7 {
            // 3+4 is favourable (G2); 4+3 is unfavourable (G3).
            if primary <= 3 {
                GradeGroup::G2
            } else {
                GradeGroup::G3
            }
        } else if score == 8 {
            GradeGroup::G4
        } else {
            // Score 9 or 10.
            GradeGroup::G5
        }
    }

    /// Numeric label, 1-5.
    fn number(self) -> u8 {
        match self {
            GradeGroup::G1 => 1,
            GradeGroup::G2 => 2,
            GradeGroup::G3 => 3,
            GradeGroup::G4 => 4,
            GradeGroup::G5 => 5,
        }
    }

    /// Short risk descriptor for the Grade Group.
    fn descriptor(self) -> &'static str {
        match self {
            GradeGroup::G1 => "low risk",
            GradeGroup::G2 => "favourable intermediate risk",
            GradeGroup::G3 => "unfavourable intermediate risk",
            GradeGroup::G4 => "high risk",
            GradeGroup::G5 => "very high risk",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GleasonOutcome {
    /// Gleason score (primary + secondary), 2-10.
    pub gleason_score: u8,
    pub grade_group: GradeGroup,
    pub interpretation: String,
}

/// Pure mapping: patterns -> Gleason score and Grade Group.
pub fn compute(input: &GleasonInput) -> Result<GleasonOutcome, CalcError> {
    for (label, p) in [
        ("primary_pattern", input.primary_pattern),
        ("secondary_pattern", input.secondary_pattern),
    ] {
        if !(PATTERN_MIN..=PATTERN_MAX).contains(&p) {
            return Err(CalcError::InvalidInput(format!(
                "{label} must be between {PATTERN_MIN} and {PATTERN_MAX}, got {p}"
            )));
        }
    }

    let gleason_score = input.primary_pattern + input.secondary_pattern;
    let grade_group = GradeGroup::from_patterns(input.primary_pattern, input.secondary_pattern);

    let unusual = if gleason_score < 6 {
        " A Gleason score below 6 is rarely reported in modern practice (patterns below 3 are \
generally not graded as carcinoma in needle biopsies); confirm the patterns are correct."
    } else {
        ""
    };

    let interpretation = format!(
        "Gleason {}+{} = {gleason_score}, ISUP/WHO Grade Group {} ({}).{unusual} Risk \
descriptors follow the Grade Group; formal risk stratification (for example the Cambridge \
Prognostic Group used by NICE NG131) also requires PSA and clinical/tumour stage.",
        input.primary_pattern,
        input.secondary_pattern,
        grade_group.number(),
        grade_group.descriptor(),
    );

    Ok(GleasonOutcome {
        gleason_score,
        grade_group,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &GleasonInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("gleason_score".into(), json!(o.gleason_score));
    working.insert("grade_group".into(), json!(o.grade_group.number()));
    working.insert("risk".into(), json!(o.grade_group.descriptor()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(format!("Grade Group {}", o.grade_group.number())),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Gleason;

impl Calculator for Gleason {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Gleason Grade Group (ISUP/WHO)"
    }

    fn description(&self) -> &'static str {
        "Gleason score and ISUP/WHO Grade Group (1-5) from the primary and secondary prostate cancer patterns."
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
            "title": "GleasonInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["primary_pattern", "secondary_pattern"],
            "properties": {
                "primary_pattern": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 5,
                    "description": "Primary (most prevalent) Gleason pattern, 1-5",
                    "definition": {
                        "concept": "Primary Gleason pattern",
                        "statement": "The architectural Gleason pattern (1-5) that is most prevalent in the specimen. It is written first in the Gleason score (the '3' in 3+4).",
                        "caveats": "Patterns below 3 are rarely assigned to carcinoma in modern needle biopsies.",
                        "excludes": [
                            "For score 7, the primary pattern distinguishes Grade Group 2 (3+4) from Grade Group 3 (4+3); do NOT swap primary and secondary"
                        ],
                        "source": {
                            "citation": "Epstein JI et al. Am J Surg Pathol. 2016;40(2):244-252.",
                            "url": "https://pubmed.ncbi.nlm.nih.gov/26492179/"
                        },
                        "status": "draft"
                    }
                },
                "secondary_pattern": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 5,
                    "description": "Secondary (next most prevalent) Gleason pattern, 1-5",
                    "definition": {
                        "concept": "Secondary Gleason pattern",
                        "statement": "The architectural Gleason pattern (1-5) that is the next most prevalent after the primary. It is written second in the Gleason score (the '4' in 3+4).",
                        "caveats": "Patterns below 3 are rarely assigned to carcinoma in modern needle biopsies.",
                        "source": {
                            "citation": "Epstein JI et al. Am J Surg Pathol. 2016;40(2):244-252.",
                            "url": "https://pubmed.ncbi.nlm.nih.gov/26492179/"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: GleasonInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(primary: u8, secondary: u8) -> GleasonInput {
        GleasonInput {
            primary_pattern: primary,
            secondary_pattern: secondary,
        }
    }

    #[test]
    fn three_plus_four_is_grade_group_2() {
        let o = compute(&input(3, 4)).unwrap();
        assert_eq!(o.gleason_score, 7);
        assert_eq!(o.grade_group, GradeGroup::G2);
    }

    #[test]
    fn four_plus_three_is_grade_group_3() {
        let o = compute(&input(4, 3)).unwrap();
        assert_eq!(o.gleason_score, 7);
        assert_eq!(o.grade_group, GradeGroup::G3);
    }

    #[test]
    fn score_seven_splits_only_by_primary_pattern() {
        // Same sum, different Grade Group: the subtlety the calculator exists for.
        let favourable = compute(&input(3, 4)).unwrap();
        let unfavourable = compute(&input(4, 3)).unwrap();
        assert_eq!(favourable.gleason_score, unfavourable.gleason_score);
        assert_ne!(favourable.grade_group, unfavourable.grade_group);
    }

    #[test]
    fn score_six_is_grade_group_1() {
        let o = compute(&input(3, 3)).unwrap();
        assert_eq!(o.gleason_score, 6);
        assert_eq!(o.grade_group, GradeGroup::G1);
    }

    #[test]
    fn low_score_is_grade_group_1_and_flagged_unusual() {
        let o = compute(&input(2, 3)).unwrap();
        assert_eq!(o.gleason_score, 5);
        assert_eq!(o.grade_group, GradeGroup::G1);
        assert!(o.interpretation.contains("rarely reported"));
    }

    #[test]
    fn score_eight_is_grade_group_4() {
        for (p, s) in [(4, 4), (3, 5), (5, 3)] {
            let o = compute(&input(p, s)).unwrap();
            assert_eq!(o.gleason_score, 8, "{p}+{s}");
            assert_eq!(o.grade_group, GradeGroup::G4, "{p}+{s}");
        }
    }

    #[test]
    fn score_nine_and_ten_are_grade_group_5() {
        let nine = compute(&input(4, 5)).unwrap();
        assert_eq!(nine.gleason_score, 9);
        assert_eq!(nine.grade_group, GradeGroup::G5);

        let ten = compute(&input(5, 5)).unwrap();
        assert_eq!(ten.gleason_score, 10);
        assert_eq!(ten.grade_group, GradeGroup::G5);
    }

    #[test]
    fn rejects_out_of_range_patterns() {
        assert!(compute(&input(0, 4)).is_err());
        assert!(compute(&input(4, 0)).is_err());
        assert!(compute(&input(6, 4)).is_err());
        assert!(compute(&input(4, 6)).is_err());
    }

    #[test]
    fn grade_group_numbers_and_descriptors() {
        assert_eq!(GradeGroup::G1.number(), 1);
        assert_eq!(GradeGroup::G5.number(), 5);
        assert_eq!(GradeGroup::G2.descriptor(), "favourable intermediate risk");
        assert_eq!(GradeGroup::G3.descriptor(), "unfavourable intermediate risk");
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "primary_pattern": 3, "secondary_pattern": 4 });
        let dynamic = Gleason.calculate(&value).unwrap();
        let typed = build_response(&input(3, 4)).unwrap();
        assert_eq!(dynamic, typed);
    }

    #[test]
    fn schema_flags_primary_secondary_exclusion() {
        let schema = Gleason.input_schema();
        let def = &schema["properties"]["primary_pattern"]["definition"];
        assert!(def["excludes"][0]
            .as_str()
            .unwrap()
            .contains("do NOT swap"));
    }
}
