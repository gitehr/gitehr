//! Waterlow Score - pressure ulcer (pressure injury) risk.
//!
//! Judy Waterlow's bedside risk-assessment card (1985, revised 2005). The total
//! is a sum of weighted categories: the higher the score the greater the risk of
//! a patient developing a pressure ulcer. The card is deliberately additive -
//! several categories let more than one descriptor apply at once - and it is this
//! additivity, not a single worst-case selection, that distinguishes Waterlow
//! from simpler scales such as Norton.
//!
//! Modelling decisions (documented because the card is genuinely complex):
//! - Most categories are mutually-exclusive single choices and are modelled as a
//!   Rust enum per category (Build, Continence, Mobility), so contradictory
//!   inputs are impossible.
//! - "Skin type / visual risk areas" is explicitly additive on the card (a
//!   patient can be both dry AND oedematous), so it is modelled as a set of
//!   independent booleans and the contributions are summed. The card's own
//!   guidance is to ring every applicable descriptor.
//! - "Sex and age" contribute as two independent axes (a sex point AND an age
//!   band point), so sex is an enum and age is numeric with derived bands,
//!   mirroring the CHA2DS2-VASc treatment of age.
//! - The "special risks" boxes (tissue malnutrition, neurological deficit, major
//!   surgery / trauma, medication) are each modelled as an enum of the card's
//!   listed options, taking the single highest applicable value within each box
//!   (the card lists discrete severities, not additive ticks, within a box),
//!   while the four boxes themselves sum.
//! - Nutrition: the 2005 card embeds a small malnutrition screen (recent weight
//!   loss + poor appetite/intake). GitEHR ships the full BAPEN MUST tool only as
//!   a proprietary stub (BAPEN holds copyright), so this module does NOT
//!   reproduce MUST. Instead it models the nutrition contribution exactly as the
//!   Waterlow card itself states it: a weight-loss severity enum plus two
//!   independent flags ("eating poorly / lack of appetite" and "acutely ill or
//!   no nutritional intake > 5 days"), summed. These are Waterlow's own fields,
//!   not a reproduction of the MUST instrument.
//!
//! Risk bands (per the card): 10+ at risk; 15+ high risk; 20+ very high risk.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "waterlow";

/// Primary citation.
pub const REFERENCE: &str =
    "Waterlow J. Pressure sores: a risk assessment card. Nurs Times. 1985;81(48):49-55. \
Revised 2005 card per judy-waterlow.co.uk. Risk bands: 10+ at risk, 15+ high, 20+ very high.";

/// Distribution licence: the Waterlow card is made freely available by the
/// author for clinical use and download. It is implemented here from the
/// published card; the scoring method is a clinical algorithm.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Free for clinical use - Waterlow card made freely available by the author \
(Judy Waterlow); implemented from the published card",
    source_url: "http://www.judy-waterlow.co.uk/the-waterlow-score-card.htm",
};

/// Build / weight for height (BMI band). Mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Build {
    /// Average (BMI 20-24.9). 0 points.
    Average,
    /// Above average (BMI 25-29.9). 1 point.
    AboveAverage,
    /// Obese (BMI > 30). 2 points.
    Obese,
    /// Below average (BMI < 20). 3 points.
    BelowAverage,
}

impl Build {
    fn points(self) -> u8 {
        match self {
            Build::Average => 0,
            Build::AboveAverage => 1,
            Build::Obese => 2,
            Build::BelowAverage => 3,
        }
    }
}

/// Continence. Mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Continence {
    /// Complete / catheterised. 0 points.
    Complete,
    /// Incontinent of urine. 1 point.
    UrineIncontinent,
    /// Incontinent of faeces. 2 points.
    FaecesIncontinent,
    /// Doubly incontinent. 3 points.
    DoublyIncontinent,
}

impl Continence {
    fn points(self) -> u8 {
        match self {
            Continence::Complete => 0,
            Continence::UrineIncontinent => 1,
            Continence::FaecesIncontinent => 2,
            Continence::DoublyIncontinent => 3,
        }
    }
}

