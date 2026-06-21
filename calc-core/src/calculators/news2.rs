//! NEWS2 - National Early Warning Score 2 (RCP, 2017).
//!
//! NHS-mandated track-and-trigger aggregate physiology score. Each of seven
//! parameters scores 0-3; the sum drives a clinical-response band. Two subtleties
//! are clinically load-bearing and encoded here:
//!
//! - SpO2 has two scales. Scale 1 is the default. Scale 2 is used ONLY for
//!   patients with confirmed hypercapnic (type 2) respiratory failure who have a
//!   prescribed target saturation of 88-92% (typically COPD). Selecting Scale 2
//!   inappropriately is a recognised patient-safety error: it under-scores
//!   hypoxaemia in patients who should be on Scale 1. On Scale 2 the upper
//!   saturation bands depend on whether the patient is breathing air or oxygen.
//! - A single parameter scoring 3 ("a red score") escalates an otherwise-low
//!   aggregate (1-4) to low-medium / urgent ward review, and is surfaced as a
//!   flag regardless of the total.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "news2";

/// Primary citation.
pub const REFERENCE: &str =
    "Royal College of Physicians. National Early Warning Score (NEWS) 2: Standardising the \
assessment of acute-illness severity in the NHS. Updated report of a working party. London: RCP, \
2017.";

/// Distribution licence: the RCP places no copyright restriction on NEWS2 to
/// encourage its use; reproduction requires attribution and that the content is
/// not modified.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free to use, no copyright restriction (RCP) - attribution required, content unmodified",
    source_url: "https://www.rcp.ac.uk/resources/national-early-warning-score-news-2/",
};

/// Which SpO2 scale to score against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Spo2Scale {
    /// Scale 1 - the default, used for all patients except those on Scale 2.
    #[serde(rename = "1")]
    #[default]
    Scale1,
    /// Scale 2 - ONLY for confirmed hypercapnic respiratory failure with a
    /// prescribed target saturation of 88-92% (e.g. COPD).
    #[serde(rename = "2")]
    Scale2,
}

/// Level of consciousness on the ACVPU scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Consciousness {
    /// Alert.
    Alert,
    /// New-onset confusion, or any of Voice / Pain / Unresponsive.
    ConfusionOrVpu,
}

/// NEWS2 inputs. All physiology numeric except the two enums and the oxygen flag.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct News2Input {
    /// Respiratory rate, breaths per minute.
    pub respiratory_rate: u16,
    /// Oxygen saturation, percent.
    pub spo2: u16,
    /// Which SpO2 scale to score against. Defaults to Scale 1.
    #[serde(default)]
    pub spo2_scale: Spo2Scale,
    /// Whether the patient is receiving supplemental oxygen.
    pub on_oxygen: bool,
    /// Systolic blood pressure, mmHg.
    pub systolic_bp: u16,
    /// Pulse, beats per minute.
    pub pulse: u16,
    /// Level of consciousness (ACVPU).
    pub consciousness: Consciousness,
    /// Temperature, degrees Celsius.
    pub temperature: f64,
}

/// Aggregate clinical-response band (RCP NEWS2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseBand {
    /// Aggregate 0: routine monitoring.
    Routine,
    /// Aggregate 1-4 with no single parameter scoring 3: low - ward-based response.
    Low,
    /// Aggregate 1-4 with a single parameter scoring 3: low-medium - urgent ward review.
    LowMedium,
    /// Aggregate 5-6: medium - urgent (key threshold for escalation).
    Medium,
    /// Aggregate >=7: high - emergency response.
    High,
}

impl ResponseBand {
    fn slug(self) -> &'static str {
        match self {
            ResponseBand::Routine => "routine",
            ResponseBand::Low => "low",
            ResponseBand::LowMedium => "low-medium",
            ResponseBand::Medium => "medium",
            ResponseBand::High => "high",
        }
    }
}

/// The computed outcome, with every sub-score exposed for transparency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct News2Outcome {
    /// Total aggregate score (0-20+ depending on parameters).
    pub total: u8,
    pub respiratory_rate_score: u8,
    pub spo2_score: u8,
    pub air_or_oxygen_score: u8,
    pub systolic_bp_score: u8,
    pub pulse_score: u8,
    pub consciousness_score: u8,
    pub temperature_score: u8,
    /// True if any single parameter scored 3 (a "red score").
    pub single_parameter_3: bool,
    pub band: ResponseBand,
    pub interpretation: String,
}

