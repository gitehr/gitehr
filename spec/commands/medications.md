<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr medications`

Status: implemented v1 (medications only; immunisations and family history
remain outstanding, see [issue #84](https://github.com/gitehr/gitehr/issues/84)).

`gitehr medications` manages typed medication state in `state/medications.md`.
It follows the same pattern as `gitehr allergies` and `gitehr vaccinations`:
the state file is the current queryable view; the journal entry written with
each mutation is the audit narrative.

## Storage

`state/medications.md` is YAML front matter with a top-level `medications`
array. Each entry is one medication or supplement.

Minimum useful record:

```yaml
---
medications:
  - id: MED-20260115T090000Z-1a2b3c4d
    name: Atorvastatin
    dose: 20mg
    route: oral
    frequency: once daily at night
    indication: Hypercholesterolaemia
    prescriber: Dr Example
    supplement: false
    status: active
    started: "2026-01-15"
    stopped: null
    stopped_reason: null
    recorded_at: 2026-01-15T09:00:00Z
    recorded_by: dr-example
    note: null
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `MED-<timestamp>-<random>`.
- `name` - medication or supplement display text.
- `dose` - optional dose, for example `20mg`.
- `route` - optional administration route, for example `oral`.
- `frequency` - optional dosing frequency, for example `twice daily`.
- `indication` - optional clinical reason for the medication.
- `prescriber` - optional prescriber name.
- `supplement` - `true` distinguishes an over-the-counter supplement from a
  prescribed medication.
- `status` - `active` or `stopped`.
- `started` - optional start date, `YYYY-MM-DD`.
- `stopped` / `stopped_reason` - set when a medication is stopped.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `note` - optional clinical note.

## FHIR R4

Expected mapping to FHIR R4 `MedicationStatement`:

| GitEHR field | FHIR R4 `MedicationStatement` |
|---|---|
| `status` | `status` (`active` / `stopped`) |
| `name` | `medicationCodeableConcept.text` |
| `dose` / `route` / `frequency` | `dosage[0].text` (or the structured `dosage` fields once dm+d coding lands) |
| `indication` | `reasonCode` |
| `prescriber` | `informationSource` |
| `started` | `effectivePeriod.start` |
| `stopped` | `effectivePeriod.end` |
| `note` | `note` |

`supplement` has no direct FHIR R4 `MedicationStatement` field; it is a
GitEHR-native flag pending a coded category. dm+d coding for `name` is
planned but not yet implemented (v1 stores display text only).

## Commands

```bash
gitehr medications list [--json] [--all]
gitehr medications add --name <name> [OPTIONS]
gitehr medications stop <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

`list` hides `stopped` entries by default. `--all` includes them.

`add` writes `state/medications.md`, stages it, writes a journal entry, and
commits both together. `--started`, when given, must be `YYYY-MM-DD`.

`stop` never deletes a medication entry. It changes the current state row and
writes a journal entry, preserving prior belief in Git history. `--date`
defaults to today and, when given, must be `YYYY-MM-DD`.
