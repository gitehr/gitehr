//! AUDIT-C - Alcohol Use Disorders Identification Test (Consumption).
//!
//! AUDIT-C is the three-item alcohol consumption subscale of the WHO AUDIT: the
//! first three questions of the full ten-item instrument. Each item is scored
//! 0-4 and the three are summed to a 0-12 total. It identifies hazardous
//! drinking and possible alcohol use disorders.
//!
//! The originally validated, sex-specific cut-points (Bush et al. 1998 and the
//! 2003 validation in women) are a total of 4 or more in men and 3 or more in
//! women; this is what we encode. Because the positive threshold depends on sex,
//! the input requires a `sex` field. A higher unisex cut-point of 5 is used by
//! some services (notably the US VA/DoD) to reduce false positives; we surface
//! that in the interpretation but key the primary "screen positive" flag off the
//! validated sex-specific thresholds.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "auditc";

/// Distribution licence: AUDIT is a WHO instrument. The WHO AUDIT manual (Babor
/// et al., WHO/MSD/MSB/01.6a, 2nd ed. 2001) states the document "may be freely
/// reviewed, abstracted, reproduced and translated, in part or in whole, but
/// not for sale or for use in conjunction with commercial purposes". No
/// permission is required for non-commercial reproduction.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "WHO instrument - may be freely reviewed, abstracted, reproduced and translated, in part or in whole, but not for sale or for commercial purposes (WHO AUDIT manual, Babor et al. 2001)",
    source_url: "https://iris.who.int/handle/10665/67205",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Bush K, Kivlahan DR, McDonell MB, Fihn SD, Bradley KA. The AUDIT alcohol consumption \
questions (AUDIT-C): an effective brief screening test for problem drinking. Arch Intern Med. \
1998;158(16):1789-1795. doi:10.1001/archinte.158.16.1789";

/// Number of items.
pub const ITEM_COUNT: usize = 3;

/// Maximum score per item, and minimum/maximum totals.
pub const MAX_ITEM_SCORE: u8 = 4;

/// Validated positive cut-point in men (total >= 4).
pub const THRESHOLD_MALE: u16 = 4;

/// Validated positive cut-point in women (total >= 3).
pub const THRESHOLD_FEMALE: u16 = 3;

/// Higher unisex cut-point used by some services (e.g. US VA/DoD) to reduce
/// false positives. Surfaced in the interpretation only.
pub const THRESHOLD_HIGHER_SPECIFICITY: u16 = 5;

/// Patient sex, which selects the validated AUDIT-C positive cut-point.
///
/// AUDIT-C was validated separately in men and women and the positive threshold
/// differs (4 vs 3), so this is a required input rather than a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

impl Sex {
    /// The validated positive cut-point for this sex.
    pub fn threshold(self) -> u16 {
        match self {
            Sex::Male => THRESHOLD_MALE,
            Sex::Female => THRESHOLD_FEMALE,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Sex::Male => "male",
            Sex::Female => "female",
        }
    }
}

/// The three AUDIT-C responses, each 0-4, in question order Q1-Q3, plus the
/// patient sex used to select the validated positive cut-point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCInput {
    /// Q1-Q3 responses, each 0-4 (see [`input_schema`](AuditC::input_schema)).
    pub responses: Vec<u8>,
    /// Patient sex, selecting the 4 (male) or 3 (female) positive cut-point.
    pub sex: Sex,
}

/// Risk band implied by the total score.
///
/// AUDIT-C is a graded consumption measure; higher scores indicate a greater
/// likelihood and severity of hazardous drinking. The bands below are a common
/// pragmatic grouping (low / increasing / higher / possible dependence). The
/// formal "screen positive" decision is the sex-specific cut-point, captured
/// separately as [`AuditCOutcome::screen_positive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Total 0-2: lower risk.
    Lower,
    /// Total 3-4: increasing risk.
    Increasing,
    /// Total 5-7: higher risk.
    Higher,
    /// Total 8-12: possible dependence; warrants fuller assessment.
    PossibleDependence,
}

