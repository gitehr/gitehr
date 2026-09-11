# FHIR

This directory holds FHIR content: compiled resource definitions and
concrete resource instances, plus optional derived indexes. It is laid out
for future FHIR support (roadmap R9/R11-R16); today it is structure and
documentation only - nothing in GitEHR currently reads or writes here.

More than one FHIR release is expected here. `spec/fhir.md` targets R5
("v5") for validation, while NHS App import (roadmap R4) plans to land R4
resources in this same layout, and `gitehr vaccinations` already embeds
source R4 `Immunization` JSON in typed State. A resource instance is only
interpretable alongside the release it conforms to, so record that release
explicitly - in the resource itself (`meta.profile`) or in a manifest beside
it - rather than inferring it from the directory.

Layout (see `spec/fhir.md`):

- `definitions/` - compiled FHIR definitions (StructureDefinitions, ValueSets,
  CodeSystems). These are **derived**: whether they come from a pinned
  official FHIR definitions release or are compiled from FHIR Shorthand (FSH)
  source is not yet decided (roadmap R11); nothing here is a canonical source
  to edit by hand.
- `resources/` - concrete FHIR resource instances (canonical JSON), the
  clinical data itself.
- `indexes/` - optional, always-regenerable query indexes (e.g. search or
  cache indexes). Indexes are derived views; deleting and rebuilding them must
  never lose clinical data - the resources are the custody layer.

Conventions (from ADR-0002, record only grows):

- Resource instances are immutable and versioned; a correction is a new
  version, never an in-place edit.
- Definitions are pinned/versioned rather than mutated in place, once R11
  decides their source.
- When FHIR support lands (R13+), resource changes will be
  journal-referenced like Documents.
- Nothing in this directory is clinically active until the FHIR commands
  (R12-R14) exist; it is scaffolded now so the layout and its conventions are
  settled before data exists.

Examples of content:

- Compiled StructureDefinitions and ValueSets for the resource profiles in use
- FHIR resource instances (e.g. Patient, Condition, Observation) imported from
  or exported to an external FHIR server
- Regenerable search indexes over local resource instances
