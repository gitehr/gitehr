<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# GitEHR Roadmap

Legend: `[x]` done, `[~]` in progress, `[ ]` not started. This roadmap lists outstanding work only. Every item has a stable reference code: use it in discussion, commits, and decision notes (for example, `implements R12`).

## Import and Acquisition

- [ ] **R1 - Add offline OCR for imported documents:** when importing a scan or photo with `--mode documents`, write searchable derived text alongside the original without sending clinical images to a cloud service. The original remains canonical.
- [ ] **R2 - Add further import modes:** add modes only when a concrete need arises, beginning with an imaging-scanned mode if required.
- [ ] **R3 - Add a configured document-format whitelist:** once the config format can express it, let `--mode documents` accept only configured file formats.
- [ ] **R4 - Decide and implement NHS App import:** resolve the extraction-surface and bundle-format forks in [`nhs-app-import.md`](nhs-app-import.md), then build the local-only extraction agent and idempotent provenance-stamped importer.

## Repository and Commands

- [ ] **R5 - Add `gitehr export`:** generate standardised FHIR, EHRxF, and openEHR export bundles from a repository (see [`fhir-openehr.md`](fhir-openehr.md) and [`long-term-ideas.md`](long-term-ideas.md)).
- [ ] **R6 - Extend Store identifier operations:** add `search`, `link`, `unlink`, `merge`, and `path`, plus the `GITEHR_MPI_PATH` override, as `gitehr store` subcommands.
- [ ] **R7 - Document the self-hoster on-ramp:** make families, carers, and pet owners first-class audiences in site and GUI onboarding, alongside clinics (ADR-0005).
- [ ] **R8 - Align GUI launch behaviour:** prefer the bundled `.gitehr/gitehr-gui`, then `gitehr-gui` on `$PATH`, rather than the current development launcher.

## Repository Template

- [ ] **R9 - Add the FHIR layout to the template:** add `/fhir/definitions`, `/fhir/resources`, and `/fhir/indexes`, with lifecycle documentation.
- [ ] **R10 - Add openEHR layout and storage conventions:** add `/openehr/` to the template and document its native storage model.

## FHIR v5

- [ ] **R11 - Confirm FHIR lifecycle documentation:** specify storage, journaling, and provenance rules for FHIR resources.
- [ ] **R12 - Download pinned FHIR definitions:** build tooling to place pinned FHIR v5 definitions in `/fhir/definitions`.
- [ ] **R13 - Implement Rust FHIR modules:** load definitions and validate resources in `cli/src/fhir/`.
- [ ] **R14 - Add FHIR CLI commands:** implement FHIR import and validation commands.
- [ ] **R15 - Add journal references for FHIR provenance:** connect resource changes to journal entries.
- [ ] **R16 - Add FHIR workflow tests and documentation.**

## openEHR

- [ ] **R17 - Design and implement native openEHR RM storage.**
- [ ] **R18 - Implement required openEHR REST endpoints and content negotiation.**
- [ ] **R19 - Add archetype and template validation.**
- [ ] **R20 - Implement openEHR versioning, audit, and contribution semantics.**
- [ ] **R21 - Add AQL support and the conformance manifest/OPTIONS surface.**
- [ ] **R22 - Add openEHR conformance tests and implementation documentation.**

## GUI and TUI

- [ ] **R23 - Restore GUI end-to-end coverage and keep it green in CI.**
- [ ] **R24 - Build the planned terminal user interface:** start with the smallest useful record browsing, journal, state, and status workflows (see [`gui/gui.md`](gui/gui.md) and [`../docs/tui/tui.md`](../docs/tui/tui.md)).

## Clinical Calculators

The calculator engine lives in [clincalc](https://github.com/pacharanero/clincalc). GitEHR delegates `gitehr clincalc <command>` to `gitehr-clincalc` on `$PATH`.

- [ ] **R25 - Record calculator results in the journal:** record calculator, version, inputs, result, and citation in an immutable entry.
- [ ] **R26 - Store latest calculation results:** add `state/calculations/<name>-latest.json`.
- [ ] **R27 - Add a GUI calculator panel:** expose a Tauri `calculate_clinical` command integrating with clincalc.

## Model Context Protocol

- [ ] **R28 - Implement full MCP JSON-RPC transports:** support stdio, HTTP, and SSE.
- [ ] **R29 - Add MCP resource handlers:** journal, state, imaging, documents, and status.
- [ ] **R30 - Add MCP tool handlers:** journal/state mutations, search, repository-policy checks, and clinical calculation through clincalc.
- [ ] **R31 - Add MCP prompt templates:** SOAP note, discharge summary, referral, and medication review.
- [ ] **R32 - Add token-based MCP authentication:** use `.gitehr/mcp-tokens.json`.
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

## Documentation and Operations

- [ ] **R43 - Keep command documentation aligned with runtime behaviour.**
- [ ] **R44 - Expand user-facing documentation:** installation, CLI reference, GUI walkthroughs, TUI, safety/Turva, and troubleshooting.
- [ ] **R45 - Document CLI/GUI packaging, upgrade, and migration compatibility.**
- [ ] **R46 - Add a calculator usage guide:** include clinical examples and validation references.
- [ ] **R47 - Document long-term strategic considerations:** EHDS, EHRxF, quantum cryptography, and federated learning.
- [ ] **R51 - Verify a strict Zensical build and internal links after site changes.**

## Distribution

- [ ] **R52 - Publish the CLI to crates.io:** make `cargo install gitehr --locked` available after each release.
- [ ] **R53 - Publish verified prebuilt release archives:** provide supported Linux, macOS, and Windows binaries with checksums.
- [ ] **R54 - Publish native Linux packages:** distribute `.deb` packages through an APT repository and `.rpm` packages through an RPM repository.
- [ ] **R55 - Publish an Arch Linux package:** maintain an AUR package for installation through `pacman` helpers.
- [ ] **R56 - Publish native desktop installers:** distribute signed Windows `.exe` and macOS `.dmg` installers.
- [ ] **R57 - Maintain package-manager channels:** publish and update the Homebrew formula and Scoop manifest from verified release checksums.
