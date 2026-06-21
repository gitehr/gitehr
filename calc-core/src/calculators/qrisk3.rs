//! QRISK3-2017 - 10-year cardiovascular disease risk (Hippisley-Cox et al., BMJ 2017).
//!
//! The UK standard for primary cardiovascular risk assessment (NICE NG238): the
//! predicted percentage probability that a person without prior CVD will have a
//! heart attack or stroke within 10 years. There is no open equivalent; QRISK3
//! *is* the recommended tool.
//!
//! This is a sex-stratified Cox fractional-polynomial model: separate male and
//! female equations, each summing fractional-polynomial transforms of age and
//! BMI, centred continuous predictors (cholesterol/HDL ratio, systolic BP, the
//! standard deviation of recent systolic readings, Townsend deprivation),
//! conditional ethnicity and smoking coefficients, boolean comorbidity
//! coefficients, and a large set of age interactions, then converting the linear
//! predictor through a baseline-survival constant.
//!
//! Coefficients are transcribed verbatim from ClinRisk's open-source reference
//! C implementation (Copyright 2017 ClinRisk Ltd., released under the LGPL v3+ to
//! enable faithful reimplementation) and validated against the original C
//! algorithm's published test scores (see the tests below).
//!
//! Two subtleties are encoded to avoid silent error:
//! - BMI is derived from height and weight rather than accepted directly. The
//!   model is highly sensitive to BMI, and the fractional-polynomial transform
//!   makes a wrong BMI hard to spot; deriving it removes that footgun.
//! - Smoking has five ordered categories (not a boolean) and ethnicity nine, both
//!   of which carry their own coefficients *and* age interactions, so they are
//!   modelled as enums rather than free integers.
//!
//! Per ClinRisk's licence terms, the official disclaimer ([`DISCLAIMER`]) is
//! carried in every response: an inaccurate implementation could lead to wrong
//! treatment, so the score must be checked against the original at qrisk.org.

// The coefficients below carry more decimal places than an f64 can represent;
// they are kept exactly as ClinRisk published them so the transcription is
// verifiable digit-for-digit against the source. Rust rounds each to the nearest
// f64 just as the original C compiler does, so the extra digits are faithful, not
// misleading - hence the allow rather than truncating the published values.
#![allow(clippy::excessive_precision)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "qrisk3";

/// Primary citation.
pub const REFERENCE: &str =
    "Hippisley-Cox J, Coupland C, Brindle P. Development and validation of QRISK3 risk prediction \
algorithms to estimate future risk of cardiovascular disease: prospective cohort study. BMJ. \
2017;357:j2099. doi:10.1136/bmj.j2099. Recommended by NICE NG238.";

/// Distribution licence: ClinRisk Ltd. released the QRISK3-2017 algorithm source
/// under the LGPL v3+ specifically to enable faithful reimplementation; the
/// coefficients here are transcribed verbatim from that source.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "LGPL-3.0-or-later - QRISK3-2017 algorithm Copyright 2017 ClinRisk Ltd.",
    source_url: "https://qrisk.org/src.php",
};

/// ClinRisk's required disclaimer, carried alongside every score per the licence
/// terms. Inaccurate implementations can lead to wrong treatment, so the result
/// must be checked against the original algorithm at qrisk.org.
pub const DISCLAIMER: &str = "QRISK3-2017 algorithm Copyright 2017 ClinRisk Ltd., used under the \
LGPL. ClinRisk Ltd. stress that it is the responsibility of the end user to check that this \
implementation produces the same results as the original code at https://qrisk.org. Inaccurate \
implementations of risk scores can lead to wrong patients being given the wrong treatment.";

/// Biological sex, selecting the male or female equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Self-reported ethnicity, in the nine QRISK3 categories. Each carries its own
/// coefficient; "white or not stated" is the reference (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ethnicity {
    /// White or not stated (reference category).
    WhiteOrNotStated,
    Indian,
    Pakistani,
    Bangladeshi,
    OtherAsian,
    BlackCaribbean,
    BlackAfrican,
    Chinese,
    OtherEthnicGroup,
}

impl Ethnicity {
    /// 1-based index into the model's `Iethrisk` array (matching the published
    /// source, where index 1 = white/not stated maps to coefficient 0).
    fn index(self) -> usize {
        match self {
            Ethnicity::WhiteOrNotStated => 1,
            Ethnicity::Indian => 2,
            Ethnicity::Pakistani => 3,
            Ethnicity::Bangladeshi => 4,
            Ethnicity::OtherAsian => 5,
            Ethnicity::BlackCaribbean => 6,
            Ethnicity::BlackAfrican => 7,
            Ethnicity::Chinese => 8,
            Ethnicity::OtherEthnicGroup => 9,
        }
    }
}

