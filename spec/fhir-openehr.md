<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# FHIR + openEHR Integration in GitEHR

This document provides a high-level integration overview for supporting both FHIR v5 and openEHR native entities in GitEHR.

- For FHIR-specific requirements and implementation, see [fhir.md](fhir.md).
- For openEHR-specific requirements and implementation, see [openehr.md](openehr.md).

## Integration Model

- Journal entries remain the audit log, referencing FHIR/openEHR resource files and validation results.
- `state/` can keep denormalized “current state” derived from FHIR/openEHR instances, while the source-of-truth remains in `/fhir/resources` and `/openehr/instances`.

## Repository Layout

See [fhir.md](fhir.md) and [openehr.md](openehr.md) for directory structure and storage details. Journal entries should reference these resources via file links (e.g., `links.related_files` or new structured link fields in YAML front matter).

## Implementation Overview

- Ingest, validate, and store FHIR and openEHR resources natively.
- Rust modules for FHIR and openEHR reference models, validation, and serialization.
- CLI commands for import/validate (see [fhir.md](fhir.md) and [openehr.md](openehr.md)).
- Optional generated indexes for search/caching.

## Validation Strategy

- FHIR: Validate resource JSON against StructureDefinitions and ValueSets.
- openEHR: Validate composition JSON against operational template constraints.

## Implementation Steps

See [fhir.md](fhir.md) and [openehr.md](openehr.md) for detailed implementation steps for each standard.

## Risks / Open Questions

See [fhir.md](fhir.md) and [openehr.md](openehr.md) for standard-specific risks and open questions.

## Deliverables

- Spec updates describing `/fhir` and `/openehr` storage and integration.
- CLI tooling, validation modules, journal integration, tests, and documentation (see standard-specific docs).

---

For full technical plans, see [fhir.md](fhir.md) and [openehr.md](openehr.md).
