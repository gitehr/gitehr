//! FIB-4 - Fibrosis-4 index for liver fibrosis risk (NAFLD/MASLD).
//!
//! `FIB-4 = (age x AST) / (platelets x sqrt(ALT))`, a non-invasive screen for
//! advanced liver fibrosis (NICE NG49). The low-risk cut-off is 1.30, raised to
//! 2.0 for patients aged 65 or over to reduce age-related false positives;
//! above 2.67 is high risk and warrants referral.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "fib4";

/// Distribution licence: FIB-4 is a published method, implemented here from the
/// primary literature.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Public-domain method - implemented from the primary literature",
    source_url: "https://doi.org/10.1002/hep.21178",
};

/// Primary citation.
pub const REFERENCE: &str =
    "Sterling RK, Lissen E, Clumeck N, et al. Development of a simple noninvasive index to predict \
significant fibrosis in patients with HIV/HCV coinfection. Hepatology. 2006;43(6):1317-1325. \
Thresholds per NICE NG49 (NAFLD).";

/// Upper cut-off: above this is high risk of advanced fibrosis.
pub const HIGH_CUTOFF: f64 = 2.67;
/// Low-risk cut-off below age 65.
pub const LOW_CUTOFF_UNDER_65: f64 = 1.30;
/// Low-risk cut-off at age 65 and over.
pub const LOW_CUTOFF_65_PLUS: f64 = 2.0;

/// Inputs to the FIB-4 index.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fib4Input {
    /// Age in years.
    pub age: u8,
    /// Aspartate aminotransferase, U/L.
    pub ast: f64,
    /// Alanine aminotransferase, U/L.
    pub alt: f64,
    /// Platelet count, x10^9/L.
    pub platelets: f64,
}

/// Risk band for advanced fibrosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Low,
    Indeterminate,
    High,
}

