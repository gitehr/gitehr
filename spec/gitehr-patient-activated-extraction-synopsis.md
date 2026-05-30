# Patient-Activated Health Records via LLM-Mediated Extraction

A strategic synopsis for GitEHR, capturing a direction that emerged from a working Claude-in-Chrome demonstration of patient-mediated NHS App data extraction.

## Thesis

GitEHR's natural product surface is not just a Git-backed clinical record format, but a three-layer system for **patient-activated** health records:

1. **Extraction agents** that liberate health data from supplier portals on the patient's behalf.
2. **Canonical representation** in GitEHR, with FHIR/openEHR mapping.
3. **Clinical calculation tools** that demonstrate the value of structured longitudinal data.

The extraction agents collapse the activation energy for keeping a personal health record current. The canonical representation makes that record queryable and clinically meaningful rather than a folder of PDFs. The calculation tools convert structured data into immediate, tangible patient utility (QRISK3, FRAX, eGFR, etc).

Each layer is necessary. None is sufficient alone. The combination is the product.

## The Proof of Concept

Claude-in-Chrome was used to extract a complete blood test panel from the NHS App web interface, producing both a structured JSON representation and a human-readable markdown summary. It worked first time, with no prior tuning. The output was immediately usable as input to a multi-panel longitudinal comparison against an earlier privately-ordered panel (Medichecks).

This demonstrates the loop end-to-end at minimal cost:

- **Extraction**: agentic browser navigation of a supplier portal that does not expose structured APIs to the patient.
- **Storage**: file-based, version-controllable, portable.
- **Interpretation**: an LLM with the full record in context can identify trends, flag thresholds, and answer clinical questions far more usefully than any portal's native UI.

Crucially, the experience felt frictionless. The same task done by hand would have been tedious enough that no one would do it routinely. That difference is the product insight.

## Adversarial Interoperability

The strategic framing is Cory Doctorow's term **adversarial interoperability**: building tools that interoperate with incumbent platforms without their permission or cooperation, on behalf of users whose data is held within them.

Health record suppliers will not voluntarily expose the APIs that would make patient-mediated data portability easy, because the walled garden is the business model. But the patient's statutory right of access does not depend on supplier cooperation. UK GDPR Article 15 (right of access) and Article 20 (right to data portability) apply to the data subject regardless of the medium of access. A patient running a browser agent against a portal they are authorised to use is exercising a statutory right, not breaching one.

The NHS App in particular is positioned as the citizen-facing layer, weakening any "intended use" defence against patient-side automation. A defensible legal posture should be articulated early in the project's public framing.

## Architecture

### Layer 1: Extraction Agents

Pattern: a library of **portal-specific Skills**, each capturing the procedural and domain knowledge required to extract data from one supplier's interface:

- NHS App
- SystmOnline
- EMIS Patient Access
- Patients Know Best
- Patient Access (TPP)
- My Health Online Wales
- Hospital trust portals
- Private providers (Medichecks, Thriva, etc)

Each Skill packages DOM patterns, navigation flows, and field-to-clinical-concept mappings. Community contribution scales naturally: each supplier's playbook can be maintained by users of that supplier.

### Layer 2: Canonical Representation

Extracted data is normalised into structured records within a GitEHR repository. This is where existing FHIR/openEHR Rust crate work earns its keep. A line like:

> Serum cholesterol level: 8.3 mmol/L on 2025-08-14

becomes a FHIR Observation with LOINC code 2093-3, quantity, unit, effective date, performer, fasting status, and provenance metadata pointing back to the extraction source.

This layer is what differentiates GitEHR from generic personal health record apps that are functionally just tagged file storage. Without canonical representation, you cannot answer queries like "show me all LDL measurements in chronological order with fasting state" or "calculate QRISK3 using the most recent values".

### Layer 3: Clinical Calculation Tools

Each calculation is a tool that consumes the canonical record and produces a clinically meaningful output:

- **QRISK3** for cardiovascular risk
- **CHA2DS2-VASc** for stroke risk in AF
- **FRAX** for fracture risk
- **CKD-EPI 2021** for eGFR (race-free)
- **Wells score** for VTE
- **GAD-7 / PHQ-9** as self-administered instruments
- **QFracture**, **NEWS2**, **Centor**, etc

Each tool gives the patient a concrete reason to maintain structured data. The first time someone gets a QRISK3 score automatically calculated from their own record across three different blood tests, they understand what GitEHR is for.

## Strategic Positioning

**GitEHR is the substrate. The extraction agents and calculation tools are what make it a product.**

Schema-first thinking dominates clinical informatics for good reasons, but understates how much user-facing value lives in agentic frontends and analytic backends. The schema layer is necessary but boring; the agent and calculation layers are where adoption happens.

Other strategic notes:

- **Open source + local-first** is the right default for health data. The worst architecture for this domain is centralised SaaS with cloud LLM and quarterly ToS changes. Building the trustworthy version first lets you absorb migration when convenient-but-untrustworthy alternatives lose user trust.

- **Patient-clinician partnership framing** avoids the wall that adversarial framings hit. "Patient brings curated, structured, time-respecting summary to consultation" aligns with what overstretched clinicians actually want.

- **The patient-facing variant has a lower regulatory surface than the clinician-facing one.** Personal use, single user, no DCB0129 requirement. This may be the cleaner v1 demonstration than the clinician-facing GitEHR Tauri app.

- **Demonstrated patient demand is the only force that moves NHS supplier interoperability.** Adversarial interop with a polite community of activated users is how you generate that demand.

## Open Questions

- How is portal authentication handled across Skills, given that NHS App uses NHS login, private providers use email/password, hospital portals vary?
- What is the right unit of community contribution: per-Skill, per-supplier, per-data-type?
- How does this layer onto the existing GitEHR roadmap (FHIR/openEHR Rust crates, Tauri GUI, sct integration)?
- What does the relationship to `sct` look like for terminology binding within the canonical layer?
- Is the patient-facing variant a separate product, a sibling repo, or a layer within GitEHR?
- What is the safety/governance story (Turva)? At what tier does a personal-use patient record sit?
- Privacy posture: how should extraction agents handle credentials? Local-only, never transmitted, never logged.

## Next Steps

1. Capture the working Claude-in-Chrome NHS App extraction as a reproducible Skill.
2. Define a minimal canonical schema for the most common patient-accessible data types (lab results, medications, allergies, immunisations, problem list, letters).
3. Implement QRISK3 as the first calculation tool, end-to-end from extracted record to score.
4. Publish a public position statement on adversarial interoperability and patient data rights.
5. Invite contribution of additional portal Skills from the community.

---

*Notes from a Claude conversation, April 2026. Captured here for development in the GitEHR repository.*
