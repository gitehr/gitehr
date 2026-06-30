<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr vaccinations`

Status: implemented v1.

`gitehr vaccinations` manages typed vaccination/immunisation state in
`state/vaccinations.md`. It follows the same pattern as `gitehr allergies` and
`gitehr demographics`: the state file is the current queryable view; the journal
entry written with each mutation is the audit narrative.

The command also has spelling aliases:

- `gitehr immunisations`
- `gitehr immunizations`

## Storage

`state/vaccinations.md` is YAML front matter with a top-level `vaccinations`
array. Each entry is one administered vaccination or immunisation.

Minimum useful record:

```yaml
---
vaccinations:
  - id: VAC-20260630T120000Z-4f2a9c1b
    status: completed
    vaccine: MMR
    date: 2026-06-30
    dose_sequence: 1
    target_disease:
      - measles
      - mumps
      - rubella
    anatomical_site: left deltoid
    route: intramuscular
    product: Priorix
    manufacturer: GSK
    batch_number: ABC123
    performer: Nurse Example
    recorded_at: 2026-06-30T12:00:00Z
    recorded_by: nurse-example
    entered_in_error_at: null
    entered_in_error_reason: null
    note: Given at travel clinic.
    fhir_r4: null
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `VAC-<timestamp>-<random>`.
- `status` - `completed` or `entered-in-error`.
- `vaccine` - human-readable vaccine/immunisation display text.
- `date` - administration date, `YYYY-MM-DD`.
- `dose_sequence` - optional sequence number, for example first, second, booster.
- `target_disease` - repeatable display text for protected diseases.
- `anatomical_site` - administration site, for example `left deltoid`.
- `route` - administration route.
- `product` - exact product administered.
- `manufacturer` - product manufacturer.
- `batch_number` - batch or lot number.
- `performer` - person or organisation administering.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `entered_in_error_at` / `entered_in_error_reason` - correction metadata.
- `note` - optional clinical note.
- `fhir_r4` - optional embedded FHIR R4 `Immunization` JSON object.

## FHIR R4 / NHS FHIR

The v1 model intentionally keeps a readable GitEHR-native row and optionally
embeds the source FHIR R4 `Immunization` resource. This lets GitEHR preserve
NHS FHIR detail before a full export/import mapper exists.

Expected mapping:

| GitEHR field | FHIR R4 `Immunization` |
|---|---|
| `status` | `status` |
| `vaccine` / `product` | `vaccineCode` |
| `date` | `occurrenceDateTime` or `occurrenceString` |
| `dose_sequence` | `protocolApplied.doseNumber[x]` |
| `target_disease` | `protocolApplied.targetDisease` |
| `anatomical_site` | `site` |
| `route` | `route` |
| `manufacturer` | `manufacturer` |
| `batch_number` | `lotNumber` |
| `performer` | `performer.actor` |

The embedded `fhir_r4` object must have `resourceType: "Immunization"`.

## Commands

```bash
gitehr vaccinations list [--json] [--all]
gitehr vaccinations add --vaccine <name> --date <YYYY-MM-DD> [OPTIONS]
gitehr vaccinations entered-in-error <id> [--reason <text>]
```

`list` hides `entered-in-error` entries by default. `--all` includes them.

`add` writes `state/vaccinations.md`, stages it, writes a journal entry, and
commits both together.

`entered-in-error` never deletes a vaccination entry. It changes the current
state row and writes a journal entry, preserving prior belief in Git history.
