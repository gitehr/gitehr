<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# NHS App (web) import tool

*Status: proposal (draft), captured for future consideration. Two forks (below) are unresolved and need a decision before implementation. Relates to [`gitehr-patient-activated-extraction-synopsis.md`](gitehr-patient-activated-extraction-synopsis.md) (the three-layer patient-activated thesis this is the first concrete instance of), [`record-provenance-and-acquisition.md`](record-provenance-and-acquisition.md) (provenance block + acquisition register that every imported item points at), [`commands/import.md`](commands/import.md) (the ingest surface), and the FHIR workstream in [`fhir.md`](fhir.md).*

## Purpose

Let a patient bring their own NHS App record into a GitEHR repository with near-zero friction: allergies, immunisations, medications, problems/diagnoses, test results, documents/letters, consultations, and demographics. This is the cold-start problem the project explicitly cares about - the repo is empty until you acquire, and acquisition is the activation-energy barrier. The NHS App is the highest-value first portal because it is the citizen-facing front door to the GP record for most of the UK.

## Load-bearing architectural decision: extraction lives outside the core binary

Core `gitehr` ingests a well-defined intermediate bundle; it never drives a browser, never handles NHS login credentials, and never depends on the portal's HTML staying stable. The brittle, credential-sensitive, legally-sensitive extraction work lives in a separate agent that writes files to disk. Core `gitehr` only reads those files.

This mirrors the split GitEHR already has (`gitehr import` ingests; something else produces the inputs) and the three-layer model in the extraction synopsis (extraction agents -> canonical representation -> value tools). It keeps the release pipeline, the safety surface, and the legal surface of the core tool clean, and it means the fragile part can change on its own cadence without touching the binary that writes to patient records.

### A. Extraction agent (produces a bundle; never touches the repo)

- Drives the NHS App / NHS login web UI in the patient's *own* authenticated session.
- Scrapes each data category and normalises it into a canonical **extraction bundle** on disk.
- Auth is interactive and local-only: credentials and 2FA are never stored, never logged, never transmitted anywhere. This is a hard privacy invariant, not a preference.
- Emits files, not a live feed - same ethos as the record itself.

### B. Importer (core `gitehr`)

- `gitehr import --mode nhs <bundle>` (or a generic `--mode fhir <bundle>`) reads the bundle and writes into the repo.
- Maps each category to the best existing GitEHR home (table below), falling back to journal entries + `documents/` where no typed-state command exists yet.
- Stamps every written item with provenance and links it to a single acquisition record.
- Is idempotent: re-running never duplicates.

## Category mapping (NHS App -> GitEHR)

| NHS App category | GitEHR target | Notes |
|---|---|---|
| Demographics | `gitehr demographics set` | typed state |
| Allergies | `state/allergies.md` (typed) | one entry each + journal narrative |
| Immunisations | `state/vaccinations.md` (typed) | reuse the existing `fhir_r4` embedding of `Immunization` |
| Medications | `state/medications.md` | gated on the typed medications command landing; until then journal + documents |
| Problems / diagnoses | `state/conditions.md` | gated on [`problem-condition-list.md`](problem-condition-list.md); imported as review candidates, never silently confirmed |
| Test results | journal entries (+ future `state/observations`) | FHIR `Observation` |
| Documents / letters | `documents/` via the document workflow | linked from a journal entry |
| Consultations | journal entries | Medical Markdown where the source text supports it |

Where the typed-state target does not exist yet, the importer still captures the data losslessly as a journal entry and/or a Document, so nothing is dropped waiting on a future command.

## Provenance and acquisition

Every imported item carries the provenance block from [`record-provenance-and-acquisition.md`](record-provenance-and-acquisition.md):

- `source_type: portal-extracted`
- `source_detail: NHS App`
- `acquired_via: <acq-id>`
- `evidence_level: documented` where a source document/letter backs it, else `inferred`.

