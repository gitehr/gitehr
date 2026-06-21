//! 4AT - rapid assessment test for delirium and cognitive impairment.
//!
//! A short (typically <2 minute) bedside screen for delirium, usable by any
//! healthcare professional at first contact and whenever delirium is suspected,
//! without special training (MacLullich et al; www.the4at.com). Four items
//! scored 0-12: a total of 4 or above flags possible delirium.
//!
//! Two clinical subtleties are encoded here:
//! - Alertness is scored from observation: both "normal" and "mild sleepiness
//!   for <10 seconds after waking, then normal" score 0; only clearly abnormal
//!   (markedly drowsy or agitated/hyperactive) scores 4. The two zero-scoring
//!   states are kept as distinct inputs so the observed finding is recorded
//!   rather than collapsed.
//! - A score of 0 does not exclude delirium if item 4 (acute change) could not
//!   be established, so the interpretation says so explicitly.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name. Cannot start with a digit, so the spelled-out form is used.
pub const NAME: &str = "fourat";

/// Primary citation.
pub const REFERENCE: &str =
    "MacLullich AMJ, Ryan T, Cash H. 4AT: Rapid Clinical Test for Delirium Detection. Version 1.1, \
May 2014. Validated in Bellelli G, et al. Age Ageing. 2014;43(4):496-502, and Shenkin SD, et al. \
BMC Med. 2019;17:138. https://www.the4at.com";

/// Distribution licence: the 4AT is made freely available by its authors under
/// Creative Commons Attribution (CC-BY-4.0), with no permission, payment, or
/// registration required, and is explicitly free to incorporate into EHR/EMR
/// systems. Attribution is to www.the4at.com.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "CC-BY-4.0 - free to use, reproduce, and incorporate into clinical software with \
attribution to www.the4at.com; no permission or payment required",
    source_url: "https://www.the4at.com/4at-faq",
};

/// Item 1: alertness, scored from observation.
///
/// Both `Normal` and `MildSleepiness` score 0; only `ClearlyAbnormal` scores 4.
/// The two zero-scoring observations are kept distinct to record the finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alertness {
    /// Fully alert (but not agitated) throughout assessment.
    Normal,
    /// Mild sleepiness for <10 seconds after waking, then normal.
    MildSleepiness,
    /// Markedly drowsy (difficult to rouse / obviously sleepy) or
    /// agitated/hyperactive.
    ClearlyAbnormal,
}

impl Alertness {
    fn points(self) -> u8 {
        match self {
            Alertness::Normal | Alertness::MildSleepiness => 0,
            Alertness::ClearlyAbnormal => 4,
        }
    }
}

/// Item 2: AMT4 (age, date of birth, place [name of hospital/building], current
/// year), scored by number of mistakes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Amt4 {
    /// No mistakes.
    NoMistakes,
    /// 1 mistake.
    OneMistake,
    /// 2 or more mistakes, or untestable.
    TwoOrMoreOrUntestable,
}

impl Amt4 {
    fn points(self) -> u8 {
        match self {
            Amt4::NoMistakes => 0,
            Amt4::OneMistake => 1,
            Amt4::TwoOrMoreOrUntestable => 2,
        }
    }
}

/// Item 3: attention (months of the year backwards, starting from December).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attention {
    /// Achieves 7 months or more correctly.
    SevenOrMore,
    /// Starts but scores <7 months, or refuses to start.
    BelowSevenOrRefuses,
    /// Untestable (cannot start because unwell, drowsy, inattentive).
    Untestable,
}

impl Attention {
    fn points(self) -> u8 {
        match self {
            Attention::SevenOrMore => 0,
            Attention::BelowSevenOrRefuses => 1,
            Attention::Untestable => 2,
        }
    }
}

/// 4AT inputs: the four items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourAtInput {
    /// Item 1: alertness, from observation.
    pub alertness: Alertness,
    /// Item 2: AMT4 mistakes.
    pub amt4: Amt4,
    /// Item 3: attention (months backwards).
    pub attention: Attention,
    /// Item 4: evidence of acute change or fluctuating course in alertness,
    /// cognition, or other mental function over the last 2 weeks and still
    /// evident in the last 24 hours.
    pub acute_change: bool,
}

/// Screening band (the4AT.com guidance notes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Score 4 or above: possible delirium +/- cognitive impairment.
    PossibleDelirium,
    /// Score 1-3: possible cognitive impairment.
    PossibleCognitiveImpairment,
    /// Score 0: delirium or severe cognitive impairment unlikely.
    Unlikely,
}

impl Band {
    fn slug(self) -> &'static str {
        match self {
            Band::PossibleDelirium => "possible-delirium",
            Band::PossibleCognitiveImpairment => "possible-cognitive-impairment",
            Band::Unlikely => "unlikely",
        }
    }
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FourAtOutcome {
    /// Total score (0-12).
    pub score: u8,
    pub band: Band,
    pub interpretation: String,
}