/// Mobility. Mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mobility {
    /// Fully mobile. 0 points.
    Fully,
    /// Restless / fidgety. 1 point.
    Restless,
    /// Apathetic. 2 points.
    Apathetic,
    /// Restricted. 3 points.
    Restricted,
    /// Bedbound, e.g. traction. 4 points.
    Bedbound,
    /// Chairbound, e.g. wheelchair. 5 points.
    Chairbound,
}

impl Mobility {
    fn points(self) -> u8 {
        match self {
            Mobility::Fully => 0,
            Mobility::Restless => 1,
            Mobility::Apathetic => 2,
            Mobility::Restricted => 3,
            Mobility::Bedbound => 4,
            Mobility::Chairbound => 5,
        }
    }
}

/// Sex. Contributes its own point, independent of age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    /// Male. 1 point.
    Male,
    /// Female. 2 points.
    Female,
}

impl Sex {
    fn points(self) -> u8 {
        match self {
            Sex::Male => 1,
            Sex::Female => 2,
        }
    }
}

/// Recent weight-loss severity (the Waterlow card's nutrition screen). Mutually
/// exclusive. Not a reproduction of the BAPEN MUST tool - these are Waterlow's
/// own nutrition fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightLoss {
    /// No recent weight loss. 0 points.
    None,
    /// 0.5-5 kg lost. 1 point.
    Kg0_5To5,
    /// 5-10 kg lost. 2 points.
    Kg5To10,
    /// 10-15 kg lost. 3 points.
    Kg10To15,
    /// More than 15 kg lost. 4 points.
    KgOver15,
    /// Weight loss likely but amount unsure. 2 points.
    Unsure,
}

impl WeightLoss {
    fn points(self) -> u8 {
        match self {
            WeightLoss::None => 0,
            WeightLoss::Kg0_5To5 => 1,
            WeightLoss::Kg5To10 => 2,
            WeightLoss::Kg10To15 => 3,
            WeightLoss::KgOver15 => 4,
            WeightLoss::Unsure => 2,
        }
    }
}

/// Tissue malnutrition special-risk box. Highest single applicable value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TissueMalnutrition {
    /// None. 0 points.
    None,
    /// Smoking. 1 point.
    Smoking,
    /// Anaemia (Hb < 8). 2 points.
    Anaemia,
    /// Peripheral vascular disease. 5 points.
    PeripheralVascularDisease,
    /// Single-organ failure (cardiac, renal, respiratory). 5 points.
    SingleOrganFailure,
    /// Multiple organ failure. 8 points.
    MultipleOrganFailure,
    /// Terminal cachexia. 8 points.
    TerminalCachexia,
}

impl TissueMalnutrition {
    fn points(self) -> u8 {
        match self {
            TissueMalnutrition::None => 0,
            TissueMalnutrition::Smoking => 1,
            TissueMalnutrition::Anaemia => 2,
            TissueMalnutrition::PeripheralVascularDisease => 5,
            TissueMalnutrition::SingleOrganFailure => 5,
            TissueMalnutrition::MultipleOrganFailure => 8,
            TissueMalnutrition::TerminalCachexia => 8,
        }
    }
}

/// Neurological deficit special-risk box. The card gives a 4-6 range for
/// diabetes / MS / CVA / motor / sensory / paraplegia; modelled as none / mild /
/// moderate / severe to span it explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeurologicalDeficit {
    /// None. 0 points.
    None,
    /// Mild deficit (e.g. diabetes, MS, CVA - stable). 4 points.
    Mild,
    /// Moderate motor / sensory deficit. 5 points.
    Moderate,
    /// Severe deficit, e.g. paraplegia. 6 points.
    Severe,
}

impl NeurologicalDeficit {
    fn points(self) -> u8 {
        match self {
            NeurologicalDeficit::None => 0,
            NeurologicalDeficit::Mild => 4,
            NeurologicalDeficit::Moderate => 5,
            NeurologicalDeficit::Severe => 6,
        }
    }
}

/// Major surgery or trauma special-risk box. Highest single applicable value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surgery {
    /// None. 0 points.
    None,
    /// Orthopaedic (below waist) or spinal surgery. 5 points.
    OrthopaedicOrSpinal,
    /// On the operating table > 2 hours. 5 points.
    OnTableOver2h,
    /// On the operating table > 6 hours. 8 points.
    OnTableOver6h,
}