The import creates one acquisition-register record (`controller`, `contact_used`, `date_sent`/acquired, `status: received`, counts) that all imported entries reference, so "these 37 items arrived from the NHS App on 2026-07-12" is first-class, queryable provenance rather than an unattributed blob.

## Idempotency

Re-running the import must not duplicate. Key each item by its NHS App source identifier where one exists, or by a content hash otherwise; skip already-present items. This is the same guarantee `gitehr import --mode journal` already provides, extended to typed-state categories.

## Legal posture

The patient is exercising their own statutory right of access and portability (UK GDPR Article 15 and Article 20) over their own record, in a session they are authorised to use. The framing is Cory Doctorow's *adversarial interoperability*: building a tool that interoperates with an incumbent platform on the user's behalf, without the platform's cooperation. The NHS App's positioning as the citizen-facing layer weakens any "intended use" argument against patient-side automation. A defensible public statement of this posture should exist before the extraction agent is promoted beyond a prototype.

## Safety: absent must not read as "none"

The safety-critical failure mode is silent partial extraction. If the portal's markup changes and a whole category is quietly dropped, an empty allergies list reads as "no known allergies" - a potentially dangerous false negative. Mitigations, all of which belong in the bundle contract:

- The bundle `manifest.json` records **which categories were seen and their item counts**, so a missing category is a visible gap, not silent data loss.
- The importer refuses to treat a category as authoritative (e.g. to write an empty typed-state file) unless the manifest asserts that category was successfully extracted.
- Extraction fidelity is a Turva hazard: mis-mapping, stale data, wrong-patient, and silent-drop each need a hazard-log entry and a mitigation.

## Extraction bundle format (to be specified)

A directory containing:

- FHIR R4 resources per category (`AllergyIntolerance`, `Immunization`, `MedicationStatement`, `Condition`, `Observation`, `DocumentReference` + binaries, `Patient`), reusing the planned `/fhir` layout and the vaccination `fhir_r4` embedding; and
- `manifest.json`: extraction metadata (portal, timestamp, agent version), the per-category seen/count table, and provenance seed for the acquisition record.

The exact schema is deferred pending the format fork below.

## Phasing

1. Specify the extraction bundle format (FHIR R4 + manifest, including the seen/count fidelity contract).
2. Implement the importer for the categories that have a home today: `allergies`, `vaccinations`, `demographics`, `documents`, and journal entries. Idempotent, provenance-stamped, acquisition-linked.
3. Build the extraction agent (Skill or plugin - see fork) driving the NHS App web UI with local-only auth.
4. Extend the importer to `conditions` / `medications` / `observations` as those typed-state commands land.
5. Turva hazards: mis-map, stale data, wrong-patient, silent-drop.

## Open forks (decide before building)

1. **Extraction surface.** An Agent Skill (Claude-in-Chrome, matches the existing proof of concept, fastest to a working demo) versus a `gitehr-nhs` plugin (Playwright/CDP, reproducible, CI-testable, but more to build and maintain). Provisional lean: prototype as a Skill, harden into a plugin once the category mappings are proven.
2. **Bundle format.** Pure FHIR R4 (reuses the planned `/fhir` layout, the vaccination `fhir_r4` embedding, and the future `gitehr export` mapping code) versus a lighter GitEHR-native JSON (simpler now, but a second format to maintain). Provisional lean: FHIR R4, so extraction, import, and export share one mapping layer.

## Cross-references

- [`gitehr-patient-activated-extraction-synopsis.md`](gitehr-patient-activated-extraction-synopsis.md) - the strategy this instantiates (extraction agents, canonical representation, value tools).
- [`record-provenance-and-acquisition.md`](record-provenance-and-acquisition.md) - the provenance block and acquisition register imported items attach to.
- [`commands/import.md`](commands/import.md) - the existing ingest command this extends with a new mode.
- [`fhir.md`](fhir.md) - the FHIR storage/validation workstream the bundle format should share.
- [`problem-condition-list.md`](problem-condition-list.md) - the conditions typed-state that problem/diagnosis import depends on.
