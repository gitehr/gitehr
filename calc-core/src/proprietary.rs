//! Proprietary / licence-locked calculators.
//!
//! Some clinical tools cannot be shipped openly because their algorithm is a
//! trade secret (e.g. FRAX) or their content is copyrighted and licence-locked
//! (e.g. the MMSE). GitEHR refuses to ship a half-right reimplementation or to
//! quietly omit them. Instead each is registered as a first-class calculator
//! whose computation returns a structured explanation: that it is proprietary,
//! who owns it, what open alternatives exist (often one GitEHR already ships),
//! and what a thwarted clinician can do about it.
//!
//! The point is transparency, not obstruction: a clinician searching for FRAX
//! finds out exactly why it is not here and where to turn, rather than silence.

use serde_json::{json, Map, Value};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Shared advice appended to every proprietary calculator's response.
pub const ADVOCACY: &str = "Clinical decision tools that public healthcare relies on should be open \
and free to use. If you agree, consider writing to your MP or elected representative to ask why \
tools essential to patient care are locked behind proprietary licences, and to support open \
clinical knowledge. Open alternatives are listed above where they exist.";

/// A calculator that cannot be shipped because it is proprietary or
/// licence-locked. Computing it returns the reason, the owner, alternatives,
/// and advocacy advice rather than a score.
#[derive(Debug, Clone, Copy)]
pub struct ProprietaryCalculator {
    pub name: &'static str,
    pub title: &'static str,
    /// One-line description of what the tool does.
    pub purpose: &'static str,
    /// Who owns / controls the rights.
    pub owner: &'static str,
    /// Why it cannot be shipped (trade-secret algorithm, copyright, etc.).
    pub reason: &'static str,
    /// Open alternatives a clinician can use instead (machine names of GitEHR
    /// calculators where one exists, or named external tools).
    pub alternatives: &'static [&'static str],
    /// A URL with more information (typically the owner's page).
    pub source_url: &'static str,
}

impl Calculator for ProprietaryCalculator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn title(&self) -> &'static str {
        self.title
    }

    fn description(&self) -> &'static str {
        self.purpose
    }

    fn reference(&self) -> &'static str {
        self.source_url
    }

    fn license(&self) -> CalculatorLicense {
        CalculatorLicense {
            license: "Proprietary / licence-locked - not freely distributable",
            source_url: self.source_url,
        }
    }

    fn input_schema(&self) -> Value {
        // No inputs: computing it only ever returns the explanation.
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": self.title,
            "type": "object",
            "additionalProperties": false,
            "properties": {},
            "description": "Proprietary calculator: takes no inputs and returns an explanation of why it cannot be shipped."
        })
    }

    fn calculate(&self, _input: &Value) -> Result<CalculationResponse, CalcError> {
        let mut working = Map::new();
        working.insert("status".into(), json!("unavailable-proprietary"));
        working.insert("owner".into(), json!(self.owner));
        working.insert("reason".into(), json!(self.reason));
        working.insert("alternatives".into(), json!(self.alternatives));
        working.insert("what_you_can_do".into(), json!(ADVOCACY));

        let interpretation = format!(
            "{title} is not available in GitEHR because it is proprietary or licence-locked. \
Owner: {owner}. {reason} {advocacy}",
            title = self.title,
            owner = self.owner,
            reason = self.reason,
            advocacy = ADVOCACY
        );

        Ok(CalculationResponse {
            calculator: self.name.to_string(),
            result: json!("unavailable: proprietary"),
            interpretation,
            working,
            reference: self.source_url.to_string(),
        })
    }
}