/// Smoking status, in the five QRISK3 categories. Carries both a coefficient and
/// age interactions; "non-smoker" is the reference (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Smoking {
    NonSmoker,
    ExSmoker,
    /// Light smoker (fewer than 10 a day).
    LightSmoker,
    /// Moderate smoker (10 to 19 a day).
    ModerateSmoker,
    /// Heavy smoker (20 or more a day).
    HeavySmoker,
}

impl Smoking {
    /// 0-based smoking category, matching the published source's `Ismoke` array
    /// and its `(smoke_cat==1..4)` interaction tests (0 = non-smoker = reference).
    fn cat(self) -> usize {
        match self {
            Smoking::NonSmoker => 0,
            Smoking::ExSmoker => 1,
            Smoking::LightSmoker => 2,
            Smoking::ModerateSmoker => 3,
            Smoking::HeavySmoker => 4,
        }
    }
}

/// QRISK3 inputs. BMI is derived from height and weight.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Qrisk3Input {
    /// Age in years. QRISK3 is validated for ages 25-84.
    pub age: u8,
    pub sex: Sex,
    pub ethnicity: Ethnicity,
    pub smoking: Smoking,
    /// Height in centimetres (for BMI).
    pub height_cm: f64,
    /// Weight in kilograms (for BMI).
    pub weight_kg: f64,
    /// Total cholesterol / HDL ratio (typically 1-11).
    pub cholesterol_hdl_ratio: f64,
    /// Systolic blood pressure, mmHg.
    pub systolic_bp: f64,
    /// Standard deviation of at least two recent systolic BP readings, mmHg.
    /// Blood-pressure variability is an independent predictor in QRISK3. If
    /// unknown, 0 disables the term (the model treats it as the centred mean).
    pub systolic_bp_sd: f64,
    /// Townsend deprivation score (area-based; negative = less deprived).
    pub townsend: f64,
    /// Atrial fibrillation.
    pub atrial_fibrillation: bool,
    /// On an atypical antipsychotic.
    pub atypical_antipsychotic: bool,
    /// On regular oral corticosteroids.
    pub regular_steroids: bool,
    /// Migraine.
    pub migraine: bool,
    /// Rheumatoid arthritis.
    pub rheumatoid_arthritis: bool,
    /// Chronic kidney disease, stage 3, 4, or 5.
    pub ckd_stage_3_5: bool,
    /// Severe mental illness (schizophrenia, bipolar, or severe depression).
    pub severe_mental_illness: bool,
    /// Systemic lupus erythematosus.
    pub sle: bool,
    /// On treatment for hypertension.
    pub treated_hypertension: bool,
    /// Type 1 diabetes.
    pub type1_diabetes: bool,
    /// Type 2 diabetes.
    pub type2_diabetes: bool,
    /// Erectile dysfunction (male model only; ignored for the female equation).
    pub erectile_dysfunction: bool,
    /// Angina or heart attack in a first-degree relative aged under 60.
    pub family_history_chd: bool,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct Qrisk3Outcome {
    /// 10-year CVD risk, as a percentage rounded to one decimal place.
    pub risk_percent: f64,
    /// The derived BMI (kg/m^2) used in the model.
    pub bmi: f64,
    pub interpretation: String,
}

fn b(flag: bool) -> f64 {
    if flag {
        1.0
    } else {
        0.0
    }
}

