//! QFracture-2012 - 10-year risk of osteoporotic and hip fracture (Hippisley-Cox & Coupland, BMJ 2009/2012).
//!
//! The open UK fracture-risk algorithm and the intended open alternative to FRAX
//! (whose coefficients are an unpublished trade secret, so FRAX ships here only as
//! a proprietary protest stub). QFracture predicts the percentage probability that
//! a person will sustain a fracture within 10 years, reported as two outcomes:
//! a major *osteoporotic* fracture (hip, vertebral, proximal humerus, or distal
//! radius) and, separately, a *hip* fracture.
//!
//! This is a sex-stratified Cox fractional-polynomial model: separate male and
//! female equations for each outcome, each summing fractional-polynomial
//! transforms of age and BMI, centred continuous predictors, conditional
//! ethnicity / smoking / alcohol coefficients, and a set of boolean comorbidity
//! and risk-factor coefficients, then converting the linear predictor through a
//! baseline-survival constant indexed by follow-up year (here, year 10).
//!
//! The four equations do not all use the same factors, exactly as published:
//! - Only the female equations use endocrine problems and HRT/oestrogen.
//! - Only the male equations use care-home residence.
//! - The osteoporotic equations use malabsorption and a parental history of
//!   osteoporosis/hip fracture; the hip equations drop malabsorption, and the
//!   female hip equation also drops parental history.
//!
//! Inputs that a given equation does not use are simply ignored for that
//! sex/outcome (mirroring how the source omits them), so the same input set can
//! drive every model without contradictory shapes.
//!
//! Coefficients, survivor baselines, and centring constants are transcribed
//! verbatim from ClinRisk's open-source reference C implementation (Copyright 2012
//! ClinRisk Ltd., released under the LGPL v3+ to enable faithful reimplementation;
//! female equations from the `*_2_0` sources, male from the `*_2_1` sources), and
//! validated digit-for-digit against scores produced by compiling and running that
//! original C algorithm (see the tests below).
//!
//! Two subtleties are encoded to avoid silent error:
//! - BMI is derived from height and weight rather than accepted directly. The
//!   model is highly sensitive to BMI, and the fractional-polynomial transform
//!   makes a wrong BMI hard to spot; deriving it removes that footgun.
//! - Smoking has five ordered categories, alcohol six, and ethnicity nine, each
//!   carrying its own coefficient, so they are modelled as enums rather than free
//!   integers.
//!
//! Per ClinRisk's licence terms, the official disclaimer ([`DISCLAIMER`]) is
//! carried in every response: an inaccurate implementation could lead to wrong
//! treatment, so the score must be checked against the original at qfracture.org.

// The coefficients below carry more decimal places than an f64 can represent;
// they are kept exactly as ClinRisk published them so the transcription is
// verifiable digit-for-digit against the source. Rust rounds each to the nearest
// f64 just as the original C compiler does, so the extra digits are faithful, not
// misleading - hence the allow rather than truncating the published values.
#![allow(clippy::excessive_precision)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "qfracture";

/// Primary citation.
pub const REFERENCE: &str = "Hippisley-Cox J, Coupland C. Derivation and validation of updated QFracture algorithm to \
predict risk of osteoporotic fracture in primary care in the United Kingdom: prospective open \
cohort study. BMJ. 2012;344:e3427. doi:10.1136/bmj.e3427. Open UK alternative to FRAX (NICE CG146/NG6).";

/// Distribution licence: ClinRisk Ltd. released the QFracture-2012 algorithm
/// source under the LGPL v3+ specifically to enable faithful reimplementation;
/// the coefficients here are transcribed verbatim from that source.
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "LGPL-3.0-or-later - QFracture-2012 algorithm Copyright 2012 ClinRisk Ltd.",
    source_url: "https://qfracture.org/src.php",
};

/// ClinRisk's required disclaimer, carried alongside every score per the licence
/// terms. Inaccurate implementations can lead to wrong treatment, so the result
/// must be checked against the original algorithm at qfracture.org.
pub const DISCLAIMER: &str = "QFracture-2012 algorithm Copyright 2012 ClinRisk Ltd., used under the \
LGPL. ClinRisk Ltd. stress that it is the responsibility of the end user to check that this \
implementation produces the same results as the original code at https://qfracture.org. Inaccurate \
implementations of risk scores can lead to wrong patients being given the wrong treatment.";

/// Biological sex, selecting the male or female equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Self-reported ethnicity, in the nine QFracture categories. Each carries its
/// own coefficient; "white or not stated" is the reference (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ethnicity {
    /// White or not recorded (reference category).
    WhiteOrNotStated,
    Indian,
    Pakistani,
    Bangladeshi,
    OtherAsian,
    BlackCaribbean,
    BlackAfrican,
    Chinese,
    /// Other ethnic group, including mixed.
    OtherEthnicGroup,
}

impl Ethnicity {
    /// 1-based index into the model's `Iethrisk` array (matching the published
    /// source, where index 1 = white/not recorded maps to coefficient 0).
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

/// Smoking status, in the five QFracture categories. "Non-smoker" is the
/// reference (coefficient 0).
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
    /// 0-based smoking category, matching the source's `Ismoke` array
    /// (0 = non-smoker = reference).
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

/// Alcohol consumption, in the six QFracture categories. "None" is the reference
/// (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alcohol {
    /// None.
    None,
    /// Trivial: less than 1 unit a day.
    Trivial,
    /// Light: 1 to 2 units a day.
    Light,
    /// Medium: 3 to 6 units a day.
    Medium,
    /// Heavy: 7 to 9 units a day.
    Heavy,
    /// Very heavy: more than 9 units a day.
    VeryHeavy,
}