/// The proprietary / licence-locked tools, surfaced so clinicians learn why
/// they are absent and where to turn.
pub const PROPRIETARY: &[ProprietaryCalculator] = &[
    ProprietaryCalculator {
        name: "frax",
        title: "FRAX (10-year fracture risk)",
        purpose: "10-year probability of osteoporotic and hip fracture (NICE CG146).",
        owner: "University of Sheffield (Centre for Metabolic Bone Diseases)",
        reason: "The FRAX algorithm and its country-specific coefficients are a trade secret and \
have never been published, so it cannot be reimplemented from primary literature.",
        alternatives: &[
            "qfracture (open UK fracture-risk algorithm)",
            "Garvan Fracture Risk Calculator",
        ],
        source_url: "https://frax.shef.ac.uk/",
    },
    ProprietaryCalculator {
        name: "mmse",
        title: "MMSE (Mini-Mental State Examination)",
        purpose: "Cognitive screening / dementia monitoring (NICE NG97).",
        owner: "Psychological Assessment Resources, Inc. (PAR)",
        reason: "The MMSE has been copyright-controlled by PAR since 2001; reproducing or \
distributing the instrument requires a paid licence.",
        alternatives: &[
            "amts (Abbreviated Mental Test Score - public domain, shipped here)",
            "MoCA (Montreal Cognitive Assessment)",
            "6CIT, GPCOG (free cognitive screens)",
        ],
        source_url: "https://www.parinc.com/products/MMSE",
    },
    ProprietaryCalculator {
        name: "elf",
        title: "ELF (Enhanced Liver Fibrosis test)",
        purpose: "Second-line serum biomarker test for liver fibrosis (NICE NG49).",
        owner: "Siemens Healthineers",
        reason: "The commercial ELF score uses a proprietary, recalibrated algorithm over its \
serum biomarkers; the shipped score cannot be reproduced openly.",
        alternatives: &[
            "fib4 (FIB-4 index - first-line, shipped here)",
            "NAFLD Fibrosis Score",
            "Transient elastography (FibroScan)",
        ],
        source_url: "https://www.siemens-healthineers.com/laboratory-diagnostics/assays-by-diseases-conditions/liver-disease/elf-test",
    },
    ProprietaryCalculator {
        name: "cfs",
        title: "CFS (Clinical Frailty Scale)",
        purpose: "9-point judgement-based frailty grading in older adults (1 Very Fit to 9 Terminally Ill).",
        owner: "Dalhousie University (Geriatric Medicine Research; Kenneth Rockwood et al.)",
        reason: "The CFS is copyrighted by Dalhousie University. Non-commercial use is free but \
requires a signed permission agreement that forbids changing or commercialising the scale, and an \
EMR vendor incorporating it into its offering needs a licence. A no-modification, signed-permission \
agreement is incompatible with shipping the content under GitEHR's open AGPL licence.",
        alternatives: &[
            "Electronic Frailty Index (eFI) - open, derived from routine primary-care EHR data",
            "PRISMA-7 (7-item frailty screen)",
            "Edmonton Frail Scale",
        ],
        source_url: "https://www.dal.ca/sites/gmr/our-tools/clinical-frailty-scale.html",
    },
    ProprietaryCalculator {
        name: "lanss",
        title: "LANSS (Leeds Assessment of Neuropathic Symptoms and Signs)",
        purpose: "Screening for pain of predominantly neuropathic origin (7 items, 0-24; >=12 likely neuropathic).",
        owner: "Michael I. Bennett; published in Elsevier's journal Pain (2001), all rights reserved",
        reason: "The LANSS instrument is copyrighted and is reproduced in the literature only with \
the permission of M. Bennett. There is no public-domain or free-reuse grant, so embedding the \
scored instrument in software requires permission from the author/publisher.",
        alternatives: &[
            "DN4 (Douleur Neuropathique 4 - check licensing)",
            "painDETECT (check licensing)",
            "Clinical neuropathic pain assessment per NICE CG173",
        ],
        source_url: "https://doi.org/10.1016/S0304-3959(00)00482-6",
    },
    ProprietaryCalculator {
        name: "must",
        title: "MUST (Malnutrition Universal Screening Tool)",
        purpose: "Malnutrition risk screening (NICE CG32).",
        owner: "BAPEN (British Association for Parenteral and Enteral Nutrition)",
        reason: "BAPEN holds copyright in the MUST tool. The download is free only to individual \
professionals using it; reproducing MUST (which shipping an implementation does) requires \
applying to BAPEN for permission under a renewable, audited licence agreement, so it cannot be \
shipped under an open licence.",
        alternatives: &[
            "Clinical assessment per NICE CG32 (BMI, unplanned weight loss, acute disease effect)",
            "MNA (Mini Nutritional Assessment - check licensing)",
        ],
        source_url: "https://www.bapen.org.uk/must-and-self-screening/reproducing-must-application-form/",
    },
    ProprietaryCalculator {
        name: "cat",
        title: "CAT (COPD Assessment Test)",
        purpose: "Symptom-burden / health-status measure in COPD (8 items, 0-40; GOLD/NICE NG115).",
        owner: "GSK group of companies (CAT Governance Board: GSK / GOLD / COPD Foundation)",
        reason: "The CAT is trademarked and copyrighted by GSK. Free permissions cover only \
reproduction of the published instrument by researchers and explicitly forbid incorporating it \
into any other electronic system or means of data capture; embedding it in software requires a \
signed licence agreement with GSK.",
        alternatives: &[
            "mrc_dyspnoea (MRC dyspnoea scale - breathlessness grading, shipped here)",
            "Clinical COPD assessment per GOLD / NICE NG115",
        ],
        source_url: "https://www.catestonline.org/hcp-homepage/legal-notices.html",
    },
    ProprietaryCalculator {
        name: "acq",
        title: "ACQ (Asthma Control Questionnaire)",
        purpose: "Asthma control monitoring (BTS/NICE/SIGN asthma).",
        owner: "Elizabeth Juniper / QOL Technologies Ltd",
        reason: "The ACQ is copyrighted and its use and reproduction require a licence from the \
copyright holder.",
        alternatives: &[
            "Asthma Control Test (ACT)",
            "RCP three questions / clinical asthma control assessment",
        ],
        source_url: "https://www.qoltech.co.uk/acq.html",
    },
    ProprietaryCalculator {
        name: "ohs",
        title: "Oxford Hip Score (OHS)",
        purpose: "Patient-reported outcome after hip replacement (NHS England PROMs).",
        owner: "Oxford University Innovation",
        reason: "The Oxford Hip Score is copyrighted and its use in software requires a licence \
from Oxford University Innovation.",
        alternatives: &[
            "EQ-5D (generic PROM)",
            "HOOS / HOOS-12 (check licensing)",
        ],
        source_url: "https://innovation.ox.ac.uk/outcome-measures/oxford-hip-score-ohs/",
    },
    ProprietaryCalculator {
        name: "oks",
        title: "Oxford Knee Score (OKS)",
        purpose: "Patient-reported outcome after knee replacement (NHS England PROMs).",
        owner: "Oxford University Innovation",
        reason: "The Oxford Knee Score is copyrighted and its use in software requires a licence \
from Oxford University Innovation.",
        alternatives: &[
            "EQ-5D (generic PROM)",
            "KOOS / KOOS-12 (check licensing)",
        ],
        source_url: "https://innovation.ox.ac.uk/outcome-measures/oxford-knee-score-oks/",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frax_returns_protest_not_a_score() {
        let frax = PROPRIETARY.iter().find(|p| p.name == "frax").unwrap();
        let r = frax.calculate(&json!({})).unwrap();
        assert_eq!(r.result, json!("unavailable: proprietary"));
        assert_eq!(r.working["status"], json!("unavailable-proprietary"));
        assert!(r.working["owner"].as_str().unwrap().contains("Sheffield"));
        assert!(r.interpretation.contains("MP"));
    }

    #[test]
    fn every_proprietary_names_an_alternative_and_owner() {
        for p in PROPRIETARY {
            assert!(!p.owner.is_empty(), "{}: owner required", p.name);
            assert!(!p.alternatives.is_empty(), "{}: alternatives required", p.name);
            assert!(p.source_url.starts_with("http"), "{}: source_url", p.name);
        }
    }
}