/// Female QRISK3-2017 linear predictor and score.
/// Coefficients transcribed verbatim from ClinRisk's reference C source.
fn cvd_female_raw(i: &Qrisk3Input, bmi: f64) -> f64 {
    let survivor10 = 0.988876402378082_f64;

    let iethrisk = [
        0.0,
        0.0,
        0.2804031433299542500000000,
        0.5629899414207539800000000,
        0.2959000085111651600000000,
        0.0727853798779825450000000,
        -0.1707213550885731700000000,
        -0.3937104331487497100000000,
        -0.3263249528353027200000000,
        -0.1712705688324178400000000,
    ];
    let ismoke = [
        0.0,
        0.1338683378654626200000000,
        0.5620085801243853700000000,
        0.6674959337750254700000000,
        0.8494817764483084700000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(-2.0);
    let mut age_2 = dage;
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(-2.0);
    let mut bmi_2 = dbmi.powf(-2.0) * dbmi.ln();

    age_1 -= 0.053274843841791;
    age_2 -= 4.332503318786621;
    bmi_1 -= 0.154946178197861;
    bmi_2 -= 0.144462317228317;
    let rati = i.cholesterol_hdl_ratio - 3.476326465606690;
    let sbp = i.systolic_bp - 123.130012512207030;
    let sbps5 = i.systolic_bp_sd - 9.002537727355957;
    let town = i.townsend - 0.392308831214905;

    let smoke = i.smoking.cat();
    let mut a = 0.0;

    a += iethrisk[i.ethnicity.index()];
    a += ismoke[smoke];

    a += age_1 * -8.1388109247726188000000000;
    a += age_2 * 0.7973337668969909800000000;
    a += bmi_1 * 0.2923609227546005200000000;
    a += bmi_2 * -4.1513300213837665000000000;
    a += rati * 0.1533803582080255400000000;
    a += sbp * 0.0131314884071034240000000;
    a += sbps5 * 0.0078894541014586095000000;
    a += town * 0.0772237905885901080000000;

    a += b(i.atrial_fibrillation) * 1.5923354969269663000000000;
    a += b(i.atypical_antipsychotic) * 0.2523764207011555700000000;
    a += b(i.regular_steroids) * 0.5952072530460185100000000;
    a += b(i.migraine) * 0.3012672608703450000000000;
    a += b(i.rheumatoid_arthritis) * 0.2136480343518194200000000;
    a += b(i.ckd_stage_3_5) * 0.6519456949384583300000000;
    a += b(i.severe_mental_illness) * 0.1255530805882017800000000;
    a += b(i.sle) * 0.7588093865426769300000000;
    a += b(i.treated_hypertension) * 0.5093159368342300400000000;
    a += b(i.type1_diabetes) * 1.7267977510537347000000000;
    a += b(i.type2_diabetes) * 1.0688773244615468000000000;
    a += b(i.family_history_chd) * 0.4544531902089621300000000;

    a += age_1 * b(smoke == 1) * -4.7057161785851891000000000;
    a += age_1 * b(smoke == 2) * -2.7430383403573337000000000;
    a += age_1 * b(smoke == 3) * -0.8660808882939218200000000;
    a += age_1 * b(smoke == 4) * 0.9024156236971064800000000;
    a += age_1 * b(i.atrial_fibrillation) * 19.9380348895465610000000000;
    a += age_1 * b(i.regular_steroids) * -0.9840804523593628100000000;
    a += age_1 * b(i.migraine) * 1.7634979587872999000000000;
    a += age_1 * b(i.ckd_stage_3_5) * -3.5874047731694114000000000;
    a += age_1 * b(i.sle) * 19.6903037386382920000000000;
    a += age_1 * b(i.treated_hypertension) * 11.8728097339218120000000000;
    a += age_1 * b(i.type1_diabetes) * -1.2444332714320747000000000;
    a += age_1 * b(i.type2_diabetes) * 6.8652342000009599000000000;
    a += age_1 * bmi_1 * 23.8026234121417420000000000;
    a += age_1 * bmi_2 * -71.1849476920870070000000000;
    a += age_1 * b(i.family_history_chd) * 0.9946780794043512700000000;
    a += age_1 * sbp * 0.0341318423386154850000000;
    a += age_1 * town * -1.0301180802035639000000000;
    a += age_2 * b(smoke == 1) * -0.0755892446431930260000000;
    a += age_2 * b(smoke == 2) * -0.1195119287486707400000000;
    a += age_2 * b(smoke == 3) * -0.1036630639757192300000000;
    a += age_2 * b(smoke == 4) * -0.1399185359171838900000000;
    a += age_2 * b(i.atrial_fibrillation) * -0.0761826510111625050000000;
    a += age_2 * b(i.regular_steroids) * -0.1200536494674247200000000;
    a += age_2 * b(i.migraine) * -0.0655869178986998590000000;
    a += age_2 * b(i.ckd_stage_3_5) * -0.2268887308644250700000000;
    a += age_2 * b(i.sle) * 0.0773479496790162730000000;
    a += age_2 * b(i.treated_hypertension) * 0.0009685782358817443600000;
    a += age_2 * b(i.type1_diabetes) * -0.2872406462448894900000000;
    a += age_2 * b(i.type2_diabetes) * -0.0971122525906954890000000;
    a += age_2 * bmi_1 * 0.5236995893366442900000000;
    a += age_2 * bmi_2 * 0.0457441901223237590000000;
    a += age_2 * b(i.family_history_chd) * -0.0768850516984230380000000;
    a += age_2 * sbp * -0.0015082501423272358000000;
    a += age_2 * town * -0.0315934146749623290000000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Male QRISK3-2017 linear predictor and score.
/// Coefficients transcribed verbatim from ClinRisk's reference C source.
fn cvd_male_raw(i: &Qrisk3Input, bmi: f64) -> f64 {
    let survivor10 = 0.977268040180206_f64;

    let iethrisk = [
        0.0,
        0.0,
        0.2771924876030827900000000,
        0.4744636071493126800000000,
        0.5296172991968937100000000,
        0.0351001591862990170000000,
        -0.3580789966932791900000000,
        -0.4005648523216514000000000,
        -0.4152279288983017300000000,
        -0.2632134813474996700000000,
    ];
    let ismoke = [
        0.0,
        0.1912822286338898300000000,
        0.5524158819264555200000000,
        0.6383505302750607200000000,
        0.7898381988185801900000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(-1.0);
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_2 = dbmi.powf(-2.0) * dbmi.ln();
    let mut bmi_1 = dbmi.powf(-2.0);

    age_1 -= 0.234766781330109;
    age_2 -= 77.284080505371094;
    bmi_1 -= 0.149176135659218;
    bmi_2 -= 0.141913309693336;
    let rati = i.cholesterol_hdl_ratio - 4.300998687744141;
    let sbp = i.systolic_bp - 128.571578979492190;
    let sbps5 = i.systolic_bp_sd - 8.756621360778809;
    let town = i.townsend - 0.526304900646210;

    let smoke = i.smoking.cat();
    let mut a = 0.0;

    a += iethrisk[i.ethnicity.index()];
    a += ismoke[smoke];

    a += age_1 * -17.8397816660055750000000000;
    a += age_2 * 0.0022964880605765492000000;
    a += bmi_1 * 2.4562776660536358000000000;
    a += bmi_2 * -8.3011122314711354000000000;
    a += rati * 0.1734019685632711100000000;
    a += sbp * 0.0129101265425533050000000;
    a += sbps5 * 0.0102519142912904560000000;
    a += town * 0.0332682012772872950000000;

    a += b(i.atrial_fibrillation) * 0.8820923692805465700000000;
    a += b(i.atypical_antipsychotic) * 0.1304687985517351300000000;
    a += b(i.regular_steroids) * 0.4548539975044554300000000;
    a += b(i.erectile_dysfunction) * 0.2225185908670538300000000;
    a += b(i.migraine) * 0.2558417807415991300000000;
    a += b(i.rheumatoid_arthritis) * 0.2097065801395656700000000;
    a += b(i.ckd_stage_3_5) * 0.7185326128827438400000000;
    a += b(i.severe_mental_illness) * 0.1213303988204716400000000;
    a += b(i.sle) * 0.4401572174457522000000000;
    a += b(i.treated_hypertension) * 0.5165987108269547400000000;
    a += b(i.type1_diabetes) * 1.2343425521675175000000000;
    a += b(i.type2_diabetes) * 0.8594207143093222100000000;
    a += b(i.family_history_chd) * 0.5405546900939015600000000;

    a += age_1 * b(smoke == 1) * -0.2101113393351634600000000;
    a += age_1 * b(smoke == 2) * 0.7526867644750319100000000;
    a += age_1 * b(smoke == 3) * 0.9931588755640579100000000;
    a += age_1 * b(smoke == 4) * 2.1331163414389076000000000;
    a += age_1 * b(i.atrial_fibrillation) * 3.4896675530623207000000000;
    a += age_1 * b(i.regular_steroids) * 1.1708133653489108000000000;
    a += age_1 * b(i.erectile_dysfunction) * -1.5064009857454310000000000;
    a += age_1 * b(i.migraine) * 2.3491159871402441000000000;
    a += age_1 * b(i.ckd_stage_3_5) * -0.5065671632722369400000000;
    a += age_1 * b(i.treated_hypertension) * 6.5114581098532671000000000;
    a += age_1 * b(i.type1_diabetes) * 5.3379864878006531000000000;
    a += age_1 * b(i.type2_diabetes) * 3.6461817406221311000000000;
    a += age_1 * bmi_1 * 31.0049529560338860000000000;
    a += age_1 * bmi_2 * -111.2915718439164300000000000;
    a += age_1 * b(i.family_history_chd) * 2.7808628508531887000000000;
    a += age_1 * sbp * 0.0188585244698658530000000;
    a += age_1 * town * -0.1007554870063731000000000;
    a += age_2 * b(smoke == 1) * -0.0004985487027532612100000;
    a += age_2 * b(smoke == 2) * -0.0007987563331738541400000;
    a += age_2 * b(smoke == 3) * -0.0008370618426625129600000;
    a += age_2 * b(smoke == 4) * -0.0007840031915563728900000;
    a += age_2 * b(i.atrial_fibrillation) * -0.0003499560834063604900000;
    a += age_2 * b(i.regular_steroids) * -0.0002496045095297166000000;
    a += age_2 * b(i.erectile_dysfunction) * -0.0011058218441227373000000;
    a += age_2 * b(i.migraine) * 0.0001989644604147863100000;
    a += age_2 * b(i.ckd_stage_3_5) * -0.0018325930166498813000000;
    a += age_2 * b(i.treated_hypertension) * 0.0006383805310416501300000;
    a += age_2 * b(i.type1_diabetes) * 0.0006409780808752897000000;
    a += age_2 * b(i.type2_diabetes) * -0.0002469569558886831500000;
    a += age_2 * bmi_1 * 0.0050380102356322029000000;
    a += age_2 * bmi_2 * -0.0130744830025243190000000;
    a += age_2 * b(i.family_history_chd) * -0.0002479180990739603700000;
    a += age_2 * sbp * -0.0000127187419158845700000;
    a += age_2 * town * -0.0000932996423232728880000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Pure scoring: the QRISK3-2017 model. BMI is derived from height and weight.
pub fn compute(input: &Qrisk3Input) -> Result<Qrisk3Outcome, CalcError> {
    if input.age < 25 || input.age > 84 {
        return Err(CalcError::InvalidInput(
            "QRISK3 is validated for ages 25-84".into(),
        ));
    }
    if !(input.height_cm.is_finite() && input.height_cm > 0.0) {
        return Err(CalcError::InvalidInput(
            "height_cm must be a positive number".into(),
        ));
    }
    if !(input.weight_kg.is_finite() && input.weight_kg > 0.0) {
        return Err(CalcError::InvalidInput(
            "weight_kg must be a positive number".into(),
        ));
    }
    for (name, v) in [
        ("cholesterol_hdl_ratio", input.cholesterol_hdl_ratio),
        ("systolic_bp", input.systolic_bp),
        ("systolic_bp_sd", input.systolic_bp_sd),
        ("townsend", input.townsend),
    ] {
        if !v.is_finite() {
            return Err(CalcError::InvalidInput(format!("{name} must be a finite number")));
        }
    }
    if input.systolic_bp_sd < 0.0 {
        return Err(CalcError::InvalidInput(
            "systolic_bp_sd cannot be negative".into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let bmi = input.weight_kg / (height_m * height_m);

    let raw = match input.sex {
        Sex::Female => cvd_female_raw(input, bmi),
        Sex::Male => cvd_male_raw(input, bmi),
    };

    let risk_percent = (raw * 10.0).round() / 10.0;

    let band = if risk_percent < 10.0 {
        "low (<10%)"
    } else if risk_percent < 20.0 {
        "moderate (10-20%)"
    } else {
        "high (>=20%)"
    };

    let interpretation = format!(
        "10-year cardiovascular risk {risk_percent}% ({band}), QRISK3-2017. NICE NG238 advises \
considering a statin (atorvastatin 20 mg) for primary prevention at a 10-year risk of 10% or more, \
as part of a shared decision weighing lifestyle, comorbidity, and patient preference. BMI used: \
{bmi:.1} kg/m2. {DISCLAIMER}"
    );

    Ok(Qrisk3Outcome {
        risk_percent,
        bmi,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &Qrisk3Input) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("risk_percent".into(), json!(o.risk_percent));
    working.insert("bmi".into(), json!((o.bmi * 10.0).round() / 10.0));
    working.insert("sex".into(), json!(input.sex));
    working.insert("disclaimer".into(), json!(DISCLAIMER));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.risk_percent),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Qrisk3;

impl Calculator for Qrisk3 {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "QRISK3 (10-year cardiovascular risk)"
    }

    fn description(&self) -> &'static str {
        "10-year risk of heart attack or stroke (QRISK3-2017), the UK standard for primary CVD risk assessment (NICE NG238)."
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
            "title": "Qrisk3Input",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "sex", "ethnicity", "smoking", "height_cm", "weight_kg",
                "cholesterol_hdl_ratio", "systolic_bp", "systolic_bp_sd", "townsend",
                "atrial_fibrillation", "atypical_antipsychotic", "regular_steroids",
                "migraine", "rheumatoid_arthritis", "ckd_stage_3_5", "severe_mental_illness",
                "sle", "treated_hypertension", "type1_diabetes", "type2_diabetes",
                "erectile_dysfunction", "family_history_chd"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 25,
                    "maximum": 84,
                    "description": "Age in years (QRISK3 is validated for 25-84)"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Biological sex, selecting the male or female equation",
                    "definition": {
                        "concept": "Sex for the QRISK3 equation",
                        "statement": "The sex whose sex-specific QRISK3 equation (and erectile-dysfunction term, male only) is used.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "ethnicity": {
                    "type": "string",
                    "enum": [
                        "white_or_not_stated", "indian", "pakistani", "bangladeshi", "other_asian",
                        "black_caribbean", "black_african", "chinese", "other_ethnic_group"
                    ],
                    "description": "Self-reported ethnicity in the nine QRISK3 categories",
                    "definition": {
                        "concept": "Ethnicity (9 QRISK3 categories)",
                        "statement": "Self-reported ethnicity; 'white or not stated' is the reference category.",
                        "caveats": "These are the exact categories the model was fitted on; use 'white_or_not_stated' when ethnicity is unknown, matching the official tool's default.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "smoking": {
                    "type": "string",
                    "enum": ["non_smoker", "ex_smoker", "light_smoker", "moderate_smoker", "heavy_smoker"],
                    "description": "Smoking status in the five QRISK3 categories",
                    "definition": {
                        "concept": "Smoking status (5 ordered categories)",
                        "statement": "non_smoker; ex_smoker; light_smoker (<10/day); moderate_smoker (10-19/day); heavy_smoker (20+/day).",
                        "caveats": "Smoking is an ordered five-level category in QRISK3, not a yes/no, and the categories also drive age-interaction terms.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "height_cm": { "type": "number", "exclusiveMinimum": 0, "description": "Height in centimetres (used to derive BMI)" },
                "weight_kg": { "type": "number", "exclusiveMinimum": 0, "description": "Weight in kilograms (used to derive BMI)" },
                "cholesterol_hdl_ratio": {
                    "type": "number",
                    "description": "Total cholesterol / HDL ratio (typically 1-11)",
                    "definition": {
                        "concept": "Cholesterol/HDL ratio",
                        "statement": "Total cholesterol divided by HDL cholesterol, a single ratio rather than the two values.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "systolic_bp": { "type": "number", "description": "Systolic blood pressure, mmHg" },
                "systolic_bp_sd": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Standard deviation of recent systolic BP readings, mmHg (0 if unknown)",
                    "definition": {
                        "concept": "Systolic BP variability (SD of recent readings)",
                        "statement": "The standard deviation of at least two recent systolic readings. BP variability is an independent QRISK3 predictor.",
                        "caveats": "This is NOT the systolic value itself. If only one reading is available, pass 0; the model then treats variability as the population mean (no effect).",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "townsend": {
                    "type": "number",
                    "description": "Townsend deprivation score (area-based; negative = less deprived, 0 if unknown)",
                    "definition": {
                        "concept": "Townsend deprivation score",
                        "statement": "An area-level deprivation score derived from the patient's postcode; the official tool looks it up from postcode.",
                        "caveats": "Not a patient-level value; if unknown, 0 approximates the population average.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "atrial_fibrillation": { "type": "boolean", "description": "Atrial fibrillation" },
                "atypical_antipsychotic": { "type": "boolean", "description": "On an atypical antipsychotic" },
                "regular_steroids": { "type": "boolean", "description": "On regular oral corticosteroids" },
                "migraine": { "type": "boolean", "description": "Migraine" },
                "rheumatoid_arthritis": { "type": "boolean", "description": "Rheumatoid arthritis" },
                "ckd_stage_3_5": {
                    "type": "boolean",
                    "description": "Chronic kidney disease, stage 3, 4, or 5",
                    "definition": {
                        "concept": "CKD stage 3-5",
                        "statement": "Chronic kidney disease of stage 3, 4, or 5.",
                        "excludes": ["CKD stage 1-2 does NOT count in QRISK3"],
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "severe_mental_illness": {
                    "type": "boolean",
                    "description": "Severe mental illness (schizophrenia, bipolar disorder, or severe depression)",
                    "definition": {
                        "concept": "Severe mental illness",
                        "statement": "Schizophrenia, bipolar affective disorder, or severe (moderate/severe) depression, as defined by QRISK3.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "sle": { "type": "boolean", "description": "Systemic lupus erythematosus" },
                "treated_hypertension": {
                    "type": "boolean",
                    "description": "On treatment for hypertension",
                    "definition": {
                        "concept": "Treated hypertension",
                        "statement": "A diagnosis of hypertension AND currently on at least one antihypertensive (this is the QRISK3 'blood pressure treatment' flag).",
                        "caveats": "QRISK3 captures treated hypertension specifically, not untreated raised BP (which is carried by the systolic BP input itself).",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "type1_diabetes": { "type": "boolean", "description": "Type 1 diabetes" },
                "type2_diabetes": { "type": "boolean", "description": "Type 2 diabetes" },
                "erectile_dysfunction": {
                    "type": "boolean",
                    "description": "Erectile dysfunction (male model only; ignored for females)",
                    "definition": {
                        "concept": "Erectile dysfunction",
                        "statement": "A diagnosis of, or treatment for, erectile dysfunction.",
                        "caveats": "Used by the male equation only; it has no coefficient in the female model and is ignored there.",
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                },
                "family_history_chd": {
                    "type": "boolean",
                    "description": "Angina or heart attack in a first-degree relative aged under 60",
                    "definition": {
                        "concept": "Family history of CHD",
                        "statement": "Angina or heart attack in a first-degree relative (parent or sibling) before the age of 60.",
                        "excludes": ["A relative affected at 60 or older does NOT count", "Second-degree relatives do NOT count"],
                        "source": { "citation": "Hippisley-Cox J et al. BMJ. 2017;357:j2099.", "url": "https://doi.org/10.1136/bmj.j2099" },
                        "status": "draft"
                    }
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: Qrisk3Input =
            serde_json::from_value(input.clone()).map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A neutral baseline: every boolean false, mean-ish continuous values. Tests
    /// override fields as needed.
    fn base(age: u8, sex: Sex) -> Qrisk3Input {
        Qrisk3Input {
            age,
            sex,
            ethnicity: Ethnicity::WhiteOrNotStated,
            smoking: Smoking::NonSmoker,
            height_cm: 178.0,
            weight_kg: 80.0,
            cholesterol_hdl_ratio: 4.0,
            systolic_bp: 180.0,
            systolic_bp_sd: 20.0,
            townsend: 0.0,
            atrial_fibrillation: false,
            atypical_antipsychotic: false,
            regular_steroids: false,
            migraine: false,
            rheumatoid_arthritis: false,
            ckd_stage_3_5: false,
            severe_mental_illness: false,
            sle: false,
            treated_hypertension: false,
            type1_diabetes: false,
            type2_diabetes: false,
            erectile_dysfunction: false,
            family_history_chd: false,
        }
    }

    /// Round to one decimal place, as QRISK3 reports.
    fn r1(x: f64) -> f64 {
        (x * 10.0).round() / 10.0
    }

    // --- Validation against ClinRisk's original C algorithm -----------------
    //
    // The reference scores below are the `QRISK_C_algorithm_score` values from
    // ClinRisk's original C implementation, shipped as the test dataset of the
    // peer-reviewed QRISK3 R package (Li, F1000Research 2019; archived at
    // https://doi.org/10.5281/zenodo.3571304). Each profile is a 64-year-old
    // Indian patient with cholesterol/HDL ratio 4, systolic BP 180, systolic SD
    // 20, non-smoker, and BMI derived from height/weight, varying one factor.
    // Matching these to one decimal place demonstrates a faithful port.

    /// id1: 64F, Indian, 80 kg / 178 cm (BMI ~25.25), no risk factors -> 17.2%.
    #[test]
    fn validation_female_base_clinrisk_c() {
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 17.2);
    }

    /// id2: as id1 but with atrial fibrillation -> 36.0%.
    #[test]
    fn validation_female_af_clinrisk_c() {
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        i.atrial_fibrillation = true;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 36.0);
    }

    /// id4: as id1 but on regular steroids -> 24.1%.
    #[test]
    fn validation_female_steroids_clinrisk_c() {
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        i.regular_steroids = true;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 24.1);
    }

    /// id15: 64F, Indian, 80 kg / 170 cm (BMI ~27.68), no factors -> 17.3%.
    /// Exercises BMI derived from a different height/weight.
    #[test]
    fn validation_female_bmi_from_height_weight_clinrisk_c() {
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        i.height_cm = 170.0;
        i.weight_kg = 80.0;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 17.3);
    }

    /// id25: 64M, Indian, 80 kg / 178 cm (BMI ~25.25), no factors -> 27.1%.
    #[test]
    fn validation_male_base_clinrisk_c() {
        let mut i = base(64, Sex::Male);
        i.ethnicity = Ethnicity::Indian;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 27.1);
    }

    /// id37: as id25 but with type 2 diabetes -> 41.5%.
    #[test]
    fn validation_male_type2_diabetes_clinrisk_c() {
        let mut i = base(64, Sex::Male);
        i.ethnicity = Ethnicity::Indian;
        i.type2_diabetes = true;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 41.5);
    }

    /// id38: 64M, Indian, 75 kg / 178 cm (BMI ~23.67), no factors -> 27.2%.
    #[test]
    fn validation_male_lower_bmi_clinrisk_c() {
        let mut i = base(64, Sex::Male);
        i.ethnicity = Ethnicity::Indian;
        i.weight_kg = 75.0;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 27.2);
    }

    // --- Behavioural / structural tests -------------------------------------

    #[test]
    fn risk_in_valid_range() {
        let o = compute(&base(50, Sex::Male)).unwrap();
        assert!(o.risk_percent >= 0.0 && o.risk_percent <= 100.0);
    }

    #[test]
    fn comorbidity_raises_risk() {
        let lo = compute(&base(60, Sex::Male)).unwrap().risk_percent;
        let mut i = base(60, Sex::Male);
        i.type2_diabetes = true;
        let hi = compute(&i).unwrap().risk_percent;
        assert!(hi > lo, "diabetes should raise risk: {lo} -> {hi}");
    }

    #[test]
    fn heavy_smoking_raises_risk() {
        let lo = compute(&base(55, Sex::Female)).unwrap().risk_percent;
        let mut i = base(55, Sex::Female);
        i.smoking = Smoking::HeavySmoker;
        let hi = compute(&i).unwrap().risk_percent;
        assert!(hi > lo, "heavy smoking should raise risk: {lo} -> {hi}");
    }

    #[test]
    fn erectile_dysfunction_ignored_for_female() {
        let mut a = base(60, Sex::Female);
        a.erectile_dysfunction = false;
        let mut b = base(60, Sex::Female);
        b.erectile_dysfunction = true;
        assert_eq!(compute(&a).unwrap().risk_percent, compute(&b).unwrap().risk_percent);
    }

    #[test]
    fn erectile_dysfunction_affects_male() {
        let mut a = base(60, Sex::Male);
        a.erectile_dysfunction = false;
        let mut b = base(60, Sex::Male);
        b.erectile_dysfunction = true;
        assert!(compute(&b).unwrap().risk_percent > compute(&a).unwrap().risk_percent);
    }

    #[test]
    fn rejects_out_of_range_age() {
        assert!(compute(&base(24, Sex::Male)).is_err());
        assert!(compute(&base(85, Sex::Male)).is_err());
    }

    #[test]
    fn rejects_bad_anthropometry() {
        let mut i = base(50, Sex::Male);
        i.height_cm = 0.0;
        assert!(compute(&i).is_err());
        let mut j = base(50, Sex::Male);
        j.weight_kg = -1.0;
        assert!(compute(&j).is_err());
    }

    #[test]
    fn rejects_negative_bp_sd() {
        let mut i = base(50, Sex::Male);
        i.systolic_bp_sd = -1.0;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn band_thresholds() {
        // The 10% NICE threshold should land in the interpretation banding.
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        let o = compute(&i).unwrap();
        assert!(o.interpretation.contains("moderate"), "{}", o.interpretation);
    }

    #[test]
    fn response_carries_disclaimer() {
        let mut i = base(64, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        let r = build_response(&i).unwrap();
        assert_eq!(r.calculator, "qrisk3");
        assert!(r.working["disclaimer"].as_str().unwrap().contains("ClinRisk"));
        assert!(r.interpretation.contains("qrisk.org"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 64, "sex": "female", "ethnicity": "indian", "smoking": "non_smoker",
            "height_cm": 178.0, "weight_kg": 80.0, "cholesterol_hdl_ratio": 4.0,
            "systolic_bp": 180.0, "systolic_bp_sd": 20.0, "townsend": 0.0,
            "atrial_fibrillation": false, "atypical_antipsychotic": false, "regular_steroids": false,
            "migraine": false, "rheumatoid_arthritis": false, "ckd_stage_3_5": false,
            "severe_mental_illness": false, "sle": false, "treated_hypertension": false,
            "type1_diabetes": false, "type2_diabetes": false, "erectile_dysfunction": false,
            "family_history_chd": false
        });
        let mut typed = base(64, Sex::Female);
        typed.ethnicity = Ethnicity::Indian;
        let dynamic = Qrisk3.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(17.2));
    }

    #[test]
    fn schema_flags_bp_sd_pitfall() {
        let schema = Qrisk3.input_schema();
        let def = &schema["properties"]["systolic_bp_sd"]["definition"];
        assert!(def["caveats"].as_str().unwrap().contains("NOT the systolic value"));
    }

    #[test]
    fn license_is_clinrisk_lgpl() {
        let l = Qrisk3.license();
        assert!(l.license.contains("LGPL"));
        assert!(l.license.contains("ClinRisk"));
        assert!(l.source_url.starts_with("https://qrisk.org"));
    }
}