impl Alcohol {
    /// 0-based alcohol category, matching the source's `Ialcohol` array
    /// (0 = none = reference).
    fn cat(self) -> usize {
        match self {
            Alcohol::None => 0,
            Alcohol::Trivial => 1,
            Alcohol::Light => 2,
            Alcohol::Medium => 3,
            Alcohol::Heavy => 4,
            Alcohol::VeryHeavy => 5,
        }
    }
}

/// QFracture-2012 inputs. BMI is derived from height and weight.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QfractureInput {
    /// Age in years. QFracture-2012 is validated for ages 30-100.
    pub age: u8,
    pub sex: Sex,
    pub ethnicity: Ethnicity,
    pub smoking: Smoking,
    pub alcohol: Alcohol,
    /// Height in centimetres (for BMI).
    pub height_cm: f64,
    /// Weight in kilograms (for BMI).
    pub weight_kg: f64,
    /// On a tricyclic or other antidepressant (two or more prescriptions).
    pub antidepressant: bool,
    /// Any cancer.
    pub any_cancer: bool,
    /// Asthma or chronic obstructive pulmonary disease.
    pub asthma_or_copd: bool,
    /// Resident of a care or nursing home (male equations only; ignored for females).
    pub care_home: bool,
    /// On systemic corticosteroids (two or more prescriptions).
    pub corticosteroids: bool,
    /// Cardiovascular disease.
    pub cardiovascular_disease: bool,
    /// Dementia.
    pub dementia: bool,
    /// Other endocrine problems: thyrotoxicosis, hyperparathyroidism, or Cushing's
    /// syndrome (female equations only; ignored for males).
    pub endocrine_problems: bool,
    /// Epilepsy or on anticonvulsants.
    pub epilepsy_or_anticonvulsants: bool,
    /// History of falls.
    pub history_of_falls: bool,
    /// Previous osteoporotic fracture (wrist, spine, hip, or shoulder).
    pub previous_fracture: bool,
    /// On HRT / oestrogen-only HRT (female equations only; ignored for males).
    pub hrt: bool,
    /// Chronic liver disease.
    pub liver_disease: bool,
    /// Malabsorption: Crohn's, ulcerative colitis, coeliac disease, steatorrhoea,
    /// or blind-loop syndrome (osteoporotic equations only; ignored for the hip
    /// outcome).
    pub malabsorption: bool,
    /// Parkinson's disease.
    pub parkinsons: bool,
    /// Rheumatoid arthritis or SLE.
    pub rheumatoid_or_sle: bool,
    /// Chronic renal disease.
    pub renal_disease: bool,
    /// Type 1 diabetes.
    pub type1_diabetes: bool,
    /// Type 2 diabetes.
    pub type2_diabetes: bool,
    /// Parental history of osteoporosis or hip fracture (not used by the female
    /// hip equation; ignored there).
    pub parental_osteoporosis: bool,
}

/// The computed outcome: both fracture risks plus the derived BMI.
#[derive(Debug, Clone, PartialEq)]
pub struct QfractureOutcome {
    /// 10-year risk of a major osteoporotic fracture, as a percentage rounded to
    /// one decimal place.
    pub osteoporotic_percent: f64,
    /// 10-year risk of a hip fracture, as a percentage rounded to one decimal place.
    pub hip_percent: f64,
    /// The derived BMI (kg/m^2) used in the model.
    pub bmi: f64,
    pub interpretation: String,
}

fn b(flag: bool) -> f64 {
    if flag { 1.0 } else { 0.0 }
}

/// Convert a centred linear predictor and a 10-year baseline survivor into a
/// percentage risk, as the source does: `100 * (1 - survivor^exp(a))`.
fn score(survivor10: f64, a: f64) -> f64 {
    100.0 * (1.0 - survivor10.powf(a.exp()))
}

// --- Osteoporotic fracture (qfracture4) ---------------------------------------