impl RiskBand {
    /// Pragmatic AUDIT-C risk bands by total score.
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=2 => RiskBand::Lower,
            3..=4 => RiskBand::Increasing,
            5..=7 => RiskBand::Higher,
            _ => RiskBand::PossibleDependence,
        }
    }

    fn label(self) -> &'static str {
        match self {
            RiskBand::Lower => "lower risk",
            RiskBand::Increasing => "increasing risk",
            RiskBand::Higher => "higher risk",
            RiskBand::PossibleDependence => "possible dependence",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCOutcome {
    /// Total consumption score (0-12).
    pub total: u16,
    /// Pragmatic risk band for the total.
    pub band: RiskBand,
    /// The validated positive cut-point applied (4 male / 3 female).
    pub threshold: u16,
    /// True if total >= the sex-specific validated cut-point.
    pub screen_positive: bool,
    /// True if total >= 5, the higher-specificity unisex cut-point.
    pub above_higher_specificity_threshold: bool,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &AuditCInput) -> Result<AuditCOutcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }
    for (i, &v) in input.responses.iter().enumerate() {
        if v > MAX_ITEM_SCORE {
            return Err(CalcError::InvalidInput(format!(
                "response {} = {v} is out of range 0-{MAX_ITEM_SCORE}",
                i + 1
            )));
        }
    }

    let total: u16 = input.responses.iter().map(|&v| v as u16).sum();
    let band = RiskBand::from_total(total);
    let threshold = input.sex.threshold();
    let screen_positive = total >= threshold;
    let above_higher_specificity_threshold = total >= THRESHOLD_HIGHER_SPECIFICITY;

    let mut interpretation = format!(
        "Total score {total}/12 indicates {} ({}).",
        band.label(),
        input.sex.label()
    );
    if screen_positive {
        interpretation.push_str(&format!(
            " At or above the validated cut-point of {threshold} for {} patients; \
the screen is positive for hazardous drinking or a possible alcohol use disorder, \
and warrants further assessment.",
            input.sex.label()
        ));
    } else {
        interpretation.push_str(&format!(
            " Below the validated cut-point of {threshold} for {} patients; \
the screen is negative.",
            input.sex.label()
        ));
    }
    if above_higher_specificity_threshold {
        interpretation.push_str(
            " Also at or above the higher-specificity unisex cut-point of 5 used by some services.",
        );
    }
    interpretation
        .push_str(" AUDIT-C is a screen for consumption-related risk; it is not a diagnosis.");

    Ok(AuditCOutcome {
        total,
        band,
        threshold,
        screen_positive,
        above_higher_specificity_threshold,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &AuditCInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("risk_band".into(), json!(o.band.label()));
    working.insert("sex".into(), json!(input.sex.label()));
    working.insert("threshold".into(), json!(o.threshold));
    working.insert("screen_positive".into(), json!(o.screen_positive));
    working.insert(
        "above_higher_specificity_threshold".into(),
        json!(o.above_higher_specificity_threshold),
    );
    working.insert("answers".into(), json!(input.responses));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct AuditC;

impl Calculator for AuditC {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "AUDIT-C Alcohol Consumption Screen"
    }

    fn description(&self) -> &'static str {
        "Three-item WHO AUDIT consumption subscale (0-12); positive at 4+ (men) or 3+ (women)."
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
            "title": "AuditCInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses", "sex"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Three responses (Q1-Q3), each scored 0-4. Q1 How often do you have a drink containing alcohol? (0 Never, 1 Monthly or less, 2 2-4 times a month, 3 2-3 times a week, 4 4+ times a week). Q2 How many standard drinks containing alcohol do you have on a typical day when drinking? (0 1-2, 1 3-4, 2 5-6, 3 7-9, 4 10+). Q3 How often do you have six or more drinks on one occasion? (0 Never, 1 Less than monthly, 2 Monthly, 3 Weekly, 4 Daily or almost daily).",
                    "items": { "type": "integer", "minimum": 0, "maximum": 4 },
                    "minItems": 3,
                    "maxItems": 3,
                    "definition": {
                        "concept": "AUDIT-C consumption item responses",
                        "statement": "The first three AUDIT items measuring frequency, typical quantity, and frequency of heavy episodic (binge) drinking.",
                        "includes": [
                            "Q1 Frequency of drinking",
                            "Q2 Typical quantity per drinking day",
                            "Q3 Frequency of 6+ drinks on one occasion"
                        ],
                        "caveats": "Validated positive cut-points are 4+ (men) and 3+ (women); a higher unisex cut-point of 5 is used by some services to improve specificity. AUDIT-C screens for hazardous consumption and is not a diagnosis.",
                        "source": {
                            "citation": "Bush K, et al. Arch Intern Med. 1998;158(16):1789-1795.",
                            "url": "https://doi.org/10.1001/archinte.158.16.1789"
                        },
                        "status": "draft"
                    }
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Patient sex, selecting the validated positive cut-point (4 for male, 3 for female).",
                    "definition": {
                        "concept": "Sex used to select AUDIT-C cut-point",
                        "statement": "AUDIT-C was validated separately in men and women; the positive threshold is sex-specific.",
                        "source": {
                            "citation": "Bradley KA, et al. Alcohol Clin Exp Res. 2007;31(7):1208-1217 (validation in women).",
                            "url": "https://doi.org/10.1111/j.1530-0277.2007.00403.x"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AuditCInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(v: [u8; 3], sex: Sex) -> AuditCInput {
        AuditCInput {
            responses: v.to_vec(),
            sex,
        }
    }

    #[test]
    fn all_zero_is_lower_risk_and_negative() {
        let o = compute(&input([0, 0, 0], Sex::Male)).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.band, RiskBand::Lower);
        assert!(!o.screen_positive);
        assert!(!o.above_higher_specificity_threshold);
    }

    #[test]
    fn worked_example_higher_risk() {
        // Q1=3 (2-3 times a week), Q2=2 (5-6 drinks), Q3=2 (monthly) -> total 7.
        let o = compute(&input([3, 2, 2], Sex::Male)).unwrap();
        assert_eq!(o.total, 7);
        assert_eq!(o.band, RiskBand::Higher);
        assert!(o.screen_positive);
        assert!(o.above_higher_specificity_threshold);
    }

    #[test]
    fn band_boundaries() {
        assert_eq!(RiskBand::from_total(2), RiskBand::Lower);
        assert_eq!(RiskBand::from_total(3), RiskBand::Increasing);
        assert_eq!(RiskBand::from_total(4), RiskBand::Increasing);
        assert_eq!(RiskBand::from_total(5), RiskBand::Higher);
        assert_eq!(RiskBand::from_total(7), RiskBand::Higher);
        assert_eq!(RiskBand::from_total(8), RiskBand::PossibleDependence);
        assert_eq!(RiskBand::from_total(12), RiskBand::PossibleDependence);
    }

    #[test]
    fn sex_specific_thresholds_at_boundary() {
        // Total of 3: positive for women (>=3), negative for men (>=4).
        let women = compute(&input([1, 1, 1], Sex::Female)).unwrap();
        assert_eq!(women.total, 3);
        assert_eq!(women.threshold, THRESHOLD_FEMALE);
        assert!(women.screen_positive);

        let men = compute(&input([1, 1, 1], Sex::Male)).unwrap();
        assert_eq!(men.total, 3);
        assert_eq!(men.threshold, THRESHOLD_MALE);
        assert!(!men.screen_positive);

        // Total of 4: now positive for men too.
        let men4 = compute(&input([2, 1, 1], Sex::Male)).unwrap();
        assert_eq!(men4.total, 4);
        assert!(men4.screen_positive);
    }

    #[test]
    fn maximum_score_is_possible_dependence() {
        let o = compute(&input([4, 4, 4], Sex::Female)).unwrap();
        assert_eq!(o.total, 12);
        assert_eq!(o.band, RiskBand::PossibleDependence);
        assert!(o.screen_positive);
        assert!(o.above_higher_specificity_threshold);
    }

    #[test]
    fn wrong_length_and_range_are_rejected() {
        assert!(compute(&AuditCInput {
            responses: vec![0; 2],
            sex: Sex::Male
        })
        .is_err());
        assert!(compute(&AuditCInput {
            responses: vec![0; 4],
            sex: Sex::Male
        })
        .is_err());
        assert!(compute(&AuditCInput {
            responses: vec![5, 0, 0],
            sex: Sex::Female
        })
        .is_err());
    }

    #[test]
    fn missing_sex_is_rejected_by_dynamic_calculate() {
        // sex is required; omitting it must fail to deserialize.
        let err = AuditC.calculate(&json!({ "responses": [1, 1, 1] }));
        assert!(err.is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [3, 2, 2];
        let dynamic = AuditC
            .calculate(&json!({ "responses": arr, "sex": "male" }))
            .unwrap();
        let typed = build_response(&input(arr, Sex::Male)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(7));
        assert_eq!(dynamic.working["screen_positive"], json!(true));
        assert_eq!(dynamic.working["threshold"], json!(4));
    }
}