/// Pure scoring.
pub fn compute(input: &FourAtInput) -> Result<FourAtOutcome, CalcError> {
    let score = input.alertness.points()
        + input.amt4.points()
        + input.attention.points()
        + 4 * u8::from(input.acute_change);

    let band = if score >= 4 {
        Band::PossibleDelirium
    } else if score >= 1 {
        Band::PossibleCognitiveImpairment
    } else {
        Band::Unlikely
    };

    let interpretation = match band {
        Band::PossibleDelirium => format!(
            "Score {score}: possible delirium +/- cognitive impairment. A score of 4 or above \
suggests delirium but is not diagnostic; more detailed assessment of mental status may be \
required to reach a diagnosis."
        ),
        Band::PossibleCognitiveImpairment => format!(
            "Score {score}: possible cognitive impairment. More detailed cognitive testing and \
informant history-taking are usually required."
        ),
        Band::Unlikely => {
            if input.acute_change {
                "Score 0: delirium or severe cognitive impairment unlikely.".to_string()
            } else {
                "Score 0: delirium or severe cognitive impairment unlikely - but delirium is \
still possible if the acute-change (item 4) information was incomplete, and more detailed \
testing may be required depending on the clinical context."
                    .to_string()
            }
        }
    };

    Ok(FourAtOutcome {
        score,
        band,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &FourAtInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("alertness".into(), json!(input.alertness.points()));
    working.insert("amt4".into(), json!(input.amt4.points()));
    working.insert("attention".into(), json!(input.attention.points()));
    working.insert("acute_change".into(), json!(4 * u8::from(input.acute_change)));
    working.insert("band".into(), json!(o.band.slug()));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct FourAt;

impl Calculator for FourAt {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "4AT Rapid Delirium Screening"
    }

    fn description(&self) -> &'static str {
        "Rapid bedside screen for delirium and cognitive impairment (four items, score 0-12)."
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
            "title": "FourAtInput",
            "type": "object",
            "additionalProperties": false,
            "required": ["alertness", "amt4", "attention", "acute_change"],
            "properties": {
                "alertness": {
                    "type": "string",
                    "enum": ["normal", "mild_sleepiness", "clearly_abnormal"],
                    "description": "Item 1, from observation: normal (0), mild sleepiness <10s after waking then normal (0), or clearly abnormal i.e. markedly drowsy or agitated (4)",
                    "definition": {
                        "concept": "Alertness (item 1)",
                        "statement": "Level of alertness observed at the bedside. Score 4 only if clearly abnormal: markedly drowsy (difficult to rouse / obviously sleepy) or agitated/hyperactive.",
                        "caveats": "Both fully alert and brief (<10s) post-waking sleepiness that then normalises score 0. Rate solely on observation at the time of assessment. Altered alertness is very likely to be delirium in general hospital settings.",
                        "source": { "citation": "MacLullich AMJ, Ryan T, Cash H. 4AT v1.1, May 2014.", "url": "https://www.the4at.com" },
                        "status": "draft"
                    }
                },
                "amt4": {
                    "type": "string",
                    "enum": ["no_mistakes", "one_mistake", "two_or_more_or_untestable"],
                    "description": "Item 2, AMT4 (age, date of birth, place [name of hospital/building], current year): no mistakes (0), 1 mistake (1), 2+ mistakes or untestable (2)",
                    "definition": {
                        "concept": "AMT4 (item 2)",
                        "statement": "Abbreviated Mental Test - 4: age, date of birth, place (name of the hospital or building), and current year.",
                        "caveats": "This score can be extracted from the AMT10 if that is done immediately before.",
                        "source": { "citation": "MacLullich AMJ, Ryan T, Cash H. 4AT v1.1, May 2014.", "url": "https://www.the4at.com" },
                        "status": "draft"
                    }
                },
                "attention": {
                    "type": "string",
                    "enum": ["seven_or_more", "below_seven_or_refuses", "untestable"],
                    "description": "Item 3, months of the year backwards from December: 7 or more correct (0), starts but <7 or refuses (1), untestable - cannot start because unwell/drowsy/inattentive (2)",
                    "definition": {
                        "concept": "Attention (item 3)",
                        "statement": "Ask the patient to recite the months of the year in backwards order, starting at December. One prompt of 'what is the month before December?' is permitted to assist initial understanding.",
                        "caveats": "'Untestable' means the patient cannot start because they are unwell, drowsy, or inattentive - distinct from starting but managing fewer than 7 months or refusing.",
                        "source": { "citation": "MacLullich AMJ, Ryan T, Cash H. 4AT v1.1, May 2014.", "url": "https://www.the4at.com" },
                        "status": "draft"
                    }
                },
                "acute_change": {
                    "type": "boolean",
                    "description": "Item 4: acute change or fluctuating course (in alertness, cognition, or other mental function) over the last 2 weeks and still evident in the last 24 hours - yes scores 4",
                    "definition": {
                        "concept": "Acute change or fluctuating course (item 4)",
                        "statement": "Evidence of significant change or fluctuation in alertness, cognition, or other mental function (e.g. paranoia, hallucinations) arising over the last 2 weeks and still evident in the last 24 hours.",
                        "includes": ["Fluctuation that can occur without delirium in some dementia, but marked fluctuation usually indicates delirium"],
                        "caveats": "Item 4 requires information from one or more sources (ward nurses, GP letter, case notes, carers, the tester's own knowledge). If this information is incomplete, a total of 0 does not exclude delirium.",
                        "source": { "citation": "MacLullich AMJ, Ryan T, Cash H. 4AT v1.1, May 2014.", "url": "https://www.the4at.com" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: FourAtInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> FourAtInput {
        FourAtInput {
            alertness: Alertness::Normal,
            amt4: Amt4::NoMistakes,
            attention: Attention::SevenOrMore,
            acute_change: false,
        }
    }

    #[test]
    fn all_normal_scores_zero() {
        let o = compute(&base()).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.band, Band::Unlikely);
    }

    #[test]
    fn mild_sleepiness_scores_zero_like_normal() {
        let mut i = base();
        i.alertness = Alertness::MildSleepiness;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 0);
        assert_eq!(o.band, Band::Unlikely);
    }

    #[test]
    fn clearly_abnormal_alertness_scores_four() {
        let mut i = base();
        i.alertness = Alertness::ClearlyAbnormal;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.band, Band::PossibleDelirium);
    }

    #[test]
    fn acute_change_alone_scores_four() {
        let mut i = base();
        i.acute_change = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.band, Band::PossibleDelirium);
    }

    #[test]
    fn amt4_and_attention_bands() {
        let mut i = base();
        i.amt4 = Amt4::OneMistake;
        i.attention = Attention::BelowSevenOrRefuses;
        // 1 + 1 = 2 -> possible cognitive impairment.
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 2);
        assert_eq!(o.band, Band::PossibleCognitiveImpairment);
    }

    #[test]
    fn untestable_cognitive_items_score_two_each() {
        let mut i = base();
        i.amt4 = Amt4::TwoOrMoreOrUntestable;
        i.attention = Attention::Untestable;
        // 2 + 2 = 4 -> possible delirium.
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 4);
        assert_eq!(o.band, Band::PossibleDelirium);
    }

    #[test]
    fn maximum_score_is_twelve() {
        let i = FourAtInput {
            alertness: Alertness::ClearlyAbnormal,
            amt4: Amt4::TwoOrMoreOrUntestable,
            attention: Attention::Untestable,
            acute_change: true,
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 12);
        assert_eq!(o.band, Band::PossibleDelirium);
    }

    #[test]
    fn zero_without_acute_change_info_warns_delirium_still_possible() {
        // acute_change false: item 4 may be incomplete, so the caveat appears.
        let o = compute(&base()).unwrap();
        assert!(o.interpretation.contains("still possible"));
    }

    #[test]
    fn band_boundary_one_is_cognitive_impairment() {
        let mut i = base();
        i.amt4 = Amt4::OneMistake;
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 1);
        assert_eq!(o.band, Band::PossibleCognitiveImpairment);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "alertness": "clearly_abnormal",
            "amt4": "one_mistake",
            "attention": "seven_or_more",
            "acute_change": false
        });
        let mut typed = base();
        typed.alertness = Alertness::ClearlyAbnormal;
        typed.amt4 = Amt4::OneMistake;
        let dynamic = FourAt.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        // 4 + 1 = 5.
        assert_eq!(dynamic.result, json!(5));
    }

    #[test]
    fn working_reports_per_item_points() {
        let r = build_response(&base()).unwrap();
        assert_eq!(r.working["total_score"], json!(0));
        assert_eq!(r.working["alertness"], json!(0));
        assert_eq!(r.working["band"], json!("unlikely"));
    }

    #[test]
    fn schema_marks_all_four_items_required() {
        let schema = FourAt.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 4);
        for key in ["alertness", "amt4", "attention", "acute_change"] {
            assert!(required.iter().any(|v| v == key), "{key} must be required");
        }
    }

    #[test]
    fn license_records_free_clinical_use_and_the4at_url() {
        let lic = FourAt.license();
        assert!(lic.license.contains("CC-BY"));
        assert!(lic.source_url.contains("the4at.com"));
    }
}
