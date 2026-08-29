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
- [ ] **R72 - Prototype local research-use imaging derivation:** preserve source DICOM as an immutable Document, run explicitly non-diagnostic local models into provenance-bearing derived artifacts, require visual QC, and prohibit automatic clinical-State promotion (see [`local-imaging-model-playbook.md`](local-imaging-model-playbook.md)).

## Repository and Commands

- [ ] **R6 - Extend Store identifier operations:** add `search`, `link`, `unlink`, `merge`, and `path`, plus the `GITEHR_MPI_PATH` override, as `gitehr store` subcommands.

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
- [x] **R38 - Document MCP integration and API reference.** Published as [`docs/cli/mcp-usage.md`](../docs/cli/mcp-usage.md), with a full resource/tool reference table and corrected URI examples.
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
- [x] **R46 - Add a calculator usage guide:** include clinical examples and validation references. Published as [`docs/cli/clincalc.md`](../docs/cli/clincalc.md), now linked from CLI nav.
- [ ] **R47 - Document long-term strategic considerations:** EHDS, EHRxF, post-quantum cryptography, federated learning, genomics, streamed vitals, and purpose-scoped sharing.

## Distribution

- [ ] **R54 - Publish native Linux packages:** distribute `.deb` packages through an APT repository and `.rpm` packages through an RPM repository.
- [ ] **R55 - Publish an Arch Linux package:** maintain an AUR package for installation through `pacman` helpers.
- [ ] **R56 - Publish native desktop installers:** distribute signed Windows `.exe` and macOS `.dmg` installers.
- [ ] **R57 - Publish a Scoop manifest:** maintain a Scoop manifest from verified release checksums alongside the existing Homebrew formula.

## Code Review Findings (2026-08-29)

Findings from a bug/security review; each item records the issue and remediation so any agent can follow up. Checked and found sound: `transport/extract.rs` (uses `tar::Archive::unpack`, which sanitises entry paths - do not replace it with manual entry iteration without adding containment checks). The review was cut short by tooling faults (file reads returning the wrong file's contents), so R77 lists what remains unaudited.

- [ ] **R77 - Finish the interrupted security review:** still unaudited: `import.rs`, `store/*.rs`, `mcp/server_impl/server.rs`, `transport/create.rs`, and `upgrade.rs` (only its `update_bundled_binary` call site was seen). Since audited and cleared: `upgrade_binary.rs` (copies the running executable, no network), `git.rs` (see R80), `encrypt.rs`/`decrypt.rs` (see R79), `plugin.rs` (name validation and `$PATH`-only resolution are sound), `transport/extract.rs` (safe `tar::Archive::unpack`). Priorities for the rest: user-supplied paths joined onto a base directory (the R73/R74 pattern) and repo-local binary execution (the R78 pattern). Also still to confirm: the locked `tar` crate is >= 0.4.36 (RUSTSEC-2021-0080); 0.4.46 was claimed during review but not reliably verified.

- [ ] **R79 - Stop `gitehr encrypt` claiming encryption that never happens:** `cli/src/commands/encrypt.rs` writes `.gitehr/ENCRYPTED` and prints "Repository marked as encrypted" while encrypting nothing - every clinical file stays plaintext - and `decrypt.rs` merely deletes the marker. This is dangerous false assurance in a clinical tool. Remediation: until R67/R68 land, make both commands exit non-zero with a clear "not yet implemented" error and write no marker; alternatively hide them behind a feature flag. Verified 2026-08-29.
- [ ] **R80 - Terminate git option parsing in `git_add`:** `git_add` (`cli/src/commands/git.rs:31-33`) passes the path positionally, so a filename beginning with `-` would be parsed by git as a flag. Use `git add -- <path>` (and audit other callers of `run_git_command` for the same). Low severity (callers currently pass generated filenames); verified 2026-08-29. `git.rs` is otherwise sound - args go through `Command::args` with no shell involved.
- [x] **R73 - Fix path traversal in MCP resource reads:** `read_journal_entry` and `read_state_file` (`cli/src/commands/mcp/server_impl/resources.rs:144-145,195-196`) join the caller-supplied URI segment straight onto the repo path (`repo_path.join("state").join(filename)`), so `gitehr://repo/state/../../../../etc/passwd` reads arbitrary files. Remediation: reject any segment containing path separators or `..` (accept a bare filename only), then canonicalise and assert the result is under `repo_path`; add tests for traversal attempts.
- [x] **R74 - Fix arbitrary-file-write traversal in MCP `update_state`:** confirmed - `update_state` (`cli/src/commands/mcp/server_impl/tools.rs:171-172`) does `state_dir.join(filename)` then `fs::write`, so `filename: "../../../home/user/.bashrc"` writes outside the repository. Strictly worse than R73's read-side traversal. Remediation: same sanitisation as R73 (bare filename only, reject separators and `..`, canonicalise and assert containment), shared between resources and tools; add write-traversal tests. PR #148's tool descriptions are otherwise accurate; add a traversal bullet to its Security Considerations section before merging.
- [ ] **R76 - Validate the repository before serving MCP:** `gitehr mcp serve` (`cli/src/commands/mcp/serve.rs:13`) defaults `repo_path` to `.` and starts serving without checking that a `.gitehr` directory exists, that the path is a GitEHR repo, or that `.gitehr/ENCRYPTED` is absent (overlaps R34). PR #148 documents this honestly; the fix is to refuse to start (clear error) unless the target validates, mirroring whatever check the other repo-scoped commands use. Verified accurate as documented in PR #148.
- [x] **R75 - Make MCP `search_repository` robust to non-UTF-8 files:** `search_repository` (`tools.rs:197,214`) calls `read_to_string(...)?` on every file in `journal/` and `state/`, so a single binary or non-UTF-8 file aborts the entire search with an error. Remediation: skip unreadable/non-UTF-8 files (or read bytes and search lossily) and continue.

- [ ] **R78 - Untrusted bundled GUI binary is executed from the repo directory:** `find_gui_binary` (`cli/src/commands/gui.rs:14-32`) returns `.gitehr/gitehr-gui` (relative to the current directory) whenever that file exists, and `run` executes it (`gui.rs:48`) ahead of any `$PATH` lookup. A GitEHR repository received from another party - a transport archive (`tar` preserves the executable bit) or a clone - can carry a hostile executable at `.gitehr/gitehr-gui`, so `cd untrusted-repo && gitehr gui` is arbitrary code execution with the user's privileges. This is the "bundled binary" pattern the project ships deliberately, so the fix is a trust decision, not just a code change. Options to weigh: drop the bundled-binary launch path entirely and require an installed/`$PATH` GUI; or gate execution behind an explicit opt-in (a `--allow-bundled` flag or a recorded per-repo trust marker) with a clear prompt; at minimum verify a signature/checksum against a trusted contributor before running. Check whether any other command (for example the bundled `.gitehr/gitehr` CLI copy) executes repo-local binaries the same way, and audit `plugin.rs`'s search directories for the same repo-relative-execution risk (its plugin-*name* validation at `plugin.rs:145-152` looks sound, but the search path was not reviewed). Verification caveat: the full `gui.rs` source was never successfully delivered to the reviewing session (tooling fault) - the execution call at `gui.rs:48` is grep-confirmed, but re-verify `find_gui_binary`'s resolution order before implementing.

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