impl Surgery {
    fn points(self) -> u8 {
        match self {
            Surgery::None => 0,
            Surgery::OrthopaedicOrSpinal => 5,
            Surgery::OnTableOver2h => 5,
            Surgery::OnTableOver6h => 8,
        }
    }
}

/// Medication special-risk box. The card scores up to 4 for cytotoxics,
/// long-term / high-dose steroids, or anti-inflammatories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Medication {
    /// None of the listed medications. 0 points.
    None,
    /// Cytotoxics, long-term / high-dose steroids, or anti-inflammatories. 4 points.
    HighRisk,
}

impl Medication {
    fn points(self) -> u8 {
        match self {
            Medication::None => 0,
            Medication::HighRisk => 4,
        }
    }
}

/// Waterlow inputs. Age is numeric (the age band is derived); skin descriptors
/// and the two nutrition flags are additive booleans; everything else is a
/// single-choice enum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaterlowInput {
    pub build: Build,
    pub continence: Continence,
    pub mobility: Mobility,
    pub sex: Sex,
    /// Age in years (drives the age band: 14-49 = 1, 50-64 = 2, 65-74 = 3,
    /// 75-80 = 4, 81+ = 5).
    pub age: u8,

    // Skin type / visual risk areas - additive; ring every applicable one.
    /// Tissue paper (thin / fragile) skin. 1 point.
    pub skin_tissue_paper: bool,
    /// Dry skin. 1 point.
    pub skin_dry: bool,
    /// Oedematous skin. 1 point.
    pub skin_oedematous: bool,
    /// Clammy / pyrexial skin. 1 point.
    pub skin_clammy: bool,
    /// Discoloured skin (grade 1 - non-blanching erythema). 2 points.
    pub skin_discoloured: bool,
    /// Broken skin / spot (grade 2-4). 3 points.
    pub skin_broken: bool,

    // Nutrition (Waterlow card's own screen - NOT the MUST tool).
    pub weight_loss: WeightLoss,
    /// Patient eating poorly or lack of appetite. 1 point.
    pub eating_poorly: bool,
    /// Acutely ill or no nutritional intake > 5 days. 2 points.
    pub acutely_ill: bool,

    // Special risks.
    pub tissue_malnutrition: TissueMalnutrition,
    pub neurological_deficit: NeurologicalDeficit,
    pub surgery: Surgery,
    pub medication: Medication,
}

/// Risk band per the Waterlow card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    /// Below 10: not flagged at risk by the score.
    NotAtRisk,
    /// 10-14: at risk.
    AtRisk,
    /// 15-19: high risk.
    HighRisk,
    /// 20+: very high risk.
    VeryHighRisk,
}

impl RiskBand {
    fn from_score(score: u16) -> RiskBand {
        if score >= 20 {
            RiskBand::VeryHighRisk
        } else if score >= 15 {
            RiskBand::HighRisk
        } else if score >= 10 {
            RiskBand::AtRisk
        } else {
            RiskBand::NotAtRisk
        }
    }

    fn slug(self) -> &'static str {
        match self {
            RiskBand::NotAtRisk => "not-at-risk",
            RiskBand::AtRisk => "at-risk",
            RiskBand::HighRisk => "high-risk",
            RiskBand::VeryHighRisk => "very-high-risk",
        }
    }
}

/// The computed outcome, with each category's contribution exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterlowOutcome {
    pub score: u16,
    pub band: RiskBand,
    pub build_points: u8,
    pub continence_points: u8,
    pub skin_points: u8,
    pub mobility_points: u8,
    pub sex_points: u8,
    pub age_points: u8,
    pub nutrition_points: u8,
    pub tissue_malnutrition_points: u8,
    pub neurological_points: u8,
    pub surgery_points: u8,
    pub medication_points: u8,
    pub interpretation: String,
}

fn age_points(age: u8) -> u8 {
    if age >= 81 {
        5
    } else if age >= 75 {
        4
    } else if age >= 65 {
        3
    } else if age >= 50 {
        2
    } else {
        // The card's lowest band is 14-49; anyone below 50 scores 1.
        1
    }
}

