# GitEHR Roadmap (Spec-Aligned)

This roadmap tracks implementation status against the current `spec/` documents.

## Core CLI and Repository Lifecycle

- [x] Implement binary bundling in `gitehr store init` (`.gitehr/gitehr`).
- [x] Implement finalized journal file layout (`journal/<timestamp>-<uuid>.md`).
- [x] Implement journal YAML front matter fields: `parent_hash`, `parent_entry`, `timestamp`, optional `author`.
- [x] Implement `gitehr journal add` input modes: inline, `--file <path>`, and stdin via `--file -`.
- [x] Implement `gitehr journal show` with pagination options.
- [x] Implement `gitehr journal verify` hash-chain validation.
- [x] Implement contributor activation so journal entries include current `author`.
- [x] Implement `gitehr version` output with both GitEHR and Git versions.
- [x] Implement shell completions generation (`gitehr completions <shell>`).
- [ ] **Store-first bootstrap ([ADR-0005](adr/0005-store-first-model.md)):** make `gitehr store init` create the Store, the MPI, and the first subject repo in one step (reusing the repo-scaffolding from the old `gitehr store init`), and **remove the top-level `gitehr store init`**. Subject repos use the UUIDv7 + Crockford directory naming from the spec.
- [ ] Add robust `gitehr journal verify --verbose` (or equivalent) failure diagnostics per spec TODO.

## Plugin System

Git-style `$PATH` extensibility (see [`spec/commands/plugin.md`](commands/plugin.md), `cli/src/commands/plugin.rs`).

- [x] Plugin discovery: scan `$PATH` for `gitehr-<command>` executables (`gitehr plugins`).
- [x] Resolution order, built-in commands first: implemented via clap `#[command(external_subcommand)]`, so a plugin can never shadow a built-in (security property, with plugin-name validation against path traversal).
- [x] Argument pass-through: `gitehr foo arg1 arg2` execs `gitehr-foo arg1 arg2` (on Unix the plugin replaces the process, so stdio/signals/exit code pass through).
- [x] `gitehr plugins` command listing installed plugins (built-in-shadowed names excluded).
- [x] `gitehr --help` shows discovered plugins alongside built-in commands (dynamic `after_help`).
- [x] Plugin authoring guidelines documented in `spec/commands/plugin.md` and `docs/cli/plugins.md`.
- [x] Plugin example documented (a `gitehr-hello` sample).

## Import and Document Capture

Bringing existing records into a repository (see [`spec/commands/import.md`](commands/import.md), [`docs/cli/import.md`](../docs/cli/import.md), `cli/src/commands/import.rs`).

- [x] Implement `gitehr import` (`--mode journal | documents`): import well-formed journal entries verbatim (filename/UUID/author/provenance preserved, non-entries skipped, already-present skipped so it is idempotent, each entry committed), or bulk-import documents of any format (each copied into `documents/` with a lightweight linking journal entry). Accepts a file or a directory, walked recursively with hidden files skipped, and reports a summary count.
- [ ] **Built-in OCR for imported documents (eventually).** Make it as trivially easy as possible for less-technical patients using the GUI to bring their own medical records together. When importing a scan or photo via `--mode documents`, run OCR so the journal entry carries searchable, machine-readable text alongside the original file, not just a link - a patient should be able to drop in a photo of a letter and get a real, searchable record entry with near-zero friction. Keep it **built-in and offline** (no shipping medical images to a cloud OCR service), and treat the OCR text as a derived convenience layer, never a replacement for the original document.
- [ ] Add further import modes as the need arises (e.g. an imaging-scanned mode), per the "other modes later" note in `spec/commands/import.md`.
- [ ] Once a config file exists, let `--mode documents` filter against a configured file-format whitelist (per the TODO in `spec/commands/import.md`).

## Command Coverage vs Spec

