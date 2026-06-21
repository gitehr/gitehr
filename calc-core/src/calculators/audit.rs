//! AUDIT - Alcohol Use Disorders Identification Test (full 10-item screen).
//!
//! Ten items developed by the WHO collaborative project. Items 1-8 are rated
//! 0-4; items 9-10 have only three response options scored 0, 2, or 4. The item
//! scores are summed to a 0-40 total and read against the four WHO risk zones
//! (Babor et al., 2001): 0-7 low risk, 8-15 increasing/hazardous, 16-19 higher
//! risk/harmful, 20-40 possible dependence.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "audit";

/// Distribution licence: as a WHO-approved instrument the AUDIT is in the public
/// domain; no permission is needed for any non-commercial use, and it may be
/// reproduced provided it is not materially changed and is noted as a
/// WHO-approved instrument.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public domain - WHO-approved instrument; no permission required for non-commercial use, reproduce unaltered with a note that it is a WHO-approved instrument",
    source_url: "https://auditscreen.org/about/faqs",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Saunders JB, Aasland OG, Babor TF, de la Fuente JR, Grant M. Development of the Alcohol Use \
Disorders Identification Test (AUDIT): WHO Collaborative Project on Early Detection of Persons with \
Harmful Alcohol Consumption-II. Addiction. 1993;88(6):791-804. doi:10.1111/j.1360-0443.1993.tb02093.x. \
Scoring/interpretation per Babor TF, Higgins-Biddle JC, Saunders JB, Monteiro MG. AUDIT: Guidelines \
for Use in Primary Care, 2nd ed. Geneva: WHO; 2001.";

/// Number of items.
pub const ITEM_COUNT: usize = 10;

/// The ten AUDIT responses, in question order Q1-Q10. Items 1-8 range 0-4;
/// items 9-10 take only 0, 2, or 4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditInput {
    pub responses: Vec<u8>,
}

/// WHO risk zone implied by the total score (Babor et al., 2001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskZone {
    /// Zone I (0-7): low-risk consumption.
    LowRisk,
    /// Zone II (8-15): increasing/hazardous use.
    Hazardous,
    /// Zone III (16-19): higher-risk/harmful use.
    Harmful,
    /// Zone IV (20-40): possible dependence.
    PossibleDependence,
}

impl RiskZone {
    /// WHO four-zone bands (Babor et al., 2001).
    pub fn from_total(total: u16) -> Self {
        match total {
            0..=7 => RiskZone::LowRisk,
            8..=15 => RiskZone::Hazardous,
            16..=19 => RiskZone::Harmful,
            _ => RiskZone::PossibleDependence,
        }
    }

