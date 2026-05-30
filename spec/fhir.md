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
  /definitions-fsh/       # FHIR Shorthand source (canonical, version controlled)
    ├── sushi-config.yaml
    └── input/fsh/*.fsh
  /definitions/           # Compiled FHIR definitions (derived from FSH)
    └── StructureDefinition-*.json
  /resources/             # Concrete FHIR resource instances (JSON)
  /indexes/               # Optional derived indexes (search/cache)
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

## Implementation Strategy: FHIR Shorthand (FSH)

### Why FSH?

Based on research of FHIR implementation approaches, **FHIR Shorthand (FSH)** is the recommended method for defining FHIR profiles, extensions, and value sets in GitEHR.

**FHIR Shorthand** is an official HL7 standard (Mixed Normative/Trial Use) that provides a domain-specific language for authoring FHIR Implementation Guide content.

**Key Advantages:**
- **10-20x more concise** than manual JSON StructureDefinitions
- **Text-based**: Perfect for Git version control (meaningful diffs, easy merging)
- **Production-ready**: Used by 600+ FHIR IG projects worldwide
- **Official tooling**: SUSHI compiler with 250,000+ npm downloads
- **FHIR R5 support**: Full coverage of FHIR v5 features
- **Community-backed**: HL7 standard with active development

**Example Comparison:**

FSH (12 lines):
```fsh
Profile: GitEHRObservation
Parent: Observation
Id: gitehr-observation
Title: "GitEHR Clinical Observation"
* performer 1..* MS
* effectiveDateTime 1..1 MS
* extension contains GitEHRJournalReference named journalRef 0..1 MS

Extension: GitEHRJournalReference
Id: gitehr-journal-reference
* value[x] only string
* valueString ^short = "Journal entry filename"
```

Equivalent JSON StructureDefinition: 200+ lines of nested objects.

### SUSHI Compiler

**SUSHI** (SUSHI Unshortens ShortHand Inputs) is the reference implementation FSH compiler:
- **Language**: TypeScript/JavaScript (Node.js)
- **License**: Apache 2.0 (compatible with AGPL-3.0)
- **Maturity**: Production-grade (4+ years, extensively tested)
- **Installation**: `npm install -g fsh-sushi`

### Integration with GitEHR

```
┌─────────────────────────────────────────┐
│ GitEHR Repository                       │
│                                         │
│  /fhir/                                 │
│  ├── definitions-fsh/  (FSH source)    │
│  │   ├── sushi-config.yaml             │
│  │   └── input/fsh/                    │
│  │       ├── profiles.fsh              │
│  │       ├── extensions.fsh            │
│  │       └── valuesets.fsh             │
│  ├── definitions/  (compiled JSON)     │
│  │   └── StructureDefinition-*.json    │
│  └── resources/  (FHIR instances)      │
│      └── Patient/patient-001.json      │
└─────────────────────────────────────────┘
         │
         │ gitehr fhir compile
         ↓
    ┌──────────┐
    │  SUSHI   │  (Node.js)
    └──────────┘
```

### CLI Commands

```bash
# Initialize FSH project in repository
gitehr fhir init

# Compile FSH to JSON
gitehr fhir compile
# (Runs: sushi build fhir/definitions-fsh --out fhir/definitions)

# Validate FHIR resource
gitehr fhir validate <resource.json>

# Import and store FHIR resource
gitehr fhir import <resource.json>
```

### Example GitEHR FSH Profiles

**Patient Profile with Repository Reference:**
```fsh
Profile: GitEHRPatient
Parent: Patient
Id: gitehr-patient
Title: "GitEHR Patient Profile"

* identifier 1..* MS
* extension contains GitEHRRepositoryId named repoId 1..1 MS
* active 1..1 MS
* name 1..* MS
* gender 1..1 MS
* birthDate 1..1 MS

Extension: GitEHRRepositoryId
Id: gitehr-repository-id
Title: "GitEHR Repository ID"
* value[x] only string
* valueString ^short = "UUIDv7 identifier for this repository"
```

**Provenance Profile for Journal Linkage:**
```fsh
Profile: GitEHRProvenance
Parent: Provenance
Id: gitehr-provenance
Title: "GitEHR Journal Entry Provenance"

* target 1..* MS
* recorded 1..1 MS
* agent 1..* MS
* extension contains GitEHRJournalEntry named journalEntry 1..1 MS

Extension: GitEHRJournalEntry
Id: gitehr-journal-entry
* value[x] only string
* valueString ^short = "Journal filename (2026-03-06T10:30:00.000Z-abc123.md)"
```

### Implementation Steps

1. **Add SUSHI dependency**:
   - Document Node.js requirement in developer setup
   - Add SUSHI to PATH during development
   - Consider bundling SUSHI in release artifacts

2. **Create FSH project structure**:
   - Add `/fhir/definitions-fsh/` to repository template
   - Initialize with `sushi-config.yaml`
   - Create initial profiles for core resources

3. **Implement CLI integration**:
   ```rust
   // cli/src/commands/fhir.rs
   pub fn compile_fsh(fsh_dir: &Path, output_dir: &Path) -> Result<()> {
       let status = Command::new("sushi")
           .args(["build", fsh_dir.to_str().unwrap()])
           .args(["--out", output_dir.to_str().unwrap()])
           .status()?;
       
       if !status.success() {
           anyhow::bail!("FSH compilation failed");
       }
       Ok(())
   }
   ```

4. **Add validation pipeline**:
   - Load compiled StructureDefinitions from `/fhir/definitions/`
   - Use Rust FHIR libraries (`fhir-sdk`) for runtime validation
   - Implement resource validation against profiles

5. **Journal integration**:
   - Create FSH extensions for journal references
   - Link FHIR resources to journal entries via Provenance
   - Update journal YAML to reference FHIR resources

6. **Git workflow**:
   - FSH source files are canonical (tracked in Git)
   - Compiled JSON may be committed or gitignored
   - Pre-commit hook to ensure FSH compiles cleanly

### Dependencies

**Development:**
- Node.js 14+ (for SUSHI)
- SUSHI: `npm install -g fsh-sushi`

**Runtime:**
- `fhir-sdk` (Rust) - FHIR resource handling
- `serde_json` - JSON serialization

### Resources

- **FSH Specification**: https://build.fhir.org/ig/HL7/fhir-shorthand/
- **SUSHI Docs**: https://fshschool.org/docs/sushi/
- **FSH School**: https://fshschool.org/ (tutorials, playground)
- **VS Code Extension**: `MITRE-Health.vscode-language-fsh`

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
