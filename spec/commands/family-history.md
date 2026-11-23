<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr family-history`

Status: implemented v1.

`gitehr family-history` manages typed family-history state in
`state/family-history.md`. It follows the same pattern as `gitehr allergies` and
`gitehr vaccinations`: the state file is the current queryable view; the journal
entry written with each mutation is the audit narrative.

Family history is a direct input to risk calculators such as QRISK3 (a
first-degree relative with angina or heart attack before 60) and FRAX
(parental hip fracture), so the onset age is recorded as a number rather than
free text.

## Storage

`state/family-history.md` is YAML front matter with a top-level
`family_history` array. Each entry is one condition in one relative: a relative
with two conditions is recorded as two entries.

Minimum useful record:

```yaml
---
family_history:
  - id: FH-20261123T120000Z-4f2a9c1b
    status: completed
    relationship: father
    condition: Myocardial infarction
    onset_age: 54
    contributed_to_death: false
    recorded_at: 2026-11-23T12:00:00Z
    recorded_by: gp-example
    entered_in_error_at: null
    entered_in_error_reason: null
    note: null
    fhir_r4: null
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `FH-<timestamp>-<random>`.
- `status` - `completed` or `entered-in-error`.
- `relationship` - relationship to the patient as display text, for example
  `mother`, `brother`, `maternal grandmother`.
- `condition` - condition display text.
- `onset_age` - optional relative's age in whole years when the condition began,
  `0`-`130` (larger values are rejected as a likely year of birth).
- `contributed_to_death` - `true` when the condition contributed to the
  relative's death; defaults to `false`.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `entered_in_error_at` / `entered_in_error_reason` - correction metadata.
- `note` - optional clinical note.
- `fhir_r4` - optional embedded FHIR R4 `FamilyMemberHistory` JSON object.
- `provenance` - optional source metadata: `source_type`, `source_detail`,
  `acquired_via`, `document_ref`, `evidence_level`, and `confidence`.
  `source_type` is required whenever any other provenance field is present. See
  [`record-provenance-and-acquisition.md`](../record-provenance-and-acquisition.md).

The relative's name and other identifying details are deliberately not modelled:
they are third-party personal data and are not needed for clinical use or risk
calculation.

## FHIR R4 / NHS FHIR

Each row maps to one FHIR R4 `FamilyMemberHistory` resource with a single
`condition` element. The source resource can be embedded in `fhir_r4` to
preserve detail before a full export/import mapper exists.

| GitEHR field | FHIR R4 `FamilyMemberHistory` |
|---|---|
| `status` | `status` |
| `relationship` | `relationship` (v3 RoleCode, as `text` until coded) |
| `condition` | `condition.code` |
| `onset_age` | `condition.onsetAge` (UCUM `a`) |
| `contributed_to_death` | `condition.contributedToDeath` |
| `note` | `note` |

The embedded `fhir_r4` object must have `resourceType: "FamilyMemberHistory"`.

## Commands

```bash
gitehr family-history list [--json] [--all]
gitehr family-history add --relationship <text> --condition <text> [OPTIONS]
gitehr family-history entered-in-error <id> [--reason <text>]
```

`add` also accepts `--onset-age <years>`, `--contributed-to-death`,
`--fhir-json <path>`, and `--note <text>`. The provenance flags are
`--source-type`, `--source-detail`, `--acquired-via`, `--document-ref` (an
existing file under `documents/` or `imaging/`; its path and SHA-256 are
recorded), `--evidence-level`, and `--confidence`; `--source-type` is required
when supplying any of the other five.

`list` hides `entered-in-error` entries by default. `--all` includes them.

`add` writes `state/family-history.md` and a journal entry and commits both
together. Mutation commits contain only the state file and the generated
journal entry, leaving unrelated staged work untouched. A mutation refuses a
dirty state file (an untracked, unmodified scaffold template is accepted) and
restores the prior file and index state if writing or committing fails.

`entered-in-error` never deletes an entry. It changes the current state row and
writes a journal entry, preserving prior belief in Git history.

All subcommands run against the subject repository resolved from the current
directory, like the other typed-state commands. A relative `--fhir-json` path is
resolved against the invocation directory.