    fn label(self) -> &'static str {
        match self {
            RiskZone::LowRisk => "low risk",
            RiskZone::Hazardous => "increasing/hazardous use",
            RiskZone::Harmful => "higher-risk/harmful use",
            RiskZone::PossibleDependence => "possible dependence",
        }
    }

    fn zone(self) -> &'static str {
        match self {
            RiskZone::LowRisk => "Zone I",
            RiskZone::Hazardous => "Zone II",
            RiskZone::Harmful => "Zone III",
            RiskZone::PossibleDependence => "Zone IV",
        }
    }

    fn advice(self) -> &'static str {
        match self {
            RiskZone::LowRisk => "Alcohol education.",
            RiskZone::Hazardous => "Simple advice (brief intervention).",
            RiskZone::Harmful => {
                "Brief intervention plus continued monitoring; consider diagnostic evaluation."
            }
            RiskZone::PossibleDependence => {
                "Referral to a specialist for diagnostic evaluation and treatment."
            }
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditOutcome {
    /// Total score (0-40).
    pub total: u16,
    pub zone: RiskZone,
    pub interpretation: String,
}

/// The highest allowed value for a given item index (0-based). Items 1-8 allow
/// 0-4; items 9-10 (indices 8 and 9) allow only 0, 2, or 4.
fn item_is_valid(index: usize, value: u8) -> bool {
    if index >= 8 {
        matches!(value, 0 | 2 | 4)
    } else {
        value <= 4
    }
}

/// Pure scoring.
pub fn compute(input: &AuditInput) -> Result<AuditOutcome, CalcError> {
    if input.responses.len() != ITEM_COUNT {
        return Err(CalcError::InvalidInput(format!(
            "expected {ITEM_COUNT} responses, got {}",
            input.responses.len()
        )));
    }
    for (i, &v) in input.responses.iter().enumerate() {
        if !item_is_valid(i, v) {
            let allowed = if i >= 8 { "0, 2, or 4" } else { "0-4" };
            return Err(CalcError::InvalidInput(format!(
                "response {} = {v} is out of range (item {} allows {allowed})",
                i + 1,
                i + 1
            )));
        }
    }

    let total: u16 = input.responses.iter().map(|&v| v as u16).sum();
    let zone = RiskZone::from_total(total);

    let interpretation = format!(
        "Total score {total}/40 falls in {} ({}): {}. {} AUDIT is a screening instrument; it is not a diagnosis.",
        zone.zone(),
        zone.label(),
        zone.advice().trim_end_matches('.'),
        "Risk thresholds may be lower for women and older people."
    );

    Ok(AuditOutcome {
        total,
        zone,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &AuditInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("risk_zone".into(), json!(o.zone.zone()));
    working.insert("risk_level".into(), json!(o.zone.label()));
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
pub struct Audit;

impl Calculator for Audit {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "AUDIT Alcohol Use Screen"
    }

    fn description(&self) -> &'static str {
        "Ten-item WHO alcohol-use screen (0-40); four risk zones from low risk to possible dependence."
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
            "title": "AuditInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["responses"],
            "properties": {
                "responses": {
                    "type": "array",
                    "description": "Ten responses (Q1-Q10). Items 1-8 score 0-4; items 9 and 10 score only 0, 2, or 4.",
                    "items": { "type": "integer", "minimum": 0, "maximum": 4 },
                    "minItems": 10,
                    "maxItems": 10,
                    "definition": {
                        "concept": "AUDIT item responses",
                        "statement": "Each item scores a response about the patient's alcohol consumption, dependence symptoms, and alcohol-related harm. Items 1-8 use a 0-4 scale; items 9 and 10 have only three options scored 0, 2, or 4.",
                        "includes": [
                            "Q1 How often do you have a drink containing alcohol?",
                            "Q2 How many drinks containing alcohol do you have on a typical day when you are drinking?",
                            "Q3 How often do you have six or more drinks on one occasion?",
                            "Q4 How often during the last year have you found that you were not able to stop drinking once you had started?",
                            "Q5 How often during the last year have you failed to do what was normally expected from you because of drinking?",
                            "Q6 How often during the last year have you needed a first drink in the morning to get yourself going after a heavy drinking session?",
                            "Q7 How often during the last year have you had a feeling of guilt or remorse after drinking?",
                            "Q8 How often during the last year have you been unable to remember what happened the night before because you had been drinking?",
                            "Q9 Have you or someone else been injured as a result of your drinking? (0=No, 2=Yes but not in the last year, 4=Yes during the last year)",
                            "Q10 Has a relative or friend or a doctor or other health worker been concerned about your drinking or suggested you cut down? (0=No, 2=Yes but not in the last year, 4=Yes during the last year)"
                        ],
                        "caveats": "Risk thresholds may be lower for women and older people. The AUDIT screens; it does not diagnose an alcohol use disorder.",
                        "source": {
                            "citation": "Saunders JB, et al. Addiction. 1993;88(6):791-804; scoring per Babor TF, et al. AUDIT: Guidelines for Use in Primary Care, 2nd ed. WHO; 2001.",
                            "url": "https://doi.org/10.1111/j.1360-0443.1993.tb02093.x"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: AuditInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn responses(v: [u8; 10]) -> AuditInput {
        AuditInput { responses: v.to_vec() }
    }

    #[test]
    fn all_zero_is_low_risk() {
        let o = compute(&responses([0; 10])).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.zone, RiskZone::LowRisk);
    }

    #[test]
    fn worked_example_is_hazardous() {
        // Items 1-8 = 1 each (8), item 9 = 2, item 10 = 0 -> total 10 -> Zone II.
        let o = compute(&responses([1, 1, 1, 1, 1, 1, 1, 1, 2, 0])).unwrap();
        assert_eq!(o.total, 10);
        assert_eq!(o.zone, RiskZone::Hazardous);
        assert!(o.interpretation.contains("Zone II"));
    }

    #[test]
    fn band_boundaries_match_who_zones() {
        assert_eq!(RiskZone::from_total(0), RiskZone::LowRisk);
        assert_eq!(RiskZone::from_total(7), RiskZone::LowRisk);
        assert_eq!(RiskZone::from_total(8), RiskZone::Hazardous);
        assert_eq!(RiskZone::from_total(15), RiskZone::Hazardous);
        assert_eq!(RiskZone::from_total(16), RiskZone::Harmful);
        assert_eq!(RiskZone::from_total(19), RiskZone::Harmful);
        assert_eq!(RiskZone::from_total(20), RiskZone::PossibleDependence);
        assert_eq!(RiskZone::from_total(40), RiskZone::PossibleDependence);
    }

    #[test]
    fn maximum_score_is_possible_dependence() {
        // Items 1-8 max 4 (32) + items 9-10 max 4 (8) = 40.
        let o = compute(&responses([4, 4, 4, 4, 4, 4, 4, 4, 4, 4])).unwrap();
        assert_eq!(o.total, 40);
        assert_eq!(o.zone, RiskZone::PossibleDependence);
    }

    #[test]
    fn wrong_length_is_rejected() {
        assert!(compute(&AuditInput { responses: vec![0; 9] }).is_err());
        assert!(compute(&AuditInput { responses: vec![0; 11] }).is_err());
    }

    #[test]
    fn out_of_range_items_1_to_8_are_rejected() {
        assert!(compute(&responses([5, 0, 0, 0, 0, 0, 0, 0, 0, 0])).is_err());
    }

    #[test]
    fn items_9_and_10_reject_odd_values() {
        // Item 9 (index 8) = 1 is invalid (only 0, 2, 4 allowed).
        assert!(compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 1, 0])).is_err());
        assert!(compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 3, 0])).is_err());
        // Item 10 (index 9) = 1 is invalid.
        assert!(compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 0, 1])).is_err());
        // But 0, 2, 4 are accepted.
        assert!(compute(&responses([0, 0, 0, 0, 0, 0, 0, 0, 2, 4])).is_ok());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let arr = [2, 1, 1, 0, 0, 0, 0, 0, 2, 4];
        let dynamic = Audit.calculate(&json!({ "responses": arr })).unwrap();
        let typed = build_response(&responses(arr)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(10));
        assert_eq!(dynamic.working["risk_zone"], json!("Zone II"));
    }
}
