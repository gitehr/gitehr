<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# GitEHR Roadmap

Legend: `[x]` done, `[~]` in progress, `[ ]` not started. This roadmap lists outstanding work only. Every item has a stable reference code: use it in discussion, commits, and decision notes (for example, `implements R12`).

## Import and Acquisition

- [ ] **R1 - Add offline OCR for imported documents:** when importing a scan or photo with `--mode documents`, write searchable derived text alongside the original without sending clinical images to a cloud service. The original remains canonical.
- [ ] **R2 - Add further import modes:** add modes only when a concrete need arises, beginning with an imaging-scanned mode if required.
- [ ] **R3 - Add a configured document-format whitelist:** once the config format can express it, let `--mode documents` accept only configured file formats.
- [ ] **R4 - Specify and implement NHS App import:** define the FHIR R4 extraction bundle, category-fidelity manifest, provenance/acquisition seed, and idempotent source keys, then build the local-only importer.
- [ ] **R58 - Complete NHS App recon and extraction agent v0:** complete the authenticated-session recon checklist, then build the browser-extension extractor with passive capture, DOM fallback, consent/provenance display, and a downloadable import bundle.
- [x] **R59 - Publish a patient-mediated extraction position statement:** explain the local-first, own-data, adversarial-interoperability, and safety posture for portal extraction. Published as [`docs/design/patient-mediated-extraction.md`](../docs/design/patient-mediated-extraction.md).
- [ ] **R60 - Add provenance and acquisition tracking:** add reusable fact provenance plus an acquisition/SAR register, typed commands, audit entries, and SAR-letter generation (see [`record-provenance-and-acquisition.md`](record-provenance-and-acquisition.md)).
- [ ] **R61 - Add typed Conditions, medications, and observations state:** provide audited mutations and safe import targets, including a problem-list projection over Conditions (see [`problem-condition-list.md`](problem-condition-list.md)).
- [ ] **R62 - Demonstrate QRISK3 end to end:** derive inputs from imported structured data, calculate through clincalc, and record the result with version, inputs, and citation.

## Repository and Commands

- [ ] **R6 - Extend Store identifier operations:** add `search`, `link`, `unlink`, `merge`, and `path`, plus the `GITEHR_MPI_PATH` override, as `gitehr store` subcommands.
- [~] **R7 - Document the self-hoster on-ramp:** make families, carers, and pet owners first-class audiences in site and GUI onboarding, alongside clinics (ADR-0005). Site homepage done (families/carers folded into "For Patients & Families", plus a new "For Pet Owners" card); GUI onboarding has no onboarding flow yet to update (tracked by R63).
- [ ] **R8 - Align GUI launch behaviour:** prefer the bundled `.gitehr/gitehr-gui`, then `gitehr-gui` on `$PATH`, rather than the current development launcher.

## GUI and TUI

- [ ] **R23 - Restore GUI end-to-end coverage and keep it green in CI.**
- [ ] **R24 - Build the planned terminal user interface:** start with the smallest useful record browsing, journal, state, and status workflows (see [`gui/gui.md`](gui/gui.md) and [`../docs/tui/tui.md`](../docs/tui/tui.md)).
- [ ] **R63 - Complete the five-screen clinical GUI MVP:** deliver the record selector, overview, timeline, SNOMED-coded encounter form, and typed current-state workflows described in [`DESIGN.md`](DESIGN.md).
- [ ] **R64 - Add the multi-Store GUI experience:** provide a Store chooser and switcher, recent local Stores and labels, launch-context handling, and unsaved-draft protection (ADR-0006).

## Narrative and Calculators

