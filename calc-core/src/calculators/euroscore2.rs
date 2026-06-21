//! EuroSCORE II - predicted operative mortality after cardiac surgery.
//!
//! A logistic-regression model (Nashef et al., 2012) estimating in-hospital
//! mortality from 18 preoperative variables. The linear predictor is
//! `y = b0 + sum(beta_i * x_i)` and the predicted mortality is the logistic
//! transform `exp(y) / (1 + exp(y))`, reported here as a percentage. The
//! constant `b0 = -5.324537`; every coefficient is the published multivariable
//! beta from Table 6 of the primary paper.
//!
//! Two encodings carry clinical subtlety and are handled here so they cannot be
//! got wrong:
//! - Age is a single numeric input. The model's age term is *not* the age in
//!   years: `x = 1` for any age <= 60, then increases by one per year thereafter
//!   (age 61 -> 2, age 62 -> 3, ...), i.e. `x = max(0, age - 60) + 1`.
//! - The banded categorical variables (renal function, NYHA, LV function,
//!   pulmonary-artery pressure, urgency, weight of intervention) are modelled as
//!   mutually-exclusive bands, so the reference band contributes nothing and only
//!   one coefficient from each group is ever added.
//!
//! Coefficients cross-checked against two sources that agree to all published
//! digits: the primary paper (Nashef 2012, Table 6) and the Evidencio
//! reproduction of the model. A healthy 60-year-old undergoing isolated CABG with
//! no other risk factors yields ~0.50%, the universally-cited EuroSCORE II
//! baseline, which the unit tests pin.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "euroscore2";

/// Primary citation.
pub const REFERENCE: &str =
    "Nashef SAM, Roques F, Sharples LD, et al. EuroSCORE II. Eur J Cardiothorac Surg. \
2012;41(4):734-744. doi:10.1093/ejcts/ezs043. Coefficients from Table 6 (multivariable model).";

/// Distribution licence: EuroSCORE II is published as a free clinical tool by its
/// authors (Papworth Hospital / EuroSCORE Project Group) and the equation and
/// coefficients are openly available; implemented here from the primary
/// literature and the official euroscore.org publication.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Freely usable clinical model - published openly by the EuroSCORE Project Group; \
implemented from the primary literature",
    source_url: "https://www.euroscore.org/index.php?fid=201",
};

// --- Published coefficients (Nashef 2012, Table 6) --------------------------

/// Constant `b0` of the logistic regression.
pub const CONST: f64 = -5.324537;
/// Age coefficient, applied to the encoded age step (not age in years).
pub const C_AGE: f64 = 0.0285181;
/// Female sex.
pub const C_FEMALE: f64 = 0.2196434;
/// Insulin-dependent diabetes mellitus.
pub const C_IDDM: f64 = 0.3542749;
/// Chronic pulmonary dysfunction.
pub const C_CPD: f64 = 0.1886564;
/// Extracardiac arteriopathy.
pub const C_ECA: f64 = 0.5360268;
/// Neurological or musculoskeletal dysfunction severely affecting mobility.
pub const C_POOR_MOBILITY: f64 = 0.2407181;
/// Previous cardiac surgery (redo).
pub const C_REDO: f64 = 1.118599;
/// Active endocarditis.
pub const C_ENDOCARDITIS: f64 = 0.6194522;
/// Critical preoperative state.
pub const C_CRITICAL: f64 = 1.086517;
/// Recent myocardial infarction (within 90 days).
pub const C_RECENT_MI: f64 = 0.1528943;
/// CCS class 4 angina.
pub const C_CCS4: f64 = 0.2226147;
/// Surgery on the thoracic aorta.
pub const C_THORACIC_AORTA: f64 = 0.6527205;

// Renal dysfunction bands (creatinine clearance, mL/min). Band ">85" is the
// reference (coefficient 0).
const C_RENAL_DIALYSIS: f64 = 0.6421508;
const C_RENAL_LE50: f64 = 0.8592256;
const C_RENAL_51_85: f64 = 0.303553;

// NYHA bands. Class I is the reference (coefficient 0).
const C_NYHA2: f64 = 0.1070545;
const C_NYHA3: f64 = 0.2958358;
const C_NYHA4: f64 = 0.5597929;

// LV function bands. "Good" (>50%) is the reference (coefficient 0).
const C_LV_MODERATE: f64 = 0.3150652;
const C_LV_POOR: f64 = 0.8084096;
const C_LV_VERY_POOR: f64 = 0.9346919;