fn respiratory_rate_score(rr: u16) -> u8 {
    match rr {
        0..=8 => 3,
        9..=11 => 1,
        12..=20 => 0,
        21..=24 => 2,
        _ => 3, // >=25
    }
}

fn spo2_scale1_score(spo2: u16) -> u8 {
    match spo2 {
        0..=91 => 3,
        92..=93 => 2,
        94..=95 => 1,
        _ => 0, // >=96
    }
}

/// Scale 2 - hypercapnic respiratory failure with target 88-92%. The upper
/// bands (>=93%) depend on air vs oxygen: a high saturation ON OXYGEN is bad
/// (over-oxygenation risking CO2 retention), but the same saturation ON AIR is
/// fine. The 88-92% target band scores 0 regardless of air/oxygen.
fn spo2_scale2_score(spo2: u16, on_oxygen: bool) -> u8 {
    match spo2 {
        0..=83 => 3,
        84..=85 => 2,
        86..=87 => 1,
        88..=92 => 0,
        _ => {
            // >=93%
            if !on_oxygen {
                0
            } else {
                match spo2 {
                    93..=94 => 1,
                    95..=96 => 2,
                    _ => 3, // >=97 on oxygen
                }
            }
        }
    }
}

fn air_or_oxygen_score(on_oxygen: bool) -> u8 {
    if on_oxygen {
        2
    } else {
        0
    }
}

fn systolic_bp_score(sbp: u16) -> u8 {
    match sbp {
        0..=90 => 3,
        91..=100 => 2,
        101..=110 => 1,
        111..=219 => 0,
        _ => 3, // >=220
    }
}

fn pulse_score(pulse: u16) -> u8 {
    match pulse {
        0..=40 => 3,
        41..=50 => 1,
        51..=90 => 0,
        91..=110 => 1,
        111..=130 => 2,
        _ => 3, // >=131
    }
}

fn consciousness_score(c: Consciousness) -> u8 {
    match c {
        Consciousness::Alert => 0,
        Consciousness::ConfusionOrVpu => 3,
    }
}

/// Temperature is banded on tenths of a degree, so compare with care:
/// <=35.0 ->3; 35.1-36.0 ->1; 36.1-38.0 ->0; 38.1-39.0 ->1; >=39.1 ->2.
fn temperature_score(temp: f64) -> u8 {
    if temp <= 35.0 {
        3
    } else if temp <= 36.0 {
        1
    } else if temp <= 38.0 {
        0
    } else if temp <= 39.0 {
        1
    } else {
        2
    }
}

