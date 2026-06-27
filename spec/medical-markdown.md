# Medical Markdown in GitEHR

## Why

Clinicians hate structured-data forms. They are slow, they fragment the clinical narrative across dozens of fields, and they force the author to think in the database's shape rather than the patient's story. But downstream systems (problem lists, medication reconciliation, analytics, decision support, billing) need structured, coded data. GitEHR resolves this tension with [Medical Markdown](https://github.com/pacharanero/medical-markdown): clinicians write a single free-text field using lightweight `CODE/` shorthand (`PC/ chest pain`, `HPC/ started 2 hours ago`, `IMP/ possible ACS`), which stays fully human-readable, and structured clinical data is *extracted* from it on demand rather than typed into forms.

This fits GitEHR's existing grain. The journal body is already a Markdown narrative ([`cli/src/commands/journal/mod.rs:84-89`](../cli/src/commands/journal/mod.rs)), currently treated as opaque text. Medical Markdown adds a parseable layer on top of that text without changing what the clinician sees or how the file is stored. It is also a strong fit for LLM workflows: an LLM can convert dictation into Medical Markdown, and Medical Markdown extracts into structured data far more reliably than parsing arbitrary prose.

Medical Markdown lives in its own repository under the MIT licence (rationale: a reusable format/parser must be permissively licensed to be adopted, whereas GitEHR itself is AGPL-3.0). GitEHR consumes it as an ordinary crate dependency and vendors none of its code.

## Core principle: the body is canonical, structured data is derived

The Markdown body of a journal entry remains the single source of truth. Structured data is always a pure function of that body, computed by `medical_markdown::extract_structured_data`, never a separately-authored artifact that can drift from the narrative. This is the load-bearing decision and is recorded in [ADR-0004](adr/0004-medical-markdown-structured-data-is-derived.md).

Consequences of this principle:

- A journal entry's on-disk format does not change. No new front matter, no sidecar structured file stored next to the entry. Entries written before Medical Markdown existed, and entries that contain no `CODE/` lines at all, remain valid and simply extract to an empty structure.
- Structured extraction is reproducible from any clone of the repository at any commit, with no index to rebuild and no truth to duplicate. This mirrors the reasoning in [ADR-0001](adr/0001-documents-as-plain-files.md) (reverse lookups are derived, never stored) and [ADR-0002](adr/0002-record-only-grows.md) (the record only grows).
- Because immutable entries never change after commit, their extracted structure is stable and cacheable, but the cache is always disposable.

## What the crate provides

The `medical-markdown` crate is a [`markdown-it`](https://crates.io/crates/markdown-it) plugin. The pieces GitEHR uses:

- `medical_markdown::add(md)` / `add_with_registry(md, registry)` - register the block rule that parses `CODE/ notes` and nested indented sub-codes into AST nodes.
- `medical_markdown::extract_structured_data(&ast) -> serde_json::Value` - walk the parsed AST and produce ordered JSON keyed by clinical code, with a `_source_map` of line numbers. Example output for `PC/ chest pain` followed by an `OE/` block with `RS/` and `CVS/` sub-codes:

  ```json
  {
    "PC": { "notes": "chest pain" },
    "OE": { "notes": "alert", "RS": "clear", "CVS": "no murmurs" },
    "_source_map": { "PC": 1, "OE": 2, "OE.RS": 3, "OE.CVS": 4 }
  }
  ```

- `medical_markdown::CodeRegistry` - the clinical vocabulary. `CodeRegistry::default()` gives the 34 built-in codes; `from_json` loads custom codes; `merge` overlays one registry on another. This is how a GitEHR deployment extends or localises the vocabulary.
- Semantic HTML rendering (`<section class="med-section med-pc" data-med-code="PC">`) via the AST's `render()`, for the GUI and any HTML-facing surface.

## Integration points

### A single wrapper module

All Medical Markdown access in GitEHR goes through one thin module (proposed `cli/src/medmd.rs`) so the rest of the codebase never builds a `MarkdownIt` instance directly. It exposes roughly:

```rust
pub fn extract(body: &str) -> serde_json::Value;      // structured data from a journal body
pub fn has_codes(body: &str) -> bool;                  // cheap check: any CODE/ lines present?
pub fn render_html(body: &str) -> String;             // semantic HTML for display
```

Each builds a parser, registers `cmark` + `medical_markdown` (with the active registry, see below), parses `body`, and returns the result. The body passed in is `ParsedEntry.content` ([`cli/src/commands/journal/mod.rs:84-89`](../cli/src/commands/journal/mod.rs)), i.e. the text after the YAML front matter is split off in `parse_journal_file`.

### CLI: surfacing the structure

- `gitehr journal show <entry> --structured` (or `--json`) prints the extracted JSON alongside, or instead of, the rendered narrative. Read-only, derived, safe on historic entries.
- `gitehr journal extract <entry>` as an explicit extraction command for piping into other tools.
- The editor flow (`journal add with no arguments`, [`cli/src/commands/journal/add.rs`](../cli/src/commands/journal/add.rs)) is unchanged; clinicians still write free text in their editor. A later phase can add a `--validate` pass on commit that warns about unrecognised codes without blocking the commit (the record still only grows; a warning is advisory).

### MCP: structured access for agents

The MCP server already plans an `extract_structured_data` tool ([`spec/mcp.md`](mcp.md)) and currently returns journal entries as raw `text/markdown` ([`cli/src/commands/mcp/server_impl/resources.rs`](../cli/src/commands/mcp/server_impl/resources.rs)). Medical Markdown backs both:

- Implement the `extract_structured_data` tool over `medmd::extract`, so an agent can ask for any entry (or arbitrary Medical Markdown text) as structured JSON.
- Offer a structured variant of the journal-entry resource (e.g. a `format=structured` parameter) that returns the extracted JSON with its `_source_map`, letting agents reason over coded sections while keeping the default `text/markdown` view for humans.
- The `add_journal_entry` tool (currently a placeholder, [`cli/src/commands/mcp/server_impl/tools.rs`](../cli/src/commands/mcp/server_impl/tools.rs)) can echo the extracted structure back as confirmation of what was understood, which is exactly the feedback loop an LLM authoring entries wants.

### Repo-level code registry

A deployment extends the vocabulary by committing a registry file (proposed `.gitehr/medmd-codes.json`, matching `CodeRegistry::from_json`'s schema of `{code, heading, category}` objects). The wrapper module loads `CodeRegistry::default()`, merges the repo file if present, and uses that everywhere. Because the registry lives in the repository, the codes a record was written against travel with the record - essential for interpreting historic entries correctly.

### State projection: the payoff

GitEHR's State files (`state/problems.md`, `state/medications.md`, etc. per [`spec/commands/state.md`](commands/state.md) and [`spec/DESIGN.md`](DESIGN.md)) are the longitudinal, current-view summaries of the patient, stored in the same YAML-front-matter-plus-Markdown format as journal entries. Medical Markdown closes the loop from narrative to summary: extracting `IMP/` sections feeds the problem list, `RX/` (medication syntax, on the Medical Markdown roadmap) feeds the medication list, and so on.

This is deliberately a *separate, explicit* step, not an automatic write. The clinician (or an agent under review) promotes extracted items into State; the journal entry remains the immutable evidence and the State file records provenance back to the entry. This keeps extraction non-destructive and auditable, and avoids silently mutating the current view from free text. Full realisation depends on Medical Markdown gaining medication, vitals, and SNOMED-annotation syntax (see "What Medical Markdown needs", below).

## Dependency wiring

During co-development, before Medical Markdown is published to crates.io, GitEHR uses a path dependency in `Cargo.toml` workspace dependencies:

```toml
[workspace.dependencies]
medical-markdown = { path = "../../medical-markdown" }
```

For released builds this flips to a normal version dependency (`medical-markdown = "0.x"`). A `[patch.crates-io]` override can pin to the local checkout when both repositories are moving together. Only the `cli` crate (and later `mcp`) need the dependency; it is not added to crates that do not parse journal bodies.

## Phasing

1. **Read-only extraction.** Wrapper module, `journal show --structured`, `journal extract`, MCP `extract_structured_data` tool and structured resource variant. No format change, no writes. Uses the 34 built-in codes.
2. **Vocabulary and validation.** Repo-level registry file, advisory commit-time validation of unknown codes, GUI HTML rendering of coded sections.
3. **State projection.** Promote extracted `IMP/`/`RX/`/etc. into State files with provenance. Depends on Medical Markdown medication/vitals/SNOMED syntax.
4. **Coded terminology.** SNOMED-CT annotation in Medical Markdown surfaced into GitEHR's planned terminology integration ([`spec/fhir.md`](fhir.md), [`spec/openehr.md`](openehr.md)), enabling FHIR/openEHR export from extracted structure.

## What Medical Markdown needs

Embedding surfaced several gaps and ergonomics issues in the crate. These are written up for the Medical Markdown maintainers in [that repository's `spec.md`](https://github.com/pacharanero/medical-markdown/blob/main/spec.md). In summary: a one-call library entry point and registry loading from a string (not only a file path), typed (not only `serde_json::Value`) output, a documented and versioned extraction schema, full source spans for editor/diff use, a `validate` function for unknown-code diagnostics, and prioritisation of `RX/` medication and SNOMED-annotation syntax, which the State-projection phase depends on.