// Pulmonary-artery systolic pressure bands. "None/normal" is the reference.
const C_PA_MODERATE: f64 = 0.1788899;
const C_PA_SEVERE: f64 = 0.3491475;

// Urgency bands. "Elective" is the reference (coefficient 0).
const C_URGENT: f64 = 0.3174673;
const C_EMERGENCY: f64 = 0.7039121;
const C_SALVAGE: f64 = 1.362947;

// Weight-of-intervention bands. "Isolated CABG" is the reference (coefficient 0).
const C_WT_SINGLE_NON_CABG: f64 = 0.0062118;
const C_WT_TWO: f64 = 0.5521478;
const C_WT_THREE_PLUS: f64 = 0.9724533;

// --- Banded categorical inputs ---------------------------------------------

/// Renal impairment by creatinine clearance (Cockcroft-Gault, mL/min), or
/// dialysis regardless of clearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenalFunction {
    /// CrCl > 85 mL/min (reference).
    Normal,
    /// CrCl 51-85 mL/min.
    Moderate,
    /// CrCl <= 50 mL/min.
    Severe,
    /// On dialysis (regardless of creatinine clearance).
    OnDialysis,
}

impl RenalFunction {
    fn beta(self) -> f64 {
        match self {
            RenalFunction::Normal => 0.0,
            RenalFunction::Moderate => C_RENAL_51_85,
            RenalFunction::Severe => C_RENAL_LE50,
            RenalFunction::OnDialysis => C_RENAL_DIALYSIS,
        }
    }
}

/// New York Heart Association functional class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Nyha {
    /// Class I: no limitation (reference).
    I,
    /// Class II: slight limitation on ordinary activity.
    II,
    /// Class III: marked limitation on less-than-ordinary activity.
    III,
    /// Class IV: symptoms at rest.
    IV,
}

impl Nyha {
    fn beta(self) -> f64 {
        match self {
            Nyha::I => 0.0,
            Nyha::II => C_NYHA2,
            Nyha::III => C_NYHA3,
            Nyha::IV => C_NYHA4,
        }
    }
}

/// Left-ventricular function by ejection fraction band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LvFunction {
    /// Good: EF > 50% (reference).
    Good,
    /// Moderate: EF 31-50%.
    Moderate,
    /// Poor: EF 21-30%.
    Poor,
    /// Very poor: EF <= 20%.
    VeryPoor,
}

impl LvFunction {
    fn beta(self) -> f64 {
        match self {
            LvFunction::Good => 0.0,
            LvFunction::Moderate => C_LV_MODERATE,
            LvFunction::Poor => C_LV_POOR,
            LvFunction::VeryPoor => C_LV_VERY_POOR,
        }
    }
}

/// Pulmonary-artery systolic pressure band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PulmonaryHypertension {
    /// None / normal: < 31 mmHg (reference).
    None,
    /// Moderate: 31-55 mmHg.
    Moderate,
    /// Severe: > 55 mmHg.
    Severe,
}

impl PulmonaryHypertension {
    fn beta(self) -> f64 {
        match self {
            PulmonaryHypertension::None => 0.0,
            PulmonaryHypertension::Moderate => C_PA_MODERATE,
            PulmonaryHypertension::Severe => C_PA_SEVERE,
        }
    }
}

/// Urgency of the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    /// Elective: routine admission for operation (reference).
    Elective,
    /// Urgent: not electively admitted but require surgery on the current
    /// admission for medical reasons; cannot be sent home.
    Urgent,
    /// Emergency: operation before the beginning of the next working day after
    /// the decision to operate.
    Emergency,
    /// Salvage: requiring cardiopulmonary resuscitation (external cardiac
    /// massage) en route to the operating theatre or before induction.
    Salvage,
}

impl Urgency {
    fn beta(self) -> f64 {
        match self {
            Urgency::Elective => 0.0,
            Urgency::Urgent => C_URGENT,
            Urgency::Emergency => C_EMERGENCY,
            Urgency::Salvage => C_SALVAGE,
        }
    }
}

/// Weight (complexity) of the intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightOfIntervention {
    /// Isolated CABG (the lowest-risk reference operation).
    IsolatedCabg,
    /// A single major cardiac procedure other than isolated CABG (e.g. single
    /// valve, or CABG plus one other procedure counted as one major procedure).
    SingleNonCabg,
    /// Two major cardiac procedures.
    TwoProcedures,
    /// Three or more major cardiac procedures.
    ThreeOrMore,
}

