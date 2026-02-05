<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# FHIR v5 Support in GitEHR

## Goal

Enable GitEHR to store and validate FHIR v5 resources alongside openEHR and journal/state files, preserving GitEHR’s append-only audit model.

## Scope

- **FHIR v5** resources (JSON) with conformance checks against official StructureDefinitions and value sets.
- **Rust reference models** for FHIR, integrated with GitEHR file layout and journal workflow.

## Sources of Truth

- Use the **official FHIR v5 definitions package** (JSON) as the canonical source for StructureDefinitions, ValueSets, and CodeSystems.
- Prefer a **pinned release** of the FHIR definitions to ensure stable validation.

## Repository Layout

Introduce new top-level directories (under GitEHR repo root):

```
/fhir/
  /definitions/           # pinned FHIR v5 package (StructureDefinition, ValueSet, CodeSystem)
  /resources/             # concrete FHIR resource instances (JSON)
  /indexes/               # optional derived indexes (search/cache)
```

Journal entries should reference these resources via file links (e.g., `links.related_files` or new structured link fields in YAML front matter).

## GitEHR Integration Model

### Journal

- **Journal entries remain the audit log**.
- Each import/update of FHIR data produces a journal entry referencing the updated resource files.
- The journal entry captures:
  - Source (FHIR)
  - Resource identifiers
  - Validation result summary
  - Any mapping/transformation metadata

### State

- `state/` can keep denormalized “current state” derived from FHIR instances, while the source-of-truth remains in `/fhir/resources`.

## Rust Implementation Plan

### 1) Ingest & Storage

- Store canonical JSON resources under `/fhir/resources/<ResourceType>/<id>.json`.
- Store provenance in journal entry front matter or sidecar metadata files.

### 2) Reference Models (Rust)

- `src/fhir/`:
  - Load FHIR definitions from `/fhir/definitions`.
  - JSON parsing for resources (serde + JSON schema-like validation using definitions).
  - Validation APIs: `validate_resource(resource_json, definitions)`.

### 3) CLI Commands (future)

- `gitehr fhir import <file>` → validate + store + journal entry.
- `gitehr fhir validate <file>` → report validation.

### 4) Indexing & Search

- Optional generated indexes under `/fhir/indexes`.
- Keep these derived files out of journal logic (they can be rebuilt).

## Validation Strategy

- Validate resource JSON against StructureDefinitions and ValueSets.
- Support versioned packages so validation is deterministic.

## Implementation Steps

1. **Add new spec sections** for `/fhir` directory and its lifecycle.
2. **Create download tooling** (CLI or scripts) to download pinned FHIR v5 definitions into `/fhir/definitions`.
3. **Build Rust modules** for FHIR definition loading + resource validation.
4. **Add CLI import/validate commands** with journal integration.
5. **Update journal schema** to include structured references to FHIR resources.
6. **Add tests** with sample FHIR payloads.
7. **Document** expected workflows and storage layout.

## Risks / Open Questions

- FHIR validation performance and version pinning strategy.
- Long-term maintenance of FHIR definitions.

## Deliverables

- Spec updates describing `/fhir` storage.
- CLI tooling for ingest/validate.
- Rust validation modules.
- Journal integration for auditability.
- Tests and documentation.

---

This document describes the requirements and plan for FHIR v5 support in GitEHR. For openEHR integration, see `openehr.md`.