- [x] Implement `gitehr state` (`list`, `get`, `set`).
- [x] Implement `gitehr remote` (`add`, `remove`/`rm`, `list`).
- [x] Implement `gitehr encrypt` placeholder marker flow.
- [x] Implement `gitehr decrypt` placeholder marker removal flow.
- [x] Implement `gitehr status` summary output.
- [x] Implement `gitehr transport` (`create`, `extract`) - "transport mode" bundling.
- [x] Implement `gitehr calc` clinical calculators (forwarding to `calc_cli::run`; the engine lives in the pacharanero/calc repo).
- [x] Implement `gitehr journal` `new-entry`/`commit`/`list`/`show`. Note: the journal is append-only - drafts (`new-entry`) can be edited or discarded before commit, but committed entries are immutable, so there is no `journal edit`/`journal delete` of committed entries by design.
- [ ] Add `gitehr export` - generate standardised export bundles (FHIR / EHRxF / openEHR) from a repository for cross-border sharing and portability (see `spec/fhir-openehr.md` and the EHDS/EHRxF notes in `spec/long-term-ideas.md`).
- [x] Implement `gitehr user` (`create`, `add`, `enable`, `disable`, `activate`, `deactivate`, `list`) and `contributor` alias.
- [x] Implement `gitehr upgrade`.
- [x] Implement `gitehr upgrade-binary`.
- [x] `gitehr store` ([ADR-0005](adr/0005-store-first-model.md)): `init [name]` bootstraps the Store + MPI + first subject repo; `add [name]` creates and registers a new subject repo; `remove <id|name>` de-registers; `list` shows subjects. Each subject gets a canonical UUIDv7 (Crockford base32) id and a friendly-name-or-id directory (`cli/src/commands/scaffold.rs`).
- [ ] **Store-first, remaining ([ADR-0005](adr/0005-store-first-model.md)):** Store/repo context detection (walk up for `.gitehr/` and `gitehr-mpi.json`) plus single-subject auto-targeting; then **remove the top-level `gitehr store init`** and move the test suite onto `gitehr store init`. The MPI identifier operations (`search`, `link`, `unlink`, `merge`, `path`) and the `GITEHR_MPI_PATH` override fold in as `gitehr store` subcommands later.
- [ ] **Self-hoster on-ramp docs (families and pets):** make the single-user, multi-subject story first-class on the site and in GUI onboarding - individuals and families keeping their own records, and **pet owners** keeping their animals' records - alongside the clinic story. Per ADR-0005 these are primary audiences, not afterthoughts.
- [ ] Align `gitehr gui` launcher with command spec (prefer bundled `.gitehr/gitehr-gui`, then PATH `gitehr-gui`; current implementation still launches dev command).

## Repository Template and Data Layout

- [x] Ensure init copies template directories from `folder-structure/`.
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