/// Pure scoring.
pub fn compute(input: &News2Input) -> Result<News2Outcome, CalcError> {
    if !input.temperature.is_finite() {
        return Err(CalcError::InvalidInput(
            "temperature must be a finite number".into(),
        ));
    }
    // Physiologically implausible values: guard against transcription errors
    // rather than silently scoring them.
    if input.spo2 > 100 {
        return Err(CalcError::InvalidInput(
            "spo2 must be a percentage between 0 and 100".into(),
        ));
    }
    if input.temperature < 20.0 || input.temperature > 45.0 {
        return Err(CalcError::InvalidInput(
            "temperature must be within a physiological range (20-45 degC)".into(),
        ));
    }

    let respiratory_rate_score = respiratory_rate_score(input.respiratory_rate);
    let spo2_score = match input.spo2_scale {
        Spo2Scale::Scale1 => spo2_scale1_score(input.spo2),
        Spo2Scale::Scale2 => spo2_scale2_score(input.spo2, input.on_oxygen),
    };
    let air_or_oxygen_score = air_or_oxygen_score(input.on_oxygen);
    let systolic_bp_score = systolic_bp_score(input.systolic_bp);
    let pulse_score = pulse_score(input.pulse);
    let consciousness_score = consciousness_score(input.consciousness);
    let temperature_score = temperature_score(input.temperature);

    let subscores = [
        respiratory_rate_score,
        spo2_score,
        air_or_oxygen_score,
        systolic_bp_score,
        pulse_score,
        consciousness_score,
        temperature_score,
    ];
    let total: u8 = subscores.iter().sum();
    let single_parameter_3 = subscores.contains(&3);

    let band = if total == 0 {
        ResponseBand::Routine
    } else if total >= 7 {
        ResponseBand::High
    } else if total >= 5 {
        ResponseBand::Medium
    } else if single_parameter_3 {
        // Aggregate 1-4 but a red score.
        ResponseBand::LowMedium
    } else {
        ResponseBand::Low
    };

    let interpretation = match band {
        ResponseBand::Routine => {
            "NEWS2 0: routine monitoring (minimum 12-hourly). Continue routine NEWS2 observations."
                .to_string()
        }
        ResponseBand::Low => format!(
            "NEWS2 {total} (low). Ward-based response: minimum 4-6 hourly monitoring; inform the \
registered nurse, who decides on any change to monitoring frequency or escalation."
        ),
        ResponseBand::LowMedium => format!(
            "NEWS2 {total} (low-medium): a single parameter scored 3 (a red score). Urgent review \
by a clinician with competencies in acute illness, at minimum 1-hourly monitoring."
        ),
        ResponseBand::Medium => format!(
            "NEWS2 {total} (medium). Urgent response: registered nurse to immediately inform the \
medical team and request urgent review by a clinician competent in acute illness; minimum \
1-hourly monitoring."
        ),
        ResponseBand::High => format!(
            "NEWS2 {total} (high). Emergency response: immediate assessment by a critical-care \
competent team, usually transfer to a higher level of care; continuous monitoring."
        ),
    };

    Ok(News2Outcome {
        total,
        respiratory_rate_score,
        spo2_score,
        air_or_oxygen_score,
        systolic_bp_score,
        pulse_score,
        consciousness_score,
        temperature_score,
        single_parameter_3,
        band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &News2Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.total));
    working.insert("respiratory_rate_score".into(), json!(o.respiratory_rate_score));
    working.insert("spo2_score".into(), json!(o.spo2_score));
    working.insert("air_or_oxygen_score".into(), json!(o.air_or_oxygen_score));
    working.insert("systolic_bp_score".into(), json!(o.systolic_bp_score));
    working.insert("pulse_score".into(), json!(o.pulse_score));
    working.insert("consciousness_score".into(), json!(o.consciousness_score));
    working.insert("temperature_score".into(), json!(o.temperature_score));
    working.insert("single_parameter_3".into(), json!(o.single_parameter_3));
    working.insert("band".into(), json!(o.band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.total),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct News2;

impl Calculator for News2 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "NEWS2 (National Early Warning Score 2)"
    }

    fn description(&self) -> &'static str {
        "NHS-mandated aggregate physiology score (RCP 2017) driving the clinical-response band."
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
            "title": "News2Input",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "respiratory_rate", "spo2", "on_oxygen",
                "systolic_bp", "pulse", "consciousness", "temperature"
            ],
            "properties": {
                "respiratory_rate": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 80,
                    "description": "Respiratory rate, breaths/min (<=8 or >=25 scores 3; 12-20 scores 0)"
                },
                "spo2": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "Oxygen saturation, % (scored against the chosen spo2_scale)"
                },
                "spo2_scale": {
                    "type": "string",
                    "enum": ["1", "2"],
                    "default": "1",
                    "description": "Which SpO2 scale to score against (Scale 1 is the default)",
                    "definition": {
                        "concept": "SpO2 scale selection",
                        "statement": "Scale 1 is used for all patients EXCEPT those with confirmed hypercapnic (type 2) respiratory failure who have a prescribed target oxygen saturation of 88-92% (e.g. COPD), who use Scale 2.",
                        "includes": ["Scale 2: confirmed hypercapnic respiratory failure with a documented 88-92% target (typically COPD)"],
                        "excludes": [
                            "Do NOT use Scale 2 by default or for hypoxaemia of uncertain cause: an inappropriate Scale 2 under-scores hypoxaemia and is a recognised patient-safety error",
                            "Scale 2 should only be applied on the direction of a competent clinical decision-maker"
                        ],
                        "caveats": "On Scale 2 the upper bands (>=93%) depend on whether the patient is on air or oxygen.",
                        "source": { "citation": "Royal College of Physicians. NEWS2. London: RCP, 2017.", "url": "https://www.rcp.ac.uk/resources/national-early-warning-score-news-2/" },
                        "status": "draft"
                    }
                },
                "on_oxygen": {
                    "type": "boolean",
                    "description": "Receiving supplemental oxygen (scores 2; also affects Scale 2 SpO2 scoring)",
                    "definition": {
                        "concept": "Air or oxygen",
                        "statement": "TRUE if the patient is receiving any supplemental oxygen at the time of observation; FALSE if breathing room air.",
                        "source": { "citation": "Royal College of Physicians. NEWS2. London: RCP, 2017.", "url": "https://www.rcp.ac.uk/resources/national-early-warning-score-news-2/" },
                        "status": "draft"
                    }
                },
                "systolic_bp": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 300,
                    "description": "Systolic blood pressure, mmHg (<=90 or >=220 scores 3; 111-219 scores 0)"
                },
                "pulse": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 300,
                    "description": "Pulse, beats/min (<=40 or >=131 scores 3; 51-90 scores 0)"
                },
                "consciousness": {
                    "type": "string",
                    "enum": ["alert", "confusion-or-vpu"],
                    "description": "Level of consciousness on ACVPU (Alert scores 0; anything else scores 3)",
                    "definition": {
                        "concept": "Consciousness (ACVPU)",
                        "statement": "Alert scores 0. New-onset confusion (the 'C' in ACVPU) or any response only to Voice, Pain, or being Unresponsive scores 3.",
                        "includes": ["New confusion / delirium", "Responds to Voice only", "Responds to Pain only", "Unresponsive"],
                        "excludes": ["Long-standing, baseline confusion that is not new is a clinical judgement; NEWS2 scores NEW confusion"],
                        "source": { "citation": "Royal College of Physicians. NEWS2. London: RCP, 2017.", "url": "https://www.rcp.ac.uk/resources/national-early-warning-score-news-2/" },
                        "status": "draft"
                    }
                },
                "temperature": {
                    "type": "number",
                    "minimum": 20,
                    "maximum": 45,
                    "description": "Temperature, degC (<=35.0 scores 3; 36.1-38.0 scores 0; >=39.1 scores 2)"
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: News2Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wholly normal observation set: every parameter scores 0.
    fn normal() -> News2Input {
        News2Input {
            respiratory_rate: 16,
            spo2: 98,
            spo2_scale: Spo2Scale::Scale1,
            on_oxygen: false,
            systolic_bp: 120,
            pulse: 70,
            consciousness: Consciousness::Alert,
            temperature: 36.8,
        }
    }

    #[test]
    fn all_normal_is_zero_routine() {
        let o = compute(&normal()).unwrap();
        assert_eq!(o.total, 0);
        assert_eq!(o.band, ResponseBand::Routine);
        assert!(!o.single_parameter_3);
    }

    #[test]
    fn worked_multi_parameter_example() {
        // RR 24 (2) + SpO2 93 Scale1 (2) + on oxygen (2) + SBP 105 (1)
        // + pulse 105 (1) + alert (0) + temp 38.5 (1) = 9 -> high.
        let i = News2Input {
            respiratory_rate: 24,
            spo2: 93,
            spo2_scale: Spo2Scale::Scale1,
            on_oxygen: true,
            systolic_bp: 105,
            pulse: 105,
            consciousness: Consciousness::Alert,
            temperature: 38.5,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.respiratory_rate_score, 2);
        assert_eq!(o.spo2_score, 2);
        assert_eq!(o.air_or_oxygen_score, 2);
        assert_eq!(o.systolic_bp_score, 1);
        assert_eq!(o.pulse_score, 1);
        assert_eq!(o.consciousness_score, 0);
        assert_eq!(o.temperature_score, 1);
        assert_eq!(o.total, 9);
        assert_eq!(o.band, ResponseBand::High);
        assert!(!o.single_parameter_3);
    }

    #[test]
    fn respiratory_rate_bands() {
        assert_eq!(respiratory_rate_score(8), 3);
        assert_eq!(respiratory_rate_score(9), 1);
        assert_eq!(respiratory_rate_score(11), 1);
        assert_eq!(respiratory_rate_score(12), 0);
        assert_eq!(respiratory_rate_score(20), 0);
        assert_eq!(respiratory_rate_score(21), 2);
        assert_eq!(respiratory_rate_score(24), 2);
        assert_eq!(respiratory_rate_score(25), 3);
    }

    #[test]
    fn spo2_scale1_bands() {
        assert_eq!(spo2_scale1_score(91), 3);
        assert_eq!(spo2_scale1_score(92), 2);
        assert_eq!(spo2_scale1_score(93), 2);
        assert_eq!(spo2_scale1_score(94), 1);
        assert_eq!(spo2_scale1_score(95), 1);
        assert_eq!(spo2_scale1_score(96), 0);
        assert_eq!(spo2_scale1_score(100), 0);
    }

    #[test]
    fn spo2_scale2_bands() {
        // Lower bands ignore air/oxygen.
        assert_eq!(spo2_scale2_score(83, false), 3);
        assert_eq!(spo2_scale2_score(84, true), 2);
        assert_eq!(spo2_scale2_score(86, false), 1);
        assert_eq!(spo2_scale2_score(88, true), 0);
        assert_eq!(spo2_scale2_score(92, true), 0);
        // Upper bands (>=93%) depend on air vs oxygen.
        assert_eq!(spo2_scale2_score(93, false), 0, "93%+ on air is fine");
        assert_eq!(spo2_scale2_score(100, false), 0, "high sat on air scores 0");
        assert_eq!(spo2_scale2_score(93, true), 1);
        assert_eq!(spo2_scale2_score(94, true), 1);
        assert_eq!(spo2_scale2_score(95, true), 2);
        assert_eq!(spo2_scale2_score(96, true), 2);
        assert_eq!(spo2_scale2_score(97, true), 3, "over-oxygenation in COPD scores 3");
        assert_eq!(spo2_scale2_score(100, true), 3);
    }

    #[test]
    fn scale2_differs_from_scale1_for_copd() {
        // 90% SpO2 on air: Scale 1 scores 3 (hypoxaemic), Scale 2 scores 0 (in target).
        let mut i = normal();
        i.spo2 = 90;
        i.spo2_scale = Spo2Scale::Scale1;
        assert_eq!(compute(&i).unwrap().spo2_score, 3);
        i.spo2_scale = Spo2Scale::Scale2;
        assert_eq!(compute(&i).unwrap().spo2_score, 0);
    }

    #[test]
    fn systolic_bp_bands() {
        assert_eq!(systolic_bp_score(90), 3);
        assert_eq!(systolic_bp_score(91), 2);
        assert_eq!(systolic_bp_score(100), 2);
        assert_eq!(systolic_bp_score(101), 1);
        assert_eq!(systolic_bp_score(110), 1);
        assert_eq!(systolic_bp_score(111), 0);
        assert_eq!(systolic_bp_score(219), 0);
        assert_eq!(systolic_bp_score(220), 3);
    }

    #[test]
    fn pulse_bands() {
        assert_eq!(pulse_score(40), 3);
        assert_eq!(pulse_score(41), 1);
        assert_eq!(pulse_score(50), 1);
        assert_eq!(pulse_score(51), 0);
        assert_eq!(pulse_score(90), 0);
        assert_eq!(pulse_score(91), 1);
        assert_eq!(pulse_score(110), 1);
        assert_eq!(pulse_score(111), 2);
        assert_eq!(pulse_score(130), 2);
        assert_eq!(pulse_score(131), 3);
    }

    #[test]
    fn temperature_bands() {
        assert_eq!(temperature_score(35.0), 3);
        assert_eq!(temperature_score(35.1), 1);
        assert_eq!(temperature_score(36.0), 1);
        assert_eq!(temperature_score(36.1), 0);
        assert_eq!(temperature_score(38.0), 0);
        assert_eq!(temperature_score(38.1), 1);
        assert_eq!(temperature_score(39.0), 1);
        assert_eq!(temperature_score(39.1), 2);
    }

    #[test]
    fn consciousness_scores() {
        assert_eq!(consciousness_score(Consciousness::Alert), 0);
        assert_eq!(consciousness_score(Consciousness::ConfusionOrVpu), 3);
    }

    #[test]
    fn band_boundaries() {
        // Aggregate 5 from non-red parameters -> medium.
        let mut i = normal();
        i.respiratory_rate = 21; // 2
        i.systolic_bp = 95; // 2
        i.pulse = 95; // 1
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 5);
        assert!(!o.single_parameter_3);
        assert_eq!(o.band, ResponseBand::Medium);

        // Aggregate 6 -> still medium.
        i.pulse = 115; // 2
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 6);
        assert_eq!(o.band, ResponseBand::Medium);

        // Aggregate 7 -> high.
        i.temperature = 35.5; // 1
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 7);
        assert_eq!(o.band, ResponseBand::High);
    }

    #[test]
    fn low_band_no_red_score() {
        // RR 9 (1) + SBP 105 (1) = 2, no parameter scores 3 -> low.
        let mut i = normal();
        i.respiratory_rate = 9;
        i.systolic_bp = 105;
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 2);
        assert!(!o.single_parameter_3);
        assert_eq!(o.band, ResponseBand::Low);
    }

    #[test]
    fn single_parameter_3_escalates_low_to_low_medium() {
        // A single red score (RR 26 -> 3) with an aggregate of 3 (<=4) -> low-medium.
        let mut i = normal();
        i.respiratory_rate = 26; // 3
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 3);
        assert!(o.single_parameter_3);
        assert_eq!(o.band, ResponseBand::LowMedium);
        assert!(o.interpretation.contains("red score"));
    }

    #[test]
    fn single_parameter_3_flag_set_even_when_high() {
        // The flag is surfaced regardless of the aggregate band.
        let mut i = normal();
        i.respiratory_rate = 26; // 3
        i.systolic_bp = 85; // 3
        i.pulse = 135; // 3
        let o = compute(&i).unwrap();
        assert!(o.total >= 7);
        assert!(o.single_parameter_3);
        assert_eq!(o.band, ResponseBand::High);
    }

    #[test]
    fn confusion_alone_is_a_red_score() {
        let mut i = normal();
        i.consciousness = Consciousness::ConfusionOrVpu; // 3
        let o = compute(&i).unwrap();
        assert_eq!(o.total, 3);
        assert!(o.single_parameter_3);
        assert_eq!(o.band, ResponseBand::LowMedium);
    }

    #[test]
    fn scale_defaults_to_scale1() {
        // Omitting spo2_scale should default to Scale 1.
        let value = json!({
            "respiratory_rate": 16, "spo2": 90, "on_oxygen": false,
            "systolic_bp": 120, "pulse": 70, "consciousness": "alert", "temperature": 36.8
        });
        let parsed: News2Input = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.spo2_scale, Spo2Scale::Scale1);
        // 90% on Scale 1 scores 3.
        assert_eq!(compute(&parsed).unwrap().spo2_score, 3);
    }

    #[test]
    fn rejects_implausible_input() {
        let mut i = normal();
        i.spo2 = 150;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.temperature = 60.0;
        assert!(compute(&i).is_err());

        let mut i = normal();
        i.temperature = f64::NAN;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "respiratory_rate": 24, "spo2": 93, "spo2_scale": "1", "on_oxygen": true,
            "systolic_bp": 105, "pulse": 105, "consciousness": "alert", "temperature": 38.5
        });
        let typed = News2Input {
            respiratory_rate: 24,
            spo2: 93,
            spo2_scale: Spo2Scale::Scale1,
            on_oxygen: true,
            systolic_bp: 105,
            pulse: 105,
            consciousness: Consciousness::Alert,
            temperature: 38.5,
        };
        let dynamic = News2.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(9));
    }

    #[test]
    fn schema_flags_scale2_safety() {
        let schema = News2.input_schema();
        let excludes = &schema["properties"]["spo2_scale"]["definition"]["excludes"];
        assert!(excludes[0]
            .as_str()
            .unwrap()
            .contains("patient-safety error"));
    }

    #[test]
    fn working_map_has_every_subscore() {
        let resp = build_response(&normal()).unwrap();
        for key in [
            "total_score",
            "respiratory_rate_score",
            "spo2_score",
            "air_or_oxygen_score",
            "systolic_bp_score",
            "pulse_score",
            "consciousness_score",
            "temperature_score",
            "single_parameter_3",
            "band",
        ] {
            assert!(resp.working.contains_key(key), "missing {key}");
        }
    }
}