impl Risk {
    fn slug(self) -> &'static str {
        match self {
            Risk::Low => "low",
            Risk::Indeterminate => "indeterminate",
            Risk::High => "high",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct Fib4Outcome {
    /// The FIB-4 index, rounded to two decimal places.
    pub index: f64,
    pub risk: Risk,
    /// The age-adjusted low-risk cut-off applied.
    pub low_cutoff: f64,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &Fib4Input) -> Result<Fib4Outcome, CalcError> {
    if input.ast <= 0.0 || input.alt <= 0.0 || input.platelets <= 0.0 {
        return Err(CalcError::InvalidInput(
            "AST, ALT, and platelets must be positive".into(),
        ));
    }
    if !input.ast.is_finite() || !input.alt.is_finite() || !input.platelets.is_finite() {
        return Err(CalcError::InvalidInput("values must be finite".into()));
    }

    let raw = (input.age as f64 * input.ast) / (input.platelets * input.alt.sqrt());
    let index = (raw * 100.0).round() / 100.0;

    let low_cutoff = if input.age >= 65 {
        LOW_CUTOFF_65_PLUS
    } else {
        LOW_CUTOFF_UNDER_65
    };

    let risk = if index > HIGH_CUTOFF {
        Risk::High
    } else if index < low_cutoff {
        Risk::Low
    } else {
        Risk::Indeterminate
    };

    let interpretation = match risk {
        Risk::Low => format!(
            "FIB-4 {index} is below the low-risk cut-off ({low_cutoff} for this age): advanced \
liver fibrosis is unlikely. Per NICE NG49, advanced fibrosis can be ruled out; reassess \
periodically if risk factors persist."
        ),
        Risk::Indeterminate => format!(
            "FIB-4 {index} is indeterminate (between {low_cutoff} and {HIGH_CUTOFF}): advanced \
fibrosis can be neither excluded nor confirmed. Consider further assessment such as ELF testing \
or transient elastography."
        ),
        Risk::High => format!(
            "FIB-4 {index} is above {HIGH_CUTOFF}: high risk of advanced liver fibrosis. Refer for \
specialist hepatology assessment."
        ),
    };

    Ok(Fib4Outcome {
        index,
        risk,
        low_cutoff,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Fib4Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("fib4_index".into(), json!(o.index));
    working.insert("risk".into(), json!(o.risk.slug()));
    working.insert("low_risk_cutoff".into(), json!(o.low_cutoff));
    working.insert("high_risk_cutoff".into(), json!(HIGH_CUTOFF));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.index),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Fib4;

impl Calculator for Fib4 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "FIB-4 Liver Fibrosis Index"
    }

    fn description(&self) -> &'static str {
        "Non-invasive screen for advanced liver fibrosis from age, AST, ALT, and platelets (NICE NG49)."
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
            "title": "Fib4Input",
            "type": "object",
            "additionalProperties": false,
            "required": ["age", "ast", "alt", "platelets"],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120,
                    "description": "Age in years (the low-risk cut-off rises to 2.0 at age 65+)"
                },
                "ast": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Aspartate aminotransferase (AST), U/L"
                },
                "alt": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Alanine aminotransferase (ALT), U/L"
                },
                "platelets": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "description": "Platelet count, x10^9/L",
                    "definition": {
                        "concept": "Platelet count units",
                        "statement": "Platelet count in x10^9/L (equivalently x1000/microL), the standard UK reporting unit.",
                        "excludes": [
                            "Do NOT pass the raw per-microlitre count (e.g. 250000); use 250"
                        ],
                        "source": {
                            "citation": "Sterling RK et al. Hepatology. 2006;43(6):1317-1325.",
                            "url": "https://doi.org/10.1002/hep.21178"
                        },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Fib4Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(age: u8, ast: f64, alt: f64, platelets: f64) -> Fib4Input {
        Fib4Input { age, ast, alt, platelets }
    }

    #[test]
    fn formula_matches_worked_example() {
        // age 50, AST 20, ALT 25, plt 250: (50*20)/(250*5) = 1000/1250 = 0.80.
        let o = compute(&input(50, 20.0, 25.0, 250.0)).unwrap();
        assert_eq!(o.index, 0.80);
        assert_eq!(o.risk, Risk::Low);
    }

    #[test]
    fn high_index_is_high_risk() {
        // age 70, AST 80, ALT 40, plt 100: (70*80)/(100*6.3246) = 8.85.
        let o = compute(&input(70, 80.0, 40.0, 100.0)).unwrap();
        assert!(o.index > HIGH_CUTOFF);
        assert_eq!(o.risk, Risk::High);
    }

    #[test]
    fn age_raises_the_low_cutoff() {
        // FIB-4 = 1.5 for both: low risk at 65+ (cut-off 2.0), indeterminate under 65 (cut-off 1.3).
        // age 70: (70*30)/(280*5) = 2100/1400 = 1.50.
        let old = compute(&input(70, 30.0, 25.0, 280.0)).unwrap();
        assert_eq!(old.index, 1.50);
        assert_eq!(old.low_cutoff, LOW_CUTOFF_65_PLUS);
        assert_eq!(old.risk, Risk::Low);

        // age 50: (50*42)/(280*5) = 2100/1400 = 1.50.
        let young = compute(&input(50, 42.0, 25.0, 280.0)).unwrap();
        assert_eq!(young.index, 1.50);
        assert_eq!(young.low_cutoff, LOW_CUTOFF_UNDER_65);
        assert_eq!(young.risk, Risk::Indeterminate);
    }

    #[test]
    fn indeterminate_band() {
        // age 50, target ~2.0: (50*50)/(250*5) = 2500/1250 = 2.0.
        let o = compute(&input(50, 50.0, 25.0, 250.0)).unwrap();
        assert_eq!(o.index, 2.0);
        assert_eq!(o.risk, Risk::Indeterminate);
    }

    #[test]
    fn rejects_nonpositive() {
        assert!(compute(&input(50, 0.0, 25.0, 250.0)).is_err());
        assert!(compute(&input(50, 20.0, 0.0, 250.0)).is_err());
        assert!(compute(&input(50, 20.0, 25.0, 0.0)).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({ "age": 50, "ast": 20, "alt": 25, "platelets": 250 });
        let dynamic = Fib4.calculate(&value).unwrap();
        let typed = build_response(&input(50, 20.0, 25.0, 250.0)).unwrap();
        assert_eq!(dynamic, typed);
        assert_eq!(dynamic.result, json!(0.80));
    }
}