The calculator engine lives in [clincalc](https://github.com/pacharanero/clincalc). GitEHR delegates `gitehr clincalc <command>` to `gitehr-clincalc` on `$PATH`.

- [ ] **R25 - Record calculator results in the journal:** record calculator, version, inputs, result, and citation in an immutable entry.
- [ ] **R26 - Store latest calculation results:** add `state/calculations/<name>-latest.json`.
- [ ] **R27 - Add a GUI calculator panel:** expose a Tauri `calculate_clinical` command integrating with clincalc.
- [ ] **R65 - Add read-only Medical Markdown extraction:** expose structured extraction through `journal show`, `journal extract`, and MCP without changing canonical journal bodies.
- [ ] **R66 - Add Medical Markdown vocabulary and reviewed State promotion:** support repository registries, advisory validation, semantic GUI rendering, and explicit provenance-bearing promotion to State.

## Model Context Protocol

- [ ] **R28 - Implement full MCP JSON-RPC transports:** support stdio, HTTP, SSE, Unix sockets, and Windows named pipes.
- [ ] **R29 - Add MCP resource handlers:** journal, state, imaging, documents, and status.
- [ ] **R30 - Add MCP tool handlers:** journal/state mutations, search, repository-policy checks, and clinical calculation through clincalc.
- [ ] **R31 - Add MCP prompt templates:** SOAP note, discharge summary, referral, consultation, and medication review.
- [ ] **R32 - Add MCP authentication:** start with `.gitehr/mcp-tokens.json`, then define OAuth2 and mTLS options for remote deployments.
- [ ] **R33 - Add MCP audit logging to journal entries.**
- [ ] **R34 - Make MCP encryption-aware:** respect `.gitehr/ENCRYPTED`.
- [ ] **R35 - Add MCP configuration:** use `.gitehr/mcp.json`.
- [ ] **R36 - Integrate clincalc MCP tools:** expose each calculator's JSON Schema and response contract.
- [ ] **R37 - Add a GUI MCP client panel.**
- [ ] **R38 - Document MCP integration and API reference.**
- [ ] **R39 - Add MCP client libraries for testing.**

## Security and Integrity

- [ ] **R40 - Design the repository policy checker and server-side guardian:** enforce append-only journal and authorised-authorship invariants as described in [`repository-verification.md`](repository-verification.md).
- [ ] **R41 - Add hardware-backed contributor signing credentials:** support YubiKey/PIV/smartcard, TPM-backed keys, Secure Enclave, or equivalent, including recovery and revocation.
- [ ] **R42 - Evaluate gittuf:** assess whether its policy-controlled refs, signed access, and rollback/rewrite protection should provide the server-side guardian.
- [ ] **R67 - Decide encryption at rest and record an ADR:** choose the encryption boundary, integrity semantics, key custody and recipient lifecycle, FIPS requirement, AEAD, and acceptable metadata leakage (see [`encryption-at-rest.md`](encryption-at-rest.md)).
- [ ] **R68 - Implement repository and transport encryption:** implement authenticated encryption, recipient management, and a usable decrypt workflow once R67 is decided.
- [ ] **R69 - Extend contributor identity and signing:** add committer identity, optional GPG signing, and stable external contributor/namespace links alongside hardware credentials.
- [ ] **R70 - Anchor the genesis record externally:** register the genesis seed with an external timestamped authority to prevent false-genesis and backdating claims.

## Documentation and Operations

- [ ] **R43 - Keep command documentation aligned with runtime behaviour.**
- [ ] **R44 - Expand user-facing documentation:** installation, CLI reference, GUI walkthroughs, TUI, safety/Turva, and troubleshooting.
- [ ] **R45 - Document CLI/GUI packaging, upgrade, and migration compatibility.**
- [ ] **R46 - Add a calculator usage guide:** include clinical examples and validation references.
- [ ] **R47 - Document long-term strategic considerations:** EHDS, EHRxF, post-quantum cryptography, federated learning, genomics, streamed vitals, and purpose-scoped sharing.
- [ ] **R71 - Evaluate Open Wearables for streamed vitals integration:** assess [the-momentum/open-wearables](https://github.com/the-momentum/open-wearables) as a self-hostable unified API for wearable device data (Garmin, Whoop, Apple HealthKit, etc.) that could feed observations into GitEHR through an import adapter. MIT-licensed, FastAPI + React, early-stage.

## Distribution

- [ ] **R54 - Publish native Linux packages:** distribute `.deb` packages through an APT repository and `.rpm` packages through an RPM repository.
- [ ] **R55 - Publish an Arch Linux package:** maintain an AUR package for installation through `pacman` helpers.
- [ ] **R56 - Publish native desktop installers:** distribute signed Windows `.exe` and macOS `.dmg` installers.
- [ ] **R57 - Publish a Scoop manifest:** maintain a Scoop manifest from verified release checksums alongside the existing Homebrew formula.

## Interoperability Standards

These standards are important but deliberately sequenced after the core patient-owned record, acquisition, GUI, and safety work needed for the proof of concept.

- [ ] **R9 - Add the FHIR layout to the template:** add `/fhir/definitions`, `/fhir/resources`, and `/fhir/indexes`, with lifecycle documentation.
- [ ] **R10 - Add openEHR layout and storage conventions:** add `/openehr/` to the template and document its native storage model.
- [ ] **R11 - Decide the FHIR definitions lifecycle:** resolve pinned official definitions versus GitEHR FSH profiles, then specify storage, compilation, journaling, and provenance rules.
- [ ] **R12 - Download or compile the chosen FHIR definitions:** build tooling for the selected definitions source and place its output in `/fhir/definitions`.
- [ ] **R13 - Implement Rust FHIR modules:** load definitions and validate resources in `cli/src/fhir/`.
- [ ] **R14 - Add FHIR CLI commands:** implement FHIR import and validation commands.
- [ ] **R15 - Add journal references for FHIR provenance:** connect resource changes to journal entries.
- [ ] **R16 - Add FHIR workflow tests and documentation.**
- [ ] **R17 - Design and implement native openEHR RM storage.**
- [ ] **R18 - Implement required openEHR REST endpoints and content negotiation.**
- [ ] **R19 - Add archetype and template validation.**
- [ ] **R20 - Implement openEHR versioning, audit, and contribution semantics.**
- [ ] **R21 - Add AQL support and the conformance manifest/OPTIONS surface.**
- [ ] **R22 - Add openEHR conformance tests and implementation documentation.**
- [ ] **R5 - Add `gitehr export`:** generate standardised FHIR, EHRxF, and openEHR export bundles from a repository (see [`fhir-openehr.md`](fhir-openehr.md) and [`long-term-ideas.md`](long-term-ideas.md)).
