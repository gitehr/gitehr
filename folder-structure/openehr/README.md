# openEHR

This directory holds native openEHR content: operational templates, versioned
reference-model instances, and optional derived indexes. It is laid out for
future openEHR support (roadmap R10/R17-R22); today it is structure and
documentation only - nothing in GitEHR currently reads or writes here.

Layout (see `spec/openehr.md`):

- `templates/` - Operational Templates (`.opt`, XML/JSON) that define the
  structure a COMPOSITION must follow. Templates are registered artefacts with
  their own identity and version.
- `instances/COMPOSITION/` - versioned COMPOSITION instances (canonical JSON),
  one immutable version per file, named by object id.
- `instances/EHR/` - EHR-level instances.
- `indexes/` - optional, always-regenerable query indexes (e.g. SQLite).
  Indexes are derived views; deleting and rebuilding them must never lose
  clinical data - the instances and templates are the custody layer.

Conventions (from ADR-0002, record only grows):

- Instances are immutable and versioned; a correction is a new version, never
  an in-place edit.
- openEHR CONTRIBUTION semantics (an audited set of versions committed
  together) map naturally onto a git commit; when openEHR support lands
  (R17+), instance changes will be journal-referenced like Documents.
- Nothing in this directory is clinically active until the openEHR commands
  (R14, R18) exist; it is scaffolded now so the layout and its conventions are
  settled before data exists.

Examples of content:

- Operational templates for encounter, observation, and medication orders
- COMPOSITION instances exported from or destined for an openEHR CDR
- Regenerable AQL indexes over local instances