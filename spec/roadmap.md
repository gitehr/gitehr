# GitEHR Roadmap (Spec-Aligned)

This roadmap tracks implementation status against the current `spec/` documents.

## Core CLI and Repository Lifecycle

- [x] Implement binary bundling in `gitehr init` (`.gitehr/gitehr`).
- [x] Implement finalized journal file layout (`journal/<timestamp>-<uuid>.md`).
- [x] Implement journal YAML front matter fields: `parent_hash`, `parent_entry`, `timestamp`, optional `author`.
- [x] Implement `gitehr journal add` input modes: inline, `--file <path>`, and stdin via `--file -`.
- [x] Implement `gitehr journal show` with pagination options.
- [x] Implement `gitehr journal verify` hash-chain validation.
- [x] Implement contributor activation so journal entries include current `author`.
- [x] Implement `gitehr version` output with both GitEHR and Git versions.
- [x] Implement shell completions generation (`gitehr completions <shell>`).
- [ ] Update `gitehr init` to follow store-root flow from spec: create/use `gitehr-mpi.json`, create UUIDv7+Crockford repo directory, then initialize inside that repo.
- [ ] Add robust `gitehr journal verify --verbose` (or equivalent) failure diagnostics per spec TODO.

## Command Coverage vs Spec

- [x] Implement `gitehr state` (`list`, `get`, `set`).
- [x] Implement `gitehr remote` (`add`, `remove`/`rm`, `list`).
- [x] Implement `gitehr encrypt` placeholder marker flow.
- [x] Implement `gitehr decrypt` placeholder marker removal flow.
- [x] Implement `gitehr status` summary output.
- [x] Implement `gitehr transport` (`create`, `extract`).
- [x] Implement `gitehr user` (`create`, `add`, `enable`, `disable`, `activate`, `deactivate`, `list`) and `contributor` alias.
- [x] Implement `gitehr upgrade`.
- [x] Implement `gitehr upgrade-binary`.
- [ ] Implement `gitehr mpi` command family (`search`, `link`, `unlink`, `create`, `merge`, `list`, `path`) and MPI path override behavior.
- [ ] Align `gitehr gui` launcher with command spec (prefer bundled `.gitehr/gitehr-gui`, then PATH `gitehr-gui`; current implementation still launches dev command).

## Repository Template and Data Layout

- [x] Ensure init copies template directories from `gitehr-folder-structure/`.
- [x] Persist `.gitehr/GITEHR_VERSION` and bundled binary during init/upgrade paths.
- [ ] Add `/fhir/` layout (`definitions`, `resources`, `indexes`) to template and lifecycle docs.
- [ ] Add `/openehr/` layout and storage conventions from spec.

## FHIR v5 Workstream

- [ ] Add/confirm spec-linked lifecycle docs for FHIR storage and journaling.
- [ ] Build tooling to download pinned FHIR v5 definitions into `/fhir/definitions`.
- [ ] Implement Rust FHIR modules (`src/fhir/`) for definitions loading and resource validation.
- [ ] Add CLI commands for FHIR import and validation.
- [ ] Add journal structured references for FHIR resource provenance.
- [ ] Add tests and documentation for FHIR workflows.

## openEHR Workstream

- [ ] Design and implement native openEHR RM storage model.
- [ ] Implement required openEHR REST endpoints and content negotiation.
- [ ] Add archetype/template validation integration.
- [ ] Implement versioning/audit/contribution semantics for openEHR entities.
- [ ] Add AQL query support and conformance manifest/OPTIONS support.
- [ ] Add conformance testing and implementation documentation.

## GUI and UX

- [x] Implement GUI shell and data panels connected to CLI-backed repository data.
- [x] Implement new-entry flow from GUI to journal.
- [x] Implement repo detection/folder selection flow.
- [ ] Keep GUI launch behavior aligned with CLI command spec for bundled-binary-first execution.
- [ ] Add/restore GUI E2E coverage and keep it green in CI.

## Clinical Calculators Workstream

- [ ] Convert to Cargo workspace structure (root, gitehr-cli, gitehr-calculators, gitehr-mcp).
- [ ] Create `gitehr-calculators` crate with modular calculator library.
- [ ] Implement RCPCH digital growth charts (UK-WHO 0-4y, UK90 4-20y).
- [ ] Implement core cardiology calculators (CHADS2, CHA2DS2-VASc, Wells, GRACE, TIMI).
- [ ] Implement renal calculators (eGFR, CrCl, FENa).
- [ ] Implement respiratory calculators (CURB-65, PSI/PORT).
- [ ] Implement emergency medicine calculators (GCS, qSOFA, MEWS).
- [ ] Add `gitehr calc` CLI command with subcommands for each calculator.
- [ ] Integrate calculator results with journal (structured metadata, citations).
- [ ] Add state file storage for latest calculator results (`state/calculations/`).
- [ ] Add GUI calculator panel with dynamic forms and result display.
- [ ] Add Tauri command for calculator invocation from GUI.
- [ ] Validate all calculators against published test cases and literature.
- [ ] Document clinical references, citations, and validation studies for each calculator.

## Model Context Protocol (MCP) Server

- [ ] Create `gitehr-mcp` crate for MCP server implementation.
- [ ] Implement MCP JSON-RPC 2.0 protocol with stdio/HTTP/SSE transports.
- [ ] Add MCP resource handlers (journal, state, imaging, documents, status).
- [ ] Add MCP tool handlers (add_journal_entry, update_state, calculate_clinical, verify_journal, search).
- [ ] Add MCP prompt templates (SOAP note, discharge summary, referral, medication review).
- [ ] Implement token-based authentication with `.gitehr/mcp-tokens.json`.
- [ ] Add MCP audit logging to journal entries.
- [ ] Create `gitehr mcp serve` CLI command (stdio, HTTP, config-based).
- [ ] Implement encryption awareness (respect `.gitehr/ENCRYPTED` marker).
- [ ] Add MCP configuration system (`.gitehr/mcp.json`).
- [ ] Integrate MCP with clinical calculators crate.
- [ ] Add GUI MCP client panel for LLM chat interface.
- [ ] Document MCP integration guide and API reference.
- [ ] Add MCP client libraries (Python/TypeScript) for testing.

## Documentation and Operations

- [x] Maintain MkDocs site scaffolding (`docs/`, `mkdocs.yml`).
- [ ] Keep command docs consistently aligned with runtime behavior.
- [ ] Expand user-facing docs (getting started, CLI reference, GUI walkthroughs, troubleshooting).
- [ ] Document packaging strategy for CLI+GUI distribution and upgrade/migration compatibility.
- [ ] Add calculator usage guide with clinical examples and validation references.
- [ ] Add MCP integration guide for LLM application developers.
- [ ] Document long-term strategic considerations (EHDS, EHRxF, quantum crypto, federated learning).