/// Female osteoporotic-fracture linear predictor and score, 10 years.
/// Transcribed verbatim from `Q74_qfracture4_2012_2_0.c` (`fracture4_female_raw`).
fn osteoporotic_female_raw(i: &QfractureInput, bmi: f64) -> f64 {
    let survivor10 = 0.983377099037170_f64;

    let ialcohol = [
        0.0,
        0.0002414945264996203800000,
        0.0531971614510470740000000,
        0.1624289372927301400000000,
        0.4778223231666232600000000,
        0.6270597140515218300000000,
    ];
    let iethrisk = [
        0.0,
        0.0,
        -0.2875917367450486200000000,
        -0.7824524516248326800000000,
        -0.8172794063622931300000000,
        -0.5861737865251788200000000,
        -1.4935356591327420000000000,
        -0.7355039455837261200000000,
        -0.4900951523299932300000000,
        -0.4546040850271730900000000,
    ];
    let ismoke = [
        0.0,
        0.0371938876652497460000000,
        0.0951525414150192620000000,
        0.1221740242710975300000000,
        0.1611412668468513200000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(2.0);
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(-1.0);

    age_1 -= 26.453824996948242;
    age_2 -= 136.060699462890620;
    bmi_1 -= 0.385703802108765;

    let mut a = 0.0;

    a += ialcohol[i.alcohol.cat()];
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 0.1437995480730194500000000;
    a += age_2 * -0.0093249719419669745000000;
    a += bmi_1 * 2.9094622051196999000000000;

    a += b(i.antidepressant) * 0.3175542392827512800000000;
    a += b(i.any_cancer) * 0.2384763167407743000000000;
    a += b(i.asthma_or_copd) * 0.2389060345873167400000000;
    a += b(i.corticosteroids) * 0.1926383637036637200000000;
    a += b(i.cardiovascular_disease) * 0.1914278981809385300000000;
    a += b(i.dementia) * 0.6757597945847583200000000;
    a += b(i.endocrine_problems) * 0.2105749527624362900000000;
    a += b(i.epilepsy_or_anticonvulsants) * 0.4297240630789712600000000;
    a += b(i.history_of_falls) * 0.4505018230780948300000000;
    a += b(i.previous_fracture) * 0.0804836468689180270000000;
    a += b(i.hrt) * -0.1586145398766347600000000;
    a += b(i.liver_disease) * 0.6391726322367494700000000;
    a += b(i.malabsorption) * 0.1547851620897652300000000;
    a += b(i.parkinsons) * 0.4958354577680105800000000;
    a += b(i.rheumatoid_or_sle) * 0.2888701063403104600000000;
    a += b(i.renal_disease) * 0.2390562428559968600000000;
    a += b(i.type1_diabetes) * 0.6523717632491761200000000;
    a += b(i.type2_diabetes) * 0.2355143698342233000000000;
    a += b(i.parental_osteoporosis) * 0.5517999076133333100000000;

    score(survivor10, a)
}

/// Male osteoporotic-fracture linear predictor and score, 10 years.
/// Transcribed verbatim from `Q74_qfracture4_2012_2_1.c` (`fracture4_male_raw`).
fn osteoporotic_male_raw(i: &QfractureInput, bmi: f64) -> f64 {
    let survivor10 = 0.994551837444305_f64;

    let ialcohol = [
        0.0,
        -0.0753424993511384030000000,
        0.0035640920160520625000000,
        0.1107180929467958700000000,
        0.2772772729818878100000000,
        0.7629384134280495800000000,
    ];
    let iethrisk = [
        0.0,
        0.0,
        -0.2578247985190295600000000,
        -0.2739691601862618800000000,
        -1.2488100943578264000000000,
        -0.4478136903122282900000000,
        -0.9569833717832930700000000,
        -0.6454670770263975000000000,
        -0.2441668713268753100000000,
        -0.5585671879728931800000000,
    ];
    let ismoke = [
        0.0,
        -0.0008039513520016420400000,
        0.1560272763218023000000000,
        0.2511740981322320700000000,
        0.2796740114008822700000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(0.5);
    let mut age_2 = dage;
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(-1.0);
    let mut bmi_2 = dbmi.powf(-0.5);

    age_1 -= 2.213409662246704;
    age_2 -= 4.899182319641113;
    bmi_1 -= 0.376987010240555;
    bmi_2 -= 0.613992691040039;

    let mut a = 0.0;

    a += ialcohol[i.alcohol.cat()];
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * -9.0010590056070825000000000;
    a += age_2 * 2.4013416577413533000000000;
    a += bmi_1 * 18.1789865484634670000000000;
    a += bmi_2 * -18.9164740466035500000000000;

    a += b(i.antidepressant) * 0.4687193755788741600000000;
    a += b(i.any_cancer) * 0.4507500533865196300000000;
    a += b(i.asthma_or_copd) * 0.2886693311011971400000000;
    a += b(i.care_home) * 0.4624017599741130900000000;
    a += b(i.corticosteroids) * 0.2959070482702296200000000;
    a += b(i.cardiovascular_disease) * 0.2342575101174369000000000;
    a += b(i.dementia) * 0.6410107589079159200000000;
    a += b(i.epilepsy_or_anticonvulsants) * 0.7821394592420207700000000;
    a += b(i.history_of_falls) * 0.5427801687901475700000000;
    a += b(i.previous_fracture) * 0.3037648317094442400000000;
    a += b(i.liver_disease) * 0.9492983471493211500000000;
    a += b(i.malabsorption) * 0.2198043397723023800000000;
    a += b(i.parkinsons) * 0.8971315042849318200000000;
    a += b(i.rheumatoid_or_sle) * 0.4403191212798893100000000;
    a += b(i.renal_disease) * 0.4565029417822387700000000;
    a += b(i.type1_diabetes) * 0.8447272010743575000000000;
    a += b(i.type2_diabetes) * 0.2219385025905733500000000;
    a += b(i.parental_osteoporosis) * 1.6999403855072708000000000;

    score(survivor10, a)
}

// --- Hip fracture (qnof, neck of femur) ---------------------------------------

/// Female hip-fracture linear predictor and score, 10 years.
/// Transcribed verbatim from `Q74_qnof_2012_2_0.c` (`nof_female_raw`).
/// Note: the female hip equation uses neither malabsorption nor parental history.
fn hip_female_raw(i: &QfractureInput, bmi: f64) -> f64 {
    let survivor10 = 0.998187243938446_f64;

    let ialcohol = [
        0.0,
        -0.1286446642326926600000000,
        -0.0997737785682041020000000,
        0.0542649888398008330000000,
        0.4431543152512633600000000,
        0.6633035785016026000000000,
    ];
    let iethrisk = [
        0.0,
        0.0,
        -0.5145680493118204300000000,
        -0.7809041138976792200000000,
        -0.5845922612624047100000000,
        -0.5418443926512139800000000,
        -1.3017049081958438000000000,
        -2.3170037513024733000000000,
        -1.0406259543469680000000000,
        -0.7087921758363630000000000,
    ];
    let ismoke = [
        0.0,
        0.0794965480333605090000000,
        0.2835141126936128200000000,
        0.3121458383725539400000000,
        0.4798404329218986000000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(2.0);
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(-2.0);

    age_1 -= 26.304763793945313;
    age_2 -= 134.912322998046870;
    bmi_1 -= 0.148731395602226;

    let mut a = 0.0;

    a += ialcohol[i.alcohol.cat()];
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 0.2707690096878486700000000;
    a += age_2 * -0.0178911237651764690000000;
    a += bmi_1 * 5.7829320185166670000000000;

    a += b(i.antidepressant) * 0.3327369489309151000000000;
    a += b(i.any_cancer) * 0.2685285338413267400000000;
    a += b(i.asthma_or_copd) * 0.2109878042448810100000000;
    a += b(i.corticosteroids) * 0.1700600172324418800000000;
    a += b(i.cardiovascular_disease) * 0.2023318326470322500000000;
    a += b(i.dementia) * 0.9427384234437306000000000;
    a += b(i.endocrine_problems) * 0.2873219609276543900000000;
    a += b(i.epilepsy_or_anticonvulsants) * 0.4800826666037592000000000;
    a += b(i.history_of_falls) * 0.4341031983222869400000000;
    a += b(i.previous_fracture) * 0.5498089896441453700000000;
    a += b(i.hrt) * -0.2717838245725980300000000;
    a += b(i.liver_disease) * 0.6449388096998517300000000;
    a += b(i.parkinsons) * 0.7086700991849171900000000;
    a += b(i.rheumatoid_or_sle) * 0.5226829686337491900000000;
    a += b(i.renal_disease) * 0.4121883090139577500000000;
    a += b(i.type1_diabetes) * 1.5320881578751737000000000;
    a += b(i.type2_diabetes) * 0.4487045379402456700000000;

    score(survivor10, a)
}

/// Male hip-fracture linear predictor and score, 10 years.
/// Transcribed verbatim from `Q74_qnof_2012_2_1.c` (`nof_male_raw`).
/// Note: the male hip equation does not use malabsorption.
fn hip_male_raw(i: &QfractureInput, bmi: f64) -> f64 {
    let survivor10 = 0.999112963676453_f64;

    let ialcohol = [
        0.0,
        -0.1883071508763912700000000,
        -0.1456237141772618900000000,
        -0.1131015985038896200000000,
        0.2669108383852995000000000,
        0.7159049108970482200000000,
    ];
    let iethrisk = [
        0.0,
        0.0,
        -0.4720554035932271700000000,
        -0.4404885564307023900000000,
        -2.0311044284508650000000000,
        -0.8877544935355209400000000,
        -1.5093354044488063000000000,
        -0.1169655869663822200000000,
        -0.7810018330580403800000000,
        -0.2253671795533221900000000,
    ];
    let ismoke = [
        0.0,
        -0.0156465395681702860000000,
        0.2947168223225690200000000,
        0.4319073634973120700000000,
        0.4937619134916043700000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(3.0);
    let mut age_2 = dage.powf(3.0) * dage.ln();
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(-2.0);

    age_1 -= 117.376983642578130;
    age_2 -= 186.449066162109370;
    bmi_1 -= 0.142089113593102;

    let mut a = 0.0;

    a += ialcohol[i.alcohol.cat()];
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 0.0470956645877030970000000;
    a += age_2 * -0.0173232541198013180000000;
    a += bmi_1 * 6.9051198985719147000000000;

    a += b(i.antidepressant) * 0.5222696860482879400000000;
    a += b(i.any_cancer) * 0.3904642661797034200000000;
    a += b(i.asthma_or_copd) * 0.2955316362120945000000000;
    a += b(i.care_home) * 0.7180133962015686800000000;
    a += b(i.corticosteroids) * 0.1637845766085505300000000;
    a += b(i.cardiovascular_disease) * 0.2685578286436679000000000;
    a += b(i.dementia) * 0.9660867715544014800000000;
    a += b(i.epilepsy_or_anticonvulsants) * 0.8977271850145135400000000;
    a += b(i.history_of_falls) * 0.5314298176292541200000000;
    a += b(i.previous_fracture) * 0.7025297516317516900000000;
    a += b(i.liver_disease) * 0.7566576273364045100000000;
    a += b(i.parkinsons) * 1.0980688140356138000000000;
    a += b(i.rheumatoid_or_sle) * 0.6434807364258057200000000;
    a += b(i.renal_disease) * 0.5918218708907634400000000;
    a += b(i.type1_diabetes) * 1.5742324490573854000000000;
    a += b(i.type2_diabetes) * 0.2887768858842130200000000;
    a += b(i.parental_osteoporosis) * 1.2332490177632631000000000;

    score(survivor10, a)
}

/// Round a percentage to one decimal place, as QFracture reports.
fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Pure scoring: the QFracture-2012 model. BMI is derived from height and weight.
pub fn compute(input: &QfractureInput) -> Result<QfractureOutcome, CalcError> {
    if input.age < 30 || input.age > 100 {
        return Err(CalcError::InvalidInput(
            "QFracture-2012 is validated for ages 30-100".into(),
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

    let height_m = input.height_cm / 100.0;
    let bmi = input.weight_kg / (height_m * height_m);

    let (osteoporotic, hip) = match input.sex {
        Sex::Female => (
            osteoporotic_female_raw(input, bmi),
            hip_female_raw(input, bmi),
        ),
        Sex::Male => (osteoporotic_male_raw(input, bmi), hip_male_raw(input, bmi)),
    };

    let osteoporotic_percent = r1(osteoporotic);
    let hip_percent = r1(hip);

    let interpretation = format!(
        "10-year QFracture-2012 risk: major osteoporotic fracture {osteoporotic_percent}%, hip \
fracture {hip_percent}%. NICE (CG146/NG6) suggests considering treatment to reduce fracture risk \
when the 10-year risk is around 10% or more, alongside clinical judgement and, where indicated, a \
DXA bone-density scan. BMI used: {bmi:.1} kg/m2. {DISCLAIMER}"
    );

    Ok(QfractureOutcome {
        osteoporotic_percent,
        hip_percent,
        bmi,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &QfractureInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert(
        "osteoporotic_fracture_percent".into(),
        json!(o.osteoporotic_percent),
    );
    working.insert("hip_fracture_percent".into(), json!(o.hip_percent));
    working.insert("bmi".into(), json!((o.bmi * 10.0).round() / 10.0));
    working.insert("sex".into(), json!(input.sex));
    working.insert("disclaimer".into(), json!(DISCLAIMER));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        // Primary result is the headline major-osteoporotic-fracture risk; the hip
        // risk is in the working alongside it.
        result: json!(o.osteoporotic_percent),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Qfracture;

impl Calculator for Qfracture {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "QFracture (10-year fracture risk)"
    }

    fn description(&self) -> &'static str {
        "10-year risk of major osteoporotic and hip fracture (QFracture-2012), the open UK alternative to FRAX (NICE CG146/NG6)."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        // The schema is assembled property-by-property into a Map rather than as
        // one giant `json!` object literal: that keeps every `json!` invocation
        // small, avoiding the `json_internal!` recursion-limit the full literal
        // would hit (the crate sets no raised `recursion_limit`).
        let source = json!({
            "citation": "Hippisley-Cox J, Coupland C. BMJ. 2012;344:e3427.",
            "url": "https://doi.org/10.1136/bmj.e3427"
        });

        // A plain boolean factor with no extended definition.
        let plain = |description: &str| json!({ "type": "boolean", "description": description });

        let mut props = Map::new();

        props.insert(
            "age".into(),
            json!({
                "type": "integer",
                "minimum": 30,
                "maximum": 100,
                "description": "Age in years (QFracture-2012 is validated for 30-100)"
            }),
        );
        props.insert(
            "sex".into(),
            json!({
                "type": "string",
                "enum": ["male", "female"],
                "description": "Biological sex, selecting the male or female equation",
                "definition": {
                    "concept": "Sex for the QFracture equation",
                    "statement": "The sex whose sex-specific QFracture equations are used. Endocrine problems and HRT apply to the female equations; care-home residence to the male equations.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "ethnicity".into(),
            json!({
                "type": "string",
                "enum": [
                    "white_or_not_stated", "indian", "pakistani", "bangladeshi", "other_asian",
                    "black_caribbean", "black_african", "chinese", "other_ethnic_group"
                ],
                "description": "Self-reported ethnicity in the nine QFracture categories",
                "definition": {
                    "concept": "Ethnicity (9 QFracture categories)",
                    "statement": "Self-reported ethnicity; 'white or not recorded' is the reference category.",
                    "caveats": "These are the exact categories the model was fitted on; use 'white_or_not_stated' when ethnicity is unknown, matching the official tool's default.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "smoking".into(),
            json!({
                "type": "string",
                "enum": ["non_smoker", "ex_smoker", "light_smoker", "moderate_smoker", "heavy_smoker"],
                "description": "Smoking status in the five QFracture categories",
                "definition": {
                    "concept": "Smoking status (5 ordered categories)",
                    "statement": "non_smoker; ex_smoker; light_smoker (<10/day); moderate_smoker (10-19/day); heavy_smoker (20+/day).",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "alcohol".into(),
            json!({
                "type": "string",
                "enum": ["none", "trivial", "light", "medium", "heavy", "very_heavy"],
                "description": "Alcohol consumption in the six QFracture categories",
                "definition": {
                    "concept": "Alcohol consumption (6 ordered categories)",
                    "statement": "none; trivial (<1 unit/day); light (1-2 units/day); medium (3-6 units/day); heavy (7-9 units/day); very_heavy (>9 units/day).",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "height_cm".into(),
            json!({ "type": "number", "exclusiveMinimum": 0, "description": "Height in centimetres (used to derive BMI)" }),
        );
        props.insert(
            "weight_kg".into(),
            json!({ "type": "number", "exclusiveMinimum": 0, "description": "Weight in kilograms (used to derive BMI)" }),
        );
        props.insert(
            "antidepressant".into(),
            json!({
                "type": "boolean",
                "description": "On a tricyclic or other antidepressant",
                "definition": {
                    "concept": "Antidepressant use",
                    "statement": "Two or more prescriptions for a tricyclic or other antidepressant in the six months before assessment.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert("any_cancer".into(), plain("Any cancer"));
        props.insert(
            "asthma_or_copd".into(),
            json!({
                "type": "boolean",
                "description": "Asthma or chronic obstructive pulmonary disease (combined in QFracture)",
                "definition": {
                    "concept": "Asthma or COPD",
                    "statement": "A diagnosis of asthma OR chronic obstructive pulmonary disease (the model combines them into one variable).",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "care_home".into(),
            json!({
                "type": "boolean",
                "description": "Resident of a care or nursing home (male equations only; ignored for females)",
                "definition": {
                    "concept": "Care/nursing-home residence",
                    "statement": "The patient lives in a care or nursing home.",
                    "caveats": "Used by the male equations only; it has no coefficient in the female model and is ignored there.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "corticosteroids".into(),
            json!({
                "type": "boolean",
                "description": "On systemic corticosteroids",
                "definition": {
                    "concept": "Systemic corticosteroid use",
                    "statement": "Two or more prescriptions for systemic corticosteroids in the six months before assessment.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "cardiovascular_disease".into(),
            plain("Cardiovascular disease"),
        );
        props.insert("dementia".into(), plain("Dementia"));
        props.insert(
            "endocrine_problems".into(),
            json!({
                "type": "boolean",
                "description": "Thyrotoxicosis, hyperparathyroidism, or Cushing's syndrome (female equations only; ignored for males)",
                "definition": {
                    "concept": "Other endocrine problems",
                    "statement": "Thyrotoxicosis, primary or secondary hyperparathyroidism, or Cushing's syndrome.",
                    "caveats": "Used by the female equations only; it has no coefficient in the male model and is ignored there.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "epilepsy_or_anticonvulsants".into(),
            json!({
                "type": "boolean",
                "description": "Epilepsy or on anticonvulsants",
                "definition": {
                    "concept": "Epilepsy / anticonvulsant use",
                    "statement": "A diagnosis of epilepsy or being prescribed anticonvulsant medication.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert("history_of_falls".into(), plain("History of falls"));
        props.insert(
            "previous_fracture".into(),
            json!({
                "type": "boolean",
                "description": "Previous osteoporotic fracture (wrist, spine, hip, or shoulder)",
                "definition": {
                    "concept": "Previous fragility fracture",
                    "statement": "A prior osteoporotic fracture of the wrist, spine, hip, or shoulder.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert(
            "hrt".into(),
            json!({
                "type": "boolean",
                "description": "On HRT / oestrogen (female equations only; ignored for males)",
                "definition": {
                    "concept": "HRT / oestrogen use",
                    "statement": "Two or more prescriptions for hormone replacement therapy in the six months before assessment.",
                    "caveats": "Used by the female equations only; it has no coefficient in the male model and is ignored there. It is protective (lowers predicted risk).",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert("liver_disease".into(), plain("Chronic liver disease"));
        props.insert(
            "malabsorption".into(),
            json!({
                "type": "boolean",
                "description": "Malabsorption (Crohn's, ulcerative colitis, coeliac, steatorrhoea, blind-loop) - osteoporotic outcome only; ignored for the hip outcome",
                "definition": {
                    "concept": "Gastrointestinal malabsorption",
                    "statement": "A condition likely to cause malabsorption: Crohn's disease, ulcerative colitis, coeliac disease, steatorrhoea, or blind-loop syndrome.",
                    "caveats": "Used by the osteoporotic-fracture equations only; the hip-fracture equations have no malabsorption coefficient and ignore it.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert("parkinsons".into(), plain("Parkinson's disease"));
        props.insert(
            "rheumatoid_or_sle".into(),
            json!({
                "type": "boolean",
                "description": "Rheumatoid arthritis or systemic lupus erythematosus (combined in QFracture)",
                "definition": {
                    "concept": "Rheumatoid arthritis or SLE",
                    "statement": "A diagnosis of rheumatoid arthritis OR systemic lupus erythematosus (the model combines them into one variable).",
                    "source": source,
                    "status": "draft"
                }
            }),
        );
        props.insert("renal_disease".into(), plain("Chronic renal disease"));
        props.insert("type1_diabetes".into(), plain("Type 1 diabetes"));
        props.insert("type2_diabetes".into(), plain("Type 2 diabetes"));
        props.insert(
            "parental_osteoporosis".into(),
            json!({
                "type": "boolean",
                "description": "Parental history of osteoporosis or hip fracture (not used by the female hip equation)",
                "definition": {
                    "concept": "Parental osteoporosis / hip fracture",
                    "statement": "A first-degree (parental) history of osteoporosis or hip fracture.",
                    "caveats": "Used by both osteoporotic equations and the male hip equation; the female hip equation has no coefficient for it and ignores it.",
                    "source": source,
                    "status": "draft"
                }
            }),
        );

        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "QfractureInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "sex", "ethnicity", "smoking", "alcohol", "height_cm", "weight_kg",
                "antidepressant", "any_cancer", "asthma_or_copd", "care_home", "corticosteroids",
                "cardiovascular_disease", "dementia", "endocrine_problems",
                "epilepsy_or_anticonvulsants", "history_of_falls", "previous_fracture", "hrt",
                "liver_disease", "malabsorption", "parkinsons", "rheumatoid_or_sle", "renal_disease",
                "type1_diabetes", "type2_diabetes", "parental_osteoporosis"
            ],
            "properties": Value::Object(props)
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: QfractureInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A neutral baseline: white, non-smoker, no alcohol, every factor false.
    /// Tests override fields as needed.
    fn base(age: u8, sex: Sex) -> QfractureInput {
        QfractureInput {
            age,
            sex,
            ethnicity: Ethnicity::WhiteOrNotStated,
            smoking: Smoking::NonSmoker,
            alcohol: Alcohol::None,
            height_cm: 170.0,
            weight_kg: 72.0,
            antidepressant: false,
            any_cancer: false,
            asthma_or_copd: false,
            care_home: false,
            corticosteroids: false,
            cardiovascular_disease: false,
            dementia: false,
            endocrine_problems: false,
            epilepsy_or_anticonvulsants: false,
            history_of_falls: false,
            previous_fracture: false,
            hrt: false,
            liver_disease: false,
            malabsorption: false,
            parkinsons: false,
            rheumatoid_or_sle: false,
            renal_disease: false,
            type1_diabetes: false,
            type2_diabetes: false,
            parental_osteoporosis: false,
        }
    }

    // --- Validation against ClinRisk's original C algorithm -----------------
    //
    // The reference scores below were produced by compiling ClinRisk's original
    // open-source QFracture-2012 C implementation (female from the *_2_0 sources,
    // male from the *_2_1 sources, http://svn.clinrisk.co.uk/qfracture, mirrored
    // at github.com/nhsland/clinrisk-modules) and running its command-line tools
    // at the 10-year survivor index (surv=10). For each profile the BMI was
    // derived here from height/weight (as the model does) and that exact BMI fed
    // to the C tool. Matching to one decimal place demonstrates a faithful port.

    /// Profile 1: 65F, white, 165 cm / 68 kg (BMI ~24.98), no risk factors.
    /// C: osteoporotic 4.551261%, hip 1.187594%.
    #[test]
    fn validation_female_base_clinrisk_c() {
        let mut i = base(65, Sex::Female);
        i.height_cm = 165.0;
        i.weight_kg = 68.0;
        let o = compute(&i).unwrap();
        assert_eq!(o.osteoporotic_percent, 4.6);
        assert_eq!(o.hip_percent, 1.2);
    }

    /// Profile 2: 75F, white, 160 cm / 56 kg (BMI ~21.87), heavy smoker, heavy
    /// alcohol (7-9 units), previous fracture, RA/SLE, steroids, parental
    /// osteoporosis. C: osteoporotic 45.194353%, hip 35.514128%.
    #[test]
    fn validation_female_high_risk_clinrisk_c() {
        let mut i = base(75, Sex::Female);
        i.height_cm = 160.0;
        i.weight_kg = 56.0;
        i.smoking = Smoking::HeavySmoker;
        i.alcohol = Alcohol::Heavy;
        i.previous_fracture = true;
        i.rheumatoid_or_sle = true;
        i.corticosteroids = true;
        i.parental_osteoporosis = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.osteoporotic_percent, 45.2);
        assert_eq!(o.hip_percent, 35.5);
    }

    /// Profile 3: 70M, white, 175 cm / 92 kg (BMI ~30.04), type 2 diabetes, CVD,
    /// asthma/COPD, moderate smoker. C: osteoporotic 4.146912%, hip 2.527956%.
    #[test]
    fn validation_male_comorbid_clinrisk_c() {
        let mut i = base(70, Sex::Male);
        i.height_cm = 175.0;
        i.weight_kg = 92.0;
        i.smoking = Smoking::ModerateSmoker;
        i.type2_diabetes = true;
        i.cardiovascular_disease = true;
        i.asthma_or_copd = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.osteoporotic_percent, 4.1);
        assert_eq!(o.hip_percent, 2.5);
    }

    /// Profile 4: 80M, white, 172 cm / 71 kg (BMI ~24.00), falls, Parkinson's,
    /// type 1 diabetes, care home. C: osteoporotic 47.924641%, hip 80.247719%.
    /// Exercises the male-only care-home term and a very high hip risk.
    #[test]
    fn validation_male_frail_clinrisk_c() {
        let mut i = base(80, Sex::Male);
        i.height_cm = 172.0;
        i.weight_kg = 71.0;
        i.history_of_falls = true;
        i.parkinsons = true;
        i.type1_diabetes = true;
        i.care_home = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.osteoporotic_percent, 47.9);
        assert_eq!(o.hip_percent, 80.2);
    }

    /// Profile 5: 60F, Indian, 158 cm / 60 kg (BMI ~24.03), on HRT and an
    /// antidepressant. C: osteoporotic 2.969878%, hip 0.428621%.
    /// Exercises the female-only HRT (protective) term and a non-reference ethnicity.
    #[test]
    fn validation_female_hrt_indian_clinrisk_c() {
        let mut i = base(60, Sex::Female);
        i.ethnicity = Ethnicity::Indian;
        i.height_cm = 158.0;
        i.weight_kg = 60.0;
        i.hrt = true;
        i.antidepressant = true;
        let o = compute(&i).unwrap();
        assert_eq!(o.osteoporotic_percent, 3.0);
        assert_eq!(o.hip_percent, 0.4);
    }

    // --- Behavioural / structural tests -------------------------------------

    #[test]
    fn risks_in_valid_range() {
        let o = compute(&base(60, Sex::Female)).unwrap();
        assert!(o.osteoporotic_percent >= 0.0 && o.osteoporotic_percent <= 100.0);
        assert!(o.hip_percent >= 0.0 && o.hip_percent <= 100.0);
    }

    #[test]
    fn comorbidity_raises_risk() {
        let lo = compute(&base(70, Sex::Female))
            .unwrap()
            .osteoporotic_percent;
        let mut i = base(70, Sex::Female);
        i.previous_fracture = true;
        let hi = compute(&i).unwrap().osteoporotic_percent;
        assert!(hi > lo, "previous fracture should raise risk: {lo} -> {hi}");
    }

    #[test]
    fn hrt_is_protective_for_female() {
        let mut a = base(60, Sex::Female);
        a.hrt = false;
        let mut c = base(60, Sex::Female);
        c.hrt = true;
        assert!(
            compute(&c).unwrap().osteoporotic_percent < compute(&a).unwrap().osteoporotic_percent,
            "HRT should lower predicted risk"
        );
    }

    #[test]
    fn hrt_ignored_for_male() {
        let mut a = base(60, Sex::Male);
        a.hrt = false;
        let mut c = base(60, Sex::Male);
        c.hrt = true;
        assert_eq!(
            compute(&a).unwrap().osteoporotic_percent,
            compute(&c).unwrap().osteoporotic_percent
        );
    }

    #[test]
    fn care_home_ignored_for_female() {
        let mut a = base(80, Sex::Female);
        a.care_home = false;
        let mut c = base(80, Sex::Female);
        c.care_home = true;
        assert_eq!(
            compute(&a).unwrap().osteoporotic_percent,
            compute(&c).unwrap().osteoporotic_percent
        );
    }

    #[test]
    fn care_home_affects_male() {
        let mut a = base(80, Sex::Male);
        a.care_home = false;
        let mut c = base(80, Sex::Male);
        c.care_home = true;
        assert!(
            compute(&c).unwrap().osteoporotic_percent > compute(&a).unwrap().osteoporotic_percent
        );
    }

    #[test]
    fn malabsorption_ignored_for_hip() {
        // Malabsorption affects the osteoporotic outcome but not the hip outcome.
        let mut a = base(70, Sex::Female);
        a.malabsorption = false;
        let mut c = base(70, Sex::Female);
        c.malabsorption = true;
        let oa = compute(&a).unwrap();
        let oc = compute(&c).unwrap();
        assert!(oc.osteoporotic_percent > oa.osteoporotic_percent);
        assert_eq!(oc.hip_percent, oa.hip_percent);
    }

    #[test]
    fn parental_history_ignored_for_female_hip() {
        // Parental osteoporosis affects the female osteoporotic outcome but not
        // the female hip outcome (no coefficient in nof_female).
        let mut a = base(70, Sex::Female);
        a.parental_osteoporosis = false;
        let mut c = base(70, Sex::Female);
        c.parental_osteoporosis = true;
        let oa = compute(&a).unwrap();
        let oc = compute(&c).unwrap();
        assert!(oc.osteoporotic_percent > oa.osteoporotic_percent);
        assert_eq!(oc.hip_percent, oa.hip_percent);
    }

    #[test]
    fn rejects_out_of_range_age() {
        assert!(compute(&base(29, Sex::Male)).is_err());
        assert!(compute(&base(101, Sex::Male)).is_err());
    }

    #[test]
    fn rejects_bad_anthropometry() {
        let mut i = base(60, Sex::Male);
        i.height_cm = 0.0;
        assert!(compute(&i).is_err());
        let mut j = base(60, Sex::Male);
        j.weight_kg = -1.0;
        assert!(compute(&j).is_err());
    }

    #[test]
    fn response_carries_disclaimer_and_both_risks() {
        let r = build_response(&base(65, Sex::Female)).unwrap();
        assert_eq!(r.calculator, "qfracture");
        assert!(
            r.working["disclaimer"]
                .as_str()
                .unwrap()
                .contains("ClinRisk")
        );
        assert!(r.working.contains_key("osteoporotic_fracture_percent"));
        assert!(r.working.contains_key("hip_fracture_percent"));
        assert!(r.interpretation.contains("qfracture.org"));
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let value = json!({
            "age": 65, "sex": "female", "ethnicity": "white_or_not_stated",
            "smoking": "non_smoker", "alcohol": "none", "height_cm": 165.0, "weight_kg": 68.0,
            "antidepressant": false, "any_cancer": false, "asthma_or_copd": false,
            "care_home": false, "corticosteroids": false, "cardiovascular_disease": false,
            "dementia": false, "endocrine_problems": false, "epilepsy_or_anticonvulsants": false,
            "history_of_falls": false, "previous_fracture": false, "hrt": false,
            "liver_disease": false, "malabsorption": false, "parkinsons": false,
            "rheumatoid_or_sle": false, "renal_disease": false, "type1_diabetes": false,
            "type2_diabetes": false, "parental_osteoporosis": false
        });
        let mut typed = base(65, Sex::Female);
        typed.height_cm = 165.0;
        typed.weight_kg = 68.0;
        let dynamic = Qfracture.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        assert_eq!(dynamic.result, json!(4.6));
    }

    #[test]
    fn schema_flags_sex_specific_factors() {
        let schema = Qfracture.input_schema();
        let care = &schema["properties"]["care_home"]["definition"]["caveats"];
        assert!(care.as_str().unwrap().contains("male"));
        let hrt = &schema["properties"]["hrt"]["definition"]["caveats"];
        assert!(hrt.as_str().unwrap().contains("female"));
    }

    #[test]
    fn license_is_clinrisk_lgpl() {
        let l = Qfracture.license();
        assert!(l.license.contains("LGPL"));
        assert!(l.license.contains("ClinRisk"));
        assert!(l.source_url.starts_with("https://qfracture.org"));
    }
}