The calculators live in their own repository, **[pacharanero/calc](https://github.com/pacharanero/calc)** (`~/code/pacharanero/calc`), built and tested there. GitEHR consumes them as a dependency: `cli` depends on `calc-cli` (so `gitehr calc` forwards to `calc_cli::run`) and `mcp` depends on `calc-core` (so each calculator is exposed as a `calc_<name>` MCP tool). The architecture, roadmap, and input-definition design specs moved with them to that repo's `spec/`.

- [x] **The full 50-tool calculator library is complete in pacharanero/calc** - 42 implemented and verified against primary sources (including QRISK3 and QFracture, ported from ClinRisk's LGPL source and validated against its C reference), plus 8 proprietary/licence-locked tools shipped as protest stubs. Single-engine design: `calc-core` (serde-only leaf) drives the `calc` CLI, MCP, GUI, and web; every calculator records a `license()` and carries machine-readable input definitions.
- [x] `gitehr calc` subcommand and MCP `calc_<name>` tools wired to the external crates.
- [ ] Switch the `calc-cli`/`calc-core` dependencies from a sibling path to a git dep (then crates.io once pacharanero/calc has a distribution pipeline).
- [ ] Record calculator results in the journal (immutable entry: calculator, version, inputs, result, citation) - GitEHR-side integration.
- [ ] Add state file storage for latest results (`state/calculations/<name>-latest.json`) - GitEHR-side.
- [ ] Add a GUI calculator panel + Tauri `calculate_clinical` command calling `calc_core` natively.

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
- [x] Integrate MCP with clinical calculators: each `calc-core` calculator is exposed as a `calc_<name>` MCP tool whose `inputSchema` is the calculator's own JSON Schema; `tools/call` runs the shared engine and returns the `CalculationResponse`.
- [ ] Add GUI MCP client panel for LLM chat interface.
- [ ] Document MCP integration guide and API reference.
- [ ] Add MCP client libraries (Python/TypeScript) for testing.

## Security and Integrity (to review)

- [ ] **Evaluate [gittuf](https://gittuf.dev/) for GitEHR.** gittuf applies The Update Framework (TUF) concepts to a Git repository, adding security that Git itself lacks: policy-controlled, signed access to branches/tags/refs, key management and rotation, and protection against attacks on references (unauthorised ref updates, rollback/freeze, tag tampering). This is directly relevant to GitEHR's integrity, provenance, and tamper-evidence goals (who may update which refs in a patient repository, and proving a ref's history has not been rewritten). It is still in beta - the action is to keep an eye on it and review whether and how it can fit into GitEHR's security model once it matures.

## Documentation and Operations

- [x] Maintain Zensical site scaffolding (`docs/`, `mkdocs.yml`, `requirements.txt`).
- [ ] Restructure top-level nav to seven tabs: Home, Design, Install, CLI, GUI, TUI, Safety. Move existing content into the new sections; create stubs for sections that do not yet have content (TUI, Safety).
- [ ] Keep command docs consistently aligned with runtime behavior.
- [ ] Expand user-facing docs (Install, CLI reference, GUI walkthroughs, TUI overview once it exists, Safety / Turva, troubleshooting).
- [ ] Document packaging strategy for CLI+GUI distribution and upgrade/migration compatibility.
- [ ] Add calculator usage guide with clinical examples and validation references.
- [ ] Add MCP integration guide for LLM application developers.
- [ ] Document long-term strategic considerations (EHDS, EHRxF, quantum crypto, federated learning).

## Site Content (gitehr.org)

Source: `gitehr-site-improvement-handoff.md` at the repo root. Goal is to strengthen the "files on disk vs databases" argument that underpins GitEHR's design, by framing it as the consensus the rest of software has already reached rather than as a healthcare-specific opinion. Style: ASCII hyphen-minus only (no emdash), MkDocs-compatible admonitions (work in Zensical's classic variant), relative internal links.

### High priority

- [x] Replace the blockchain reference in `docs/design/design.md` ("Using the same consensus principle from blockchain technology...") with a content-addressed-hashing / Merkle DAG framing. Git is not a Nakamoto-consensus system. *(Done in `redundancy.md` and the duplicated content in `design.md`.)*
- [x] Add a new "Files, not databases" section to `docs/design/design.md` (or a new page under `docs/design/`) covering: the one-sentence pitch, why this is not radical (Git, lakehouse/Iceberg, file-over-app, DICOM, email), and why database-to-database interoperability does not work. *(New page `docs/design/files-not-databases.md` added to nav.)*
- [x] Promote the CVS/DVCS analogy from `docs/about/the-gitehr-story.md` to the homepage (`docs/index.md`) as a one-line elevator pitch. *(Now the lead of the hero subtitle.)*
- [x] Replace the Mars colony ship example with grounded real-world cases. *(Updated in `portability.md` and the duplicated content in `design.md`.)*

### Medium priority

- [ ] Add a "Common objections" or FAQ page covering: cross-patient queries for research and population health (org-level derived databases built from canonical files, mirroring Iceberg-over-Parquet); concurrent edits (Git branch-and-merge with clinical conflict resolution); ACID and consistency (per-file atomicity plus cryptographic chain-of-custody); GDPR right to erasure (the hardest one - needs careful framing given Git's immutable history).
- [ ] Cross-reference the wider movement with explicit links: Ink and Switch local-first paper (Kleppmann et al. 2019), Steph Ango's "File over app" essay, Pat Helland's "Immutability Changes Everything" (2015), Apache Iceberg, SQLite-as-archival-format. Add to a references section or inline citations.
- [ ] Expand the N-squared integration problem into its own paragraph plus a diagram: N organisations with their own databases produces N(N-1)/2 integration pairs; N organisations agreeing on a file format produces N implementations and zero pairs.
- [ ] Add a section (in `design/files-not-databases.md` or its own page) on the agentic coding angle: clinical LLM applications can read, diff, and answer questions over a Git history of markdown files in ways that map poorly to databases. Files give you `grep`, `diff`, `git log`, and a full audit trail in context; the structured-query advantage databases historically offered shrinks when an LLM can answer "what changed in this patient's medication list last month" without writing SQL.

### Lower priority / diagrams

- [ ] Commission or generate three diagrams: (a) N(N-1)/2 integration pairs vs N implementations of a shared format, (b) "patient as folder, organisations as clones" distributed clone topology, (c) optional lakehouse-style stack diagram with canonical files at the bottom, derived org-level databases in the middle, applications at the top.
- [x] Style sweep across all `docs/*.md`: remove emdash characters in favour of ASCII hyphen-minus, audit for blockchain references, audit for the Mars colony example. *(Initial pass: en-dashes converted in `glossary.md` and `developers.md`; one U+2011 fixed; no em-dashes found. Re-run after major content additions.)*
- [ ] Verify Zensical strict build (or enable equivalent) and ensure no broken internal links after content reshuffles.
