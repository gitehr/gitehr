# Medical Markdown structured data is derived from the journal body, never separately stored

GitEHR lets clinicians write a single free-text journal body using [Medical Markdown](https://github.com/pacharanero/medical-markdown) `CODE/` shorthand instead of filling in structured-data forms. We decided that the Markdown body remains the single source of truth, and any structured representation of it is computed on demand as a pure function of that body (`medical_markdown::extract_structured_data`), never authored or stored as a separate, independently-editable artifact. The on-disk journal entry format is unchanged: no new front matter, no sidecar structured file. See [the integration design](../medical-markdown.md) for how this is surfaced through the CLI, MCP server, and State files.

## Considered Options

- **Store extracted structured data in the entry's front matter or a sidecar file at commit time**: makes structured queries cheap to read, but creates two representations of the same clinical facts that can disagree. An edit to the narrative would silently invalidate the stored structure, and a stored projection is exactly the kind of duplicated truth that ADR-0001 rejects for document reverse-lookups. The narrative and its structure must never be able to drift. Rejected.
- **Replace the free-text body with structured fields (forms)**: this is the data-entry model clinicians reject, and it fragments the clinical narrative. The whole point of Medical Markdown is to keep one human-readable narrative and derive structure from it. Rejected.
- **Body is canonical, structure is derived on demand** (chosen): extraction is reproducible from any clone at any commit with no index to maintain, historic and non-coded entries remain valid, and immutable entries yield stable, disposable, cacheable structure.

## Consequences

Reading structured data always means parsing the body, so any structured view (CLI `--structured`, the MCP `extract_structured_data` tool, GUI rendering) calls the parser rather than reading a stored field. Promoting extracted items into State files (problem list, medications) is a separate, explicit, provenance-bearing step rather than an automatic write, preserving the immutability and auditability established in ADR-0002. The clinical vocabulary a record was written against is itself committed to the repository (a registry file) so historic entries stay interpretable. The journal entry remains the immutable evidence; everything structured is a recomputable projection of it.