impl WeightOfIntervention {
    fn beta(self) -> f64 {
        match self {
            WeightOfIntervention::IsolatedCabg => 0.0,
            WeightOfIntervention::SingleNonCabg => C_WT_SINGLE_NON_CABG,
            WeightOfIntervention::TwoProcedures => C_WT_TWO,
            WeightOfIntervention::ThreeOrMore => C_WT_THREE_PLUS,
        }
    }
}

/// EuroSCORE II inputs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EuroScore2Input {
    /// Age in years. The model's age term is `1` for any age <= 60 and rises by
    /// one per year thereafter; this encoding is applied internally.
    pub age: u8,
    /// True if the patient is female.
    pub female: bool,
    /// Insulin-dependent diabetes mellitus.
    pub insulin_dependent_diabetes: bool,
    /// Chronic pulmonary dysfunction (long-term bronchodilators/steroids).
    pub chronic_pulmonary_dysfunction: bool,
    /// Extracardiac arteriopathy.
    pub extracardiac_arteriopathy: bool,
    /// Neurological or musculoskeletal dysfunction severely affecting mobility.
    pub poor_mobility: bool,
    /// Previous cardiac surgery (redo) requiring opening of the pericardium.
    pub previous_cardiac_surgery: bool,
    /// Renal impairment band.
    pub renal_function: RenalFunction,
    /// Active endocarditis (still on antibiotic treatment at operation).
    pub active_endocarditis: bool,
    /// Critical preoperative state.
    pub critical_preoperative_state: bool,
    /// New York Heart Association functional class.
    pub nyha: Nyha,
    /// CCS class 4 angina (angina at rest / on any activity).
    pub ccs4_angina: bool,
    /// Left-ventricular function band.
    pub lv_function: LvFunction,
    /// Myocardial infarction within 90 days before the operation.
    pub recent_mi: bool,
    /// Pulmonary-artery systolic pressure band.
    pub pulmonary_hypertension: PulmonaryHypertension,
    /// Urgency of the operation.
    pub urgency: Urgency,
    /// Weight (complexity) of the intervention.
    pub weight_of_intervention: WeightOfIntervention,
    /// Surgery on the thoracic aorta.
    pub thoracic_aorta_surgery: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct EuroScore2Outcome {
    /// Predicted operative mortality, as a percentage (0-100).
    pub mortality_percent: f64,
    /// The linear predictor `y = b0 + sum(beta_i * x_i)`.
    pub linear_predictor: f64,
    /// The encoded age step actually used in the model (`1` for age <= 60).
    pub age_step: u32,
    pub interpretation: String,
}

/// Encode age into the model's age step: `1` for any age <= 60, then +1 per year
/// (age 61 -> 2, age 62 -> 3, ...).
fn age_step(age: u8) -> u32 {
    1 + (i32::from(age) - 60).max(0) as u32
}