fn skin_points(i: &WaterlowInput) -> u8 {
    u8::from(i.skin_tissue_paper)
        + u8::from(i.skin_dry)
        + u8::from(i.skin_oedematous)
        + u8::from(i.skin_clammy)
        + 2 * u8::from(i.skin_discoloured)
        + 3 * u8::from(i.skin_broken)
}

fn nutrition_points(i: &WaterlowInput) -> u8 {
    i.weight_loss.points() + u8::from(i.eating_poorly) + 2 * u8::from(i.acutely_ill)
}

/// Pure scoring.
pub fn compute(input: &WaterlowInput) -> Result<WaterlowOutcome, CalcError> {
    let build_points = input.build.points();
    let continence_points = input.continence.points();
    let skin_points = skin_points(input);
    let mobility_points = input.mobility.points();
    let sex_points = input.sex.points();
    let age_points = age_points(input.age);
    let nutrition_points = nutrition_points(input);
    let tissue_malnutrition_points = input.tissue_malnutrition.points();
    let neurological_points = input.neurological_deficit.points();
    let surgery_points = input.surgery.points();
    let medication_points = input.medication.points();

    let score: u16 = u16::from(build_points)
        + u16::from(continence_points)
        + u16::from(skin_points)
        + u16::from(mobility_points)
        + u16::from(sex_points)
        + u16::from(age_points)
        + u16::from(nutrition_points)
        + u16::from(tissue_malnutrition_points)
        + u16::from(neurological_points)
        + u16::from(surgery_points)
        + u16::from(medication_points);

    let band = RiskBand::from_score(score);

    let interpretation = match band {
        RiskBand::NotAtRisk => format!(
            "Score {score}: below the at-risk threshold of 10. Continue routine skin care and \
reassess if the patient's condition changes."
        ),
        RiskBand::AtRisk => format!(
            "Score {score}: AT RISK (10+). Institute pressure-relief measures, a repositioning \
schedule, and regular skin inspection per local pressure-ulcer prevention policy."
        ),
        RiskBand::HighRisk => format!(
            "Score {score}: HIGH RISK (15+). Use a pressure-redistributing surface, frequent \
repositioning, and document a prevention plan; consider tissue-viability review."
        ),
        RiskBand::VeryHighRisk => format!(
            "Score {score}: VERY HIGH RISK (20+). High-specification support surface and \
intensive repositioning; refer to tissue viability and review nutrition and all special risks."
        ),
    };

    Ok(WaterlowOutcome {
        score,
        band,
        build_points,
        continence_points,
        skin_points,
        mobility_points,
        sex_points,
        age_points,
        nutrition_points,
        tissue_malnutrition_points,
        neurological_points,
        surgery_points,
        medication_points,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &WaterlowInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("total_score".into(), json!(o.score));
    working.insert("risk_band".into(), json!(o.band.slug()));
    working.insert("build".into(), json!(o.build_points));
    working.insert("continence".into(), json!(o.continence_points));
    working.insert("skin".into(), json!(o.skin_points));
    working.insert("mobility".into(), json!(o.mobility_points));
    working.insert("sex".into(), json!(o.sex_points));
    working.insert("age".into(), json!(o.age_points));
    working.insert("nutrition".into(), json!(o.nutrition_points));
    working.insert("tissue_malnutrition".into(), json!(o.tissue_malnutrition_points));
    working.insert("neurological_deficit".into(), json!(o.neurological_points));
    working.insert("surgery_trauma".into(), json!(o.surgery_points));
    working.insert("medication".into(), json!(o.medication_points));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.score),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Waterlow;

impl Calculator for Waterlow {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "Waterlow Score (Pressure Ulcer Risk)"
    }

    fn description(&self) -> &'static str {
        "Bedside pressure-ulcer (pressure-injury) risk assessment: summed weighted categories \
(10+ at risk, 15+ high, 20+ very high)."
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
            "title": "WaterlowInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "build", "continence", "mobility", "sex", "age",
                "skin_tissue_paper", "skin_dry", "skin_oedematous", "skin_clammy",
                "skin_discoloured", "skin_broken",
                "weight_loss", "eating_poorly", "acutely_ill",
                "tissue_malnutrition", "neurological_deficit", "surgery", "medication"
            ],
            "properties": {
                "build": {
                    "type": "string",
                    "enum": ["average", "above_average", "obese", "below_average"],
                    "description": "Build / weight for height (BMI band): average (BMI 20-24.9) = 0, above_average (25-29.9) = 1, obese (>30) = 2, below_average (<20) = 3."
                },
                "continence": {
                    "type": "string",
                    "enum": ["complete", "urine_incontinent", "faeces_incontinent", "doubly_incontinent"],
                    "description": "Continence: complete/catheterised = 0, urine_incontinent = 1, faeces_incontinent = 2, doubly_incontinent = 3."
                },
                "mobility": {
                    "type": "string",
                    "enum": ["fully", "restless", "apathetic", "restricted", "bedbound", "chairbound"],
                    "description": "Mobility: fully = 0, restless/fidgety = 1, apathetic = 2, restricted = 3, bedbound (e.g. traction) = 4, chairbound (e.g. wheelchair) = 5."
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Sex: male = 1, female = 2. Contributes independently of the age band."
                },
                "age": {
                    "type": "integer",
                    "minimum": 14,
                    "maximum": 120,
                    "description": "Age in years. Age band: 14-49 = 1, 50-64 = 2, 65-74 = 3, 75-80 = 4, 81+ = 5. Added to the sex point."
                },
                "skin_tissue_paper": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: tissue paper (thin/fragile) = 1. Additive; multiple skin descriptors may apply."
                },
                "skin_dry": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: dry = 1. Additive with other skin descriptors."
                },
                "skin_oedematous": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: oedematous = 1. Additive with other skin descriptors."
                },
                "skin_clammy": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: clammy/pyrexial = 1. Additive with other skin descriptors."
                },
                "skin_discoloured": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: discoloured, grade 1 (non-blanching erythema) = 2. Additive with other skin descriptors."
                },
                "skin_broken": {
                    "type": "boolean",
                    "description": "Skin type/visual risk: broken/spot, grade 2-4 = 3. Additive with other skin descriptors."
                },
                "weight_loss": {
                    "type": "string",
                    "enum": ["none", "kg0_5_to5", "kg5_to10", "kg10_to15", "kg_over15", "unsure"],
                    "description": "Nutrition (Waterlow card's own screen, not the MUST tool): recent weight loss none = 0, 0.5-5 kg = 1, 5-10 kg = 2, 10-15 kg = 3, >15 kg = 4, unsure = 2."
                },
                "eating_poorly": {
                    "type": "boolean",
                    "description": "Nutrition: patient eating poorly or lack of appetite = 1. Added to the weight-loss score."
                },
                "acutely_ill": {
                    "type": "boolean",
                    "description": "Nutrition: acutely ill or no nutritional intake > 5 days = 2. Added to the weight-loss score."
                },
                "tissue_malnutrition": {
                    "type": "string",
                    "enum": ["none", "smoking", "anaemia", "peripheral_vascular_disease", "single_organ_failure", "multiple_organ_failure", "terminal_cachexia"],
                    "description": "Special risk - tissue malnutrition (highest single applicable): none = 0, smoking = 1, anaemia (Hb<8) = 2, peripheral_vascular_disease = 5, single_organ_failure = 5, multiple_organ_failure = 8, terminal_cachexia = 8."
                },
                "neurological_deficit": {
                    "type": "string",
                    "enum": ["none", "mild", "moderate", "severe"],
                    "description": "Special risk - neurological deficit (card range 4-6 for diabetes/MS/CVA/motor/sensory/paraplegia): none = 0, mild = 4, moderate (motor/sensory) = 5, severe (e.g. paraplegia) = 6."
                },
                "surgery": {
                    "type": "string",
                    "enum": ["none", "orthopaedic_or_spinal", "on_table_over2h", "on_table_over6h"],
                    "description": "Special risk - major surgery/trauma (highest single applicable): none = 0, orthopaedic (below waist) or spinal = 5, on table > 2h = 5, on table > 6h = 8."
                },
                "medication": {
                    "type": "string",
                    "enum": ["none", "high_risk"],
                    "description": "Special risk - medication: none = 0; high_risk (cytotoxics, long-term/high-dose steroids, or anti-inflammatories) = 4."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: WaterlowInput =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A low-risk baseline: every category at its zero/lowest descriptor.
    fn base() -> WaterlowInput {
        WaterlowInput {
            build: Build::Average,
            continence: Continence::Complete,
            mobility: Mobility::Fully,
            sex: Sex::Male,
            age: 40,
            skin_tissue_paper: false,
            skin_dry: false,
            skin_oedematous: false,
            skin_clammy: false,
            skin_discoloured: false,
            skin_broken: false,
            weight_loss: WeightLoss::None,
            eating_poorly: false,
            acutely_ill: false,
            tissue_malnutrition: TissueMalnutrition::None,
            neurological_deficit: NeurologicalDeficit::None,
            surgery: Surgery::None,
            medication: Medication::None,
        }
    }

    #[test]
    fn baseline_is_male_under_50_only() {
        // Only sex (male, 1) + age band 14-49 (1) contribute.
        let o = compute(&base()).unwrap();
        assert_eq!(o.score, 2);
        assert_eq!(o.band, RiskBand::NotAtRisk);
    }

    #[test]
    fn age_bands() {
        assert_eq!(age_points(40), 1);
        assert_eq!(age_points(49), 1);
        assert_eq!(age_points(50), 2);
        assert_eq!(age_points(64), 2);
        assert_eq!(age_points(65), 3);
        assert_eq!(age_points(74), 3);
        assert_eq!(age_points(75), 4);
        assert_eq!(age_points(80), 4);
        assert_eq!(age_points(81), 5);
    }

    #[test]
    fn category_points_match_card() {
        assert_eq!(Build::BelowAverage.points(), 3);
        assert_eq!(Continence::DoublyIncontinent.points(), 3);
        assert_eq!(Mobility::Chairbound.points(), 5);
        assert_eq!(Sex::Female.points(), 2);
        assert_eq!(WeightLoss::KgOver15.points(), 4);
        assert_eq!(TissueMalnutrition::TerminalCachexia.points(), 8);
        assert_eq!(TissueMalnutrition::Anaemia.points(), 2);
        assert_eq!(NeurologicalDeficit::Severe.points(), 6);
        assert_eq!(Surgery::OnTableOver6h.points(), 8);
        assert_eq!(Medication::HighRisk.points(), 4);
    }

    #[test]
    fn skin_descriptors_are_additive() {
        let mut i = base();
        i.skin_dry = true; // 1
        i.skin_oedematous = true; // 1
        i.skin_broken = true; // 3
        // base 2 + 1 + 1 + 3 = 7
        let o = compute(&i).unwrap();
        assert_eq!(o.skin_points, 5);
        assert_eq!(o.score, 7);
    }

    #[test]
    fn nutrition_is_weight_loss_plus_flags() {
        let mut i = base();
        i.weight_loss = WeightLoss::Kg5To10; // 2
        i.eating_poorly = true; // 1
        i.acutely_ill = true; // 2
        let o = compute(&i).unwrap();
        assert_eq!(o.nutrition_points, 5);
    }

    #[test]
    fn unsure_weight_loss_scores_two() {
        let mut i = base();
        i.weight_loss = WeightLoss::Unsure;
        let o = compute(&i).unwrap();
        assert_eq!(o.nutrition_points, 2);
    }

    #[test]
    fn risk_band_thresholds() {
        assert_eq!(RiskBand::from_score(9), RiskBand::NotAtRisk);
        assert_eq!(RiskBand::from_score(10), RiskBand::AtRisk);
        assert_eq!(RiskBand::from_score(14), RiskBand::AtRisk);
        assert_eq!(RiskBand::from_score(15), RiskBand::HighRisk);
        assert_eq!(RiskBand::from_score(19), RiskBand::HighRisk);
        assert_eq!(RiskBand::from_score(20), RiskBand::VeryHighRisk);
    }

    #[test]
    fn worked_example_high_risk_elderly_woman() {
        // 82F (sex 2 + age 5 = 7), below-average build (3), doubly incontinent (3),
        // chairbound (5), dry + discoloured skin (1 + 2 = 3), weight loss 5-10kg (2),
        // anaemia (2). Total = 7 + 3 + 3 + 5 + 3 + 2 + 2 = 25.
        let i = WaterlowInput {
            build: Build::BelowAverage,
            continence: Continence::DoublyIncontinent,
            mobility: Mobility::Chairbound,
            sex: Sex::Female,
            age: 82,
            skin_dry: true,
            skin_discoloured: true,
            weight_loss: WeightLoss::Kg5To10,
            tissue_malnutrition: TissueMalnutrition::Anaemia,
            ..base()
        };
        let o = compute(&i).unwrap();
        assert_eq!(o.score, 25);
        assert_eq!(o.band, RiskBand::VeryHighRisk);
        assert!(o.interpretation.contains("VERY HIGH RISK"));
    }

    #[test]
    fn special_risk_boxes_sum_across_boxes() {
        let mut i = base();
        i.tissue_malnutrition = TissueMalnutrition::SingleOrganFailure; // 5
        i.neurological_deficit = NeurologicalDeficit::Moderate; // 5
        i.surgery = Surgery::OnTableOver6h; // 8
        i.medication = Medication::HighRisk; // 4
        let o = compute(&i).unwrap();
        // base 2 + 5 + 5 + 8 + 4 = 24
        assert_eq!(o.score, 24);
        assert_eq!(o.tissue_malnutrition_points, 5);
        assert_eq!(o.neurological_points, 5);
        assert_eq!(o.surgery_points, 8);
        assert_eq!(o.medication_points, 4);
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "build": "below_average",
            "continence": "doubly_incontinent",
            "mobility": "chairbound",
            "sex": "female",
            "age": 82,
            "skin_tissue_paper": false,
            "skin_dry": true,
            "skin_oedematous": false,
            "skin_clammy": false,
            "skin_discoloured": true,
            "skin_broken": false,
            "weight_loss": "kg5_to10",
            "eating_poorly": false,
            "acutely_ill": false,
            "tissue_malnutrition": "anaemia",
            "neurological_deficit": "none",
            "surgery": "none",
            "medication": "none"
        });
        let typed = WaterlowInput {
            build: Build::BelowAverage,
            continence: Continence::DoublyIncontinent,
            mobility: Mobility::Chairbound,
            sex: Sex::Female,
            age: 82,
            skin_dry: true,
            skin_discoloured: true,
            weight_loss: WeightLoss::Kg5To10,
            tissue_malnutrition: TissueMalnutrition::Anaemia,
            ..base()
        };
        let dynamic = Waterlow.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(25));
    }

    #[test]
    fn invalid_enum_is_rejected() {
        let mut value = json!({
            "build": "svelte",
            "continence": "complete",
            "mobility": "fully",
            "sex": "male",
            "age": 40,
            "skin_tissue_paper": false, "skin_dry": false, "skin_oedematous": false,
            "skin_clammy": false, "skin_discoloured": false, "skin_broken": false,
            "weight_loss": "none", "eating_poorly": false, "acutely_ill": false,
            "tissue_malnutrition": "none", "neurological_deficit": "none",
            "surgery": "none", "medication": "none"
        });
        assert!(Waterlow.calculate(&value).is_err());
        // A valid build then parses.
        value["build"] = json!("average");
        assert!(Waterlow.calculate(&value).is_ok());
    }

    #[test]
    fn schema_lists_all_enums_and_required() {
        let schema = Waterlow.input_schema();
        let props = &schema["properties"];
        assert!(props["build"]["enum"].as_array().unwrap().len() == 4);
        assert!(props["mobility"]["enum"].as_array().unwrap().len() == 6);
        assert!(props["tissue_malnutrition"]["enum"].as_array().unwrap().len() == 7);
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "weight_loss"));
        assert!(required.iter().any(|v| v == "skin_broken"));
    }
}