/// Pure scoring: the EuroSCORE II logistic model.
pub fn compute(input: &EuroScore2Input) -> Result<EuroScore2Outcome, CalcError> {
    // The model is only defined for adult cardiac-surgery candidates. Reject
    // implausible ages outright rather than extrapolating the age term.
    if input.age < 18 || input.age > 110 {
        return Err(CalcError::InvalidInput(
            "age must be between 18 and 110 years".into(),
        ));
    }

    let age_step = age_step(input.age);

    let y = CONST
        + C_AGE * f64::from(age_step)
        + C_FEMALE * f64::from(input.female)
        + C_IDDM * f64::from(input.insulin_dependent_diabetes)
        + C_CPD * f64::from(input.chronic_pulmonary_dysfunction)
        + C_ECA * f64::from(input.extracardiac_arteriopathy)
        + C_POOR_MOBILITY * f64::from(input.poor_mobility)
        + C_REDO * f64::from(input.previous_cardiac_surgery)
        + input.renal_function.beta()
        + C_ENDOCARDITIS * f64::from(input.active_endocarditis)
        + C_CRITICAL * f64::from(input.critical_preoperative_state)
        + input.nyha.beta()
        + C_CCS4 * f64::from(input.ccs4_angina)
        + input.lv_function.beta()
        + C_RECENT_MI * f64::from(input.recent_mi)
        + input.pulmonary_hypertension.beta()
        + input.urgency.beta()
        + input.weight_of_intervention.beta()
        + C_THORACIC_AORTA * f64::from(input.thoracic_aorta_surgery);

    let mortality = y.exp() / (1.0 + y.exp());
    let mortality_percent = mortality * 100.0;

    let band = if mortality_percent < 2.0 {
        "low"
    } else if mortality_percent < 5.0 {
        "intermediate"
    } else if mortality_percent < 10.0 {
        "high"
    } else {
        "very high"
    };

    let interpretation = format!(
        "Predicted operative (in-hospital) mortality {mortality_percent:.2}% ({band} risk). \
EuroSCORE II estimates the risk of death after cardiac surgery from preoperative factors; it is \
a population model for risk-adjustment and patient counselling, not a guarantee for an individual."
    );

    Ok(EuroScore2Outcome {
        mortality_percent,
        linear_predictor: y,
        age_step,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &EuroScore2Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("constant".into(), json!(CONST));
    working.insert("age_step".into(), json!(o.age_step));
    working.insert("linear_predictor".into(), json!(o.linear_predictor));
    working.insert("predicted_mortality_percent".into(), json!(o.mortality_percent));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        // Result is the predicted mortality percentage, rounded to 2 dp for a
        // stable, human-readable value; the full-precision figure is in working.
        result: json!((o.mortality_percent * 100.0).round() / 100.0),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct EuroScore2;

impl Calculator for EuroScore2 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "EuroSCORE II (Cardiac Surgery Mortality)"
    }

    fn description(&self) -> &'static str {
        "Predicted operative mortality after cardiac surgery from 18 preoperative factors (Nashef 2012)."
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
            "title": "EuroScore2Input",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "female", "insulin_dependent_diabetes", "chronic_pulmonary_dysfunction",
                "extracardiac_arteriopathy", "poor_mobility", "previous_cardiac_surgery",
                "renal_function", "active_endocarditis", "critical_preoperative_state",
                "nyha", "ccs4_angina", "lv_function", "recent_mi", "pulmonary_hypertension",
                "urgency", "weight_of_intervention", "thoracic_aorta_surgery"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 110,
                    "description": "Age in years. The model's age term is 1 for age <= 60 and rises by 1 per year thereafter (age 61 = 2, age 62 = 3, ...)."
                },
                "female": {
                    "type": "boolean",
                    "description": "True if the patient is female."
                },
                "insulin_dependent_diabetes": {
                    "type": "boolean",
                    "description": "Diabetes mellitus treated with insulin.",
                    "definition": {
                        "concept": "Insulin-dependent diabetes mellitus (IDDM)",
                        "statement": "Diabetes managed with insulin at the time of surgery.",
                        "excludes": ["Diet- or oral-agent-controlled diabetes does NOT count - only insulin-treated diabetes scores"],
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "chronic_pulmonary_dysfunction": {
                    "type": "boolean",
                    "description": "Long-term use of bronchodilators or steroids for lung disease.",
                    "definition": {
                        "concept": "Chronic pulmonary dysfunction (CPD)",
                        "statement": "Long-term use of bronchodilators or steroids for lung disease.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "extracardiac_arteriopathy": {
                    "type": "boolean",
                    "description": "Claudication, carotid occlusion/>50% stenosis, amputation for arterial disease, or prior/planned intervention on the abdominal aorta, limb arteries, or carotids.",
                    "definition": {
                        "concept": "Extracardiac arteriopathy (ECA)",
                        "statement": "One or more of: claudication; carotid occlusion or >50% stenosis; amputation for arterial disease; previous or planned intervention on the abdominal aorta, limb arteries, or carotids.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "poor_mobility": {
                    "type": "boolean",
                    "description": "Neurological or musculoskeletal dysfunction severely affecting mobility/daily functioning.",
                    "definition": {
                        "concept": "Poor mobility (neurological/musculoskeletal dysfunction)",
                        "statement": "Severe impairment of mobility secondary to musculoskeletal or neurological dysfunction.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "previous_cardiac_surgery": {
                    "type": "boolean",
                    "description": "One or more previous major cardiac operations involving opening of the pericardium (redo surgery).",
                    "definition": {
                        "concept": "Previous cardiac surgery (redo)",
                        "statement": "One or more previous major cardiac operations involving opening of the pericardium.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "renal_function": {
                    "type": "string",
                    "enum": ["normal", "moderate", "severe", "on_dialysis"],
                    "description": "Renal impairment by creatinine clearance (Cockcroft-Gault), or dialysis.",
                    "definition": {
                        "concept": "Renal impairment (creatinine clearance bands)",
                        "statement": "Banded by Cockcroft-Gault creatinine clearance: normal = CrCl > 85 mL/min (reference); moderate = CrCl 51-85; severe = CrCl <= 50; on_dialysis = on dialysis regardless of clearance.",
                        "caveats": "EuroSCORE II uses creatinine CLEARANCE bands, not raw serum creatinine. on_dialysis takes precedence over the measured clearance.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "active_endocarditis": {
                    "type": "boolean",
                    "description": "Still on antibiotic treatment for endocarditis at the time of surgery.",
                    "definition": {
                        "concept": "Active endocarditis (AE)",
                        "statement": "Patient still under antibiotic treatment for endocarditis at the time of surgery.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "critical_preoperative_state": {
                    "type": "boolean",
                    "description": "Any of: VT/VF or aborted sudden death; preoperative cardiac massage; ventilation before theatre; inotropes; IABP/VAD; or preoperative acute renal failure (anuria/oliguria <10 mL/h).",
                    "definition": {
                        "concept": "Critical preoperative state",
                        "statement": "Any one or more of: ventricular tachycardia/fibrillation or aborted sudden death; preoperative cardiac massage; ventilation before arrival in the anaesthetic room; preoperative inotropic support; intra-aortic balloon pump or ventricular assist device; or preoperative acute renal failure (anuria or oliguria < 10 mL/h).",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "nyha": {
                    "type": "string",
                    "enum": ["i", "ii", "iii", "iv"],
                    "description": "New York Heart Association functional class (I is the reference).",
                    "definition": {
                        "concept": "NYHA functional class",
                        "statement": "I = no limitation (reference); II = slight limitation on ordinary activity; III = marked limitation on less-than-ordinary activity; IV = symptoms at rest.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "ccs4_angina": {
                    "type": "boolean",
                    "description": "CCS class 4 angina (angina at rest or on any activity).",
                    "definition": {
                        "concept": "CCS class 4 angina",
                        "statement": "Canadian Cardiovascular Society class 4 angina: inability to carry out any activity without angina, or angina at rest.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "lv_function": {
                    "type": "string",
                    "enum": ["good", "moderate", "poor", "very_poor"],
                    "description": "Left-ventricular function by ejection fraction band.",
                    "definition": {
                        "concept": "LV function (ejection-fraction bands)",
                        "statement": "good = EF > 50% (reference); moderate = EF 31-50%; poor = EF 21-30%; very_poor = EF <= 20%.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "recent_mi": {
                    "type": "boolean",
                    "description": "Myocardial infarction within 90 days before the operation.",
                    "definition": {
                        "concept": "Recent myocardial infarction",
                        "statement": "Myocardial infarction within 90 days before the operation.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "pulmonary_hypertension": {
                    "type": "string",
                    "enum": ["none", "moderate", "severe"],
                    "description": "Pulmonary-artery systolic pressure band.",
                    "definition": {
                        "concept": "Pulmonary hypertension (PA systolic pressure)",
                        "statement": "none = < 31 mmHg (reference); moderate = 31-55 mmHg; severe = > 55 mmHg.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "urgency": {
                    "type": "string",
                    "enum": ["elective", "urgent", "emergency", "salvage"],
                    "description": "Urgency of the operation.",
                    "definition": {
                        "concept": "Urgency",
                        "statement": "elective = routine admission for operation (reference); urgent = not electively admitted but requires surgery on the current admission; emergency = operation before the start of the next working day after the decision to operate; salvage = requiring CPR en route to theatre or before induction.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "weight_of_intervention": {
                    "type": "string",
                    "enum": ["isolated_cabg", "single_non_cabg", "two_procedures", "three_or_more"],
                    "description": "Weight (complexity) of the intervention.",
                    "definition": {
                        "concept": "Weight of intervention",
                        "statement": "isolated_cabg = isolated coronary artery bypass grafting (lowest-risk reference); single_non_cabg = one major cardiac procedure other than isolated CABG; two_procedures = two major cardiac procedures; three_or_more = three or more major cardiac procedures.",
                        "caveats": "A 'major procedure' is any major cardiac procedure on the heart or proximate great vessels. CABG combined with one other major procedure counts as two procedures.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                },
                "thoracic_aorta_surgery": {
                    "type": "boolean",
                    "description": "Surgery on the thoracic aorta.",
                    "definition": {
                        "concept": "Surgery on thoracic aorta",
                        "statement": "Operation involving the thoracic aorta.",
                        "source": { "citation": "Nashef SAM et al. Eur J Cardiothorac Surg. 2012;41(4):734-744.", "url": "https://doi.org/10.1093/ejcts/ezs043" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: EuroScore2Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lowest-risk baseline: 60-year-old male, isolated CABG, every other
    /// factor at its reference value.
    fn baseline() -> EuroScore2Input {
        EuroScore2Input {
            age: 60,
            female: false,
            insulin_dependent_diabetes: false,
            chronic_pulmonary_dysfunction: false,
            extracardiac_arteriopathy: false,
            poor_mobility: false,
            previous_cardiac_surgery: false,
            renal_function: RenalFunction::Normal,
            active_endocarditis: false,
            critical_preoperative_state: false,
            nyha: Nyha::I,
            ccs4_angina: false,
            lv_function: LvFunction::Good,
            recent_mi: false,
            pulmonary_hypertension: PulmonaryHypertension::None,
            urgency: Urgency::Elective,
            weight_of_intervention: WeightOfIntervention::IsolatedCabg,
            thoracic_aorta_surgery: false,
        }
    }

    #[test]
    fn age_encoding_is_one_until_sixty_then_per_year() {
        assert_eq!(age_step(40), 1);
        assert_eq!(age_step(60), 1);
        assert_eq!(age_step(61), 2);
        assert_eq!(age_step(62), 3);
        assert_eq!(age_step(80), 21);
    }

    /// VALIDATION CASE 1 (anchored to euroscore.org / Evidencio).
    ///
    /// A healthy 60-year-old man having isolated CABG with no other risk factors
    /// is the universally-cited EuroSCORE II baseline of ~0.50%. Linear predictor
    /// y = -5.324537 + 0.0285181*1 = -5.296019; exp(y)/(1+exp(y)) = 0.4987%.
    #[test]
    fn validation_case_1_baseline_is_half_a_percent() {
        let o = compute(&baseline()).unwrap();
        assert!(
            (o.linear_predictor - (-5.296019)).abs() < 1e-5,
            "y = {}",
            o.linear_predictor
        );
        assert!(
            (o.mortality_percent - 0.4987).abs() < 0.005,
            "mortality = {}%",
            o.mortality_percent
        );
    }

    /// VALIDATION CASE 2 (computed from the published model; coefficients
    /// cross-checked against the Nashef 2012 Table 6 and the Evidencio
    /// reproduction, which agree to all published digits).
    ///
    /// 75-year-old woman, insulin-dependent diabetes, NYHA III, moderate LV
    /// function, urgent, single non-CABG procedure (e.g. AVR), creatinine
    /// clearance 51-85 (moderate renal), recent MI.
    ///
    /// age step = 75 - 59 = 16; y = -5.324537 + 0.0285181*16 + 0.2196434 (female)
    /// + 0.3542749 (IDDM) + 0.2958358 (NYHA III) + 0.3150652 (LV mod)
    /// + 0.3174673 (urgent) + 0.0062118 (single non-CABG) + 0.303553 (CC 51-85)
    /// + 0.1528943 (recent MI) = -2.903302; mortality = 5.1991%.
    #[test]
    fn validation_case_2_intermediate_risk() {
        let mut i = baseline();
        i.age = 75;
        i.female = true;
        i.insulin_dependent_diabetes = true;
        i.nyha = Nyha::III;
        i.lv_function = LvFunction::Moderate;
        i.urgency = Urgency::Urgent;
        i.weight_of_intervention = WeightOfIntervention::SingleNonCabg;
        i.renal_function = RenalFunction::Moderate;
        i.recent_mi = true;

        let o = compute(&i).unwrap();
        assert_eq!(o.age_step, 16);
        assert!(
            (o.linear_predictor - (-2.903302)).abs() < 1e-5,
            "y = {}",
            o.linear_predictor
        );
        assert!(
            (o.mortality_percent - 5.1991).abs() < 0.005,
            "mortality = {}%",
            o.mortality_percent
        );
    }

    #[test]
    fn each_band_uses_its_published_coefficient() {
        assert_eq!(RenalFunction::Normal.beta(), 0.0);
        assert_eq!(RenalFunction::Moderate.beta(), 0.303553);
        assert_eq!(RenalFunction::Severe.beta(), 0.8592256);
        assert_eq!(RenalFunction::OnDialysis.beta(), 0.6421508);

        assert_eq!(Nyha::I.beta(), 0.0);
        assert_eq!(Nyha::II.beta(), 0.1070545);
        assert_eq!(Nyha::III.beta(), 0.2958358);
        assert_eq!(Nyha::IV.beta(), 0.5597929);

        assert_eq!(LvFunction::Good.beta(), 0.0);
        assert_eq!(LvFunction::Moderate.beta(), 0.3150652);
        assert_eq!(LvFunction::Poor.beta(), 0.8084096);
        assert_eq!(LvFunction::VeryPoor.beta(), 0.9346919);

        assert_eq!(PulmonaryHypertension::None.beta(), 0.0);
        assert_eq!(PulmonaryHypertension::Moderate.beta(), 0.1788899);
        assert_eq!(PulmonaryHypertension::Severe.beta(), 0.3491475);

        assert_eq!(Urgency::Elective.beta(), 0.0);
        assert_eq!(Urgency::Urgent.beta(), 0.3174673);
        assert_eq!(Urgency::Emergency.beta(), 0.7039121);
        assert_eq!(Urgency::Salvage.beta(), 1.362947);

        assert_eq!(WeightOfIntervention::IsolatedCabg.beta(), 0.0);
        assert_eq!(WeightOfIntervention::SingleNonCabg.beta(), 0.0062118);
        assert_eq!(WeightOfIntervention::TwoProcedures.beta(), 0.5521478);
        assert_eq!(WeightOfIntervention::ThreeOrMore.beta(), 0.9724533);
    }

    #[test]
    fn mortality_rises_monotonically_with_risk() {
        let low = compute(&baseline()).unwrap();

        let mut high = baseline();
        high.age = 85;
        high.previous_cardiac_surgery = true;
        high.critical_preoperative_state = true;
        high.lv_function = LvFunction::VeryPoor;
        high.urgency = Urgency::Salvage;
        high.weight_of_intervention = WeightOfIntervention::ThreeOrMore;
        high.renal_function = RenalFunction::OnDialysis;
        let high = compute(&high).unwrap();

        assert!(high.mortality_percent > low.mortality_percent);
        assert!(high.mortality_percent < 100.0);
        assert!(high.mortality_percent > 50.0, "got {}", high.mortality_percent);
    }

    #[test]
    fn mortality_is_a_valid_probability() {
        let o = compute(&baseline()).unwrap();
        assert!(o.mortality_percent > 0.0 && o.mortality_percent < 100.0);
    }

    #[test]
    fn rejects_implausible_age() {
        let mut i = baseline();
        i.age = 10;
        assert!(compute(&i).is_err());
        i.age = 115;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 75,
            "female": true,
            "insulin_dependent_diabetes": true,
            "chronic_pulmonary_dysfunction": false,
            "extracardiac_arteriopathy": false,
            "poor_mobility": false,
            "previous_cardiac_surgery": false,
            "renal_function": "moderate",
            "active_endocarditis": false,
            "critical_preoperative_state": false,
            "nyha": "iii",
            "ccs4_angina": false,
            "lv_function": "moderate",
            "recent_mi": true,
            "pulmonary_hypertension": "none",
            "urgency": "urgent",
            "weight_of_intervention": "single_non_cabg",
            "thoracic_aorta_surgery": false
        });

        let mut typed = baseline();
        typed.age = 75;
        typed.female = true;
        typed.insulin_dependent_diabetes = true;
        typed.nyha = Nyha::III;
        typed.lv_function = LvFunction::Moderate;
        typed.urgency = Urgency::Urgent;
        typed.weight_of_intervention = WeightOfIntervention::SingleNonCabg;
        typed.renal_function = RenalFunction::Moderate;
        typed.recent_mi = true;

        let dynamic = EuroScore2.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(5.2));
    }

    #[test]
    fn schema_describes_all_banded_variables() {
        let schema = EuroScore2.input_schema();
        for var in [
            "renal_function",
            "nyha",
            "lv_function",
            "pulmonary_hypertension",
            "urgency",
            "weight_of_intervention",
        ] {
            assert!(
                schema["properties"][var]["definition"].is_object(),
                "{var} must carry a definition"
            );
            assert!(schema["properties"][var]["enum"].is_array(), "{var} must enumerate its bands");
        }
    }
}
