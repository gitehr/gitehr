<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr medications`

Status: implemented v1. Family history remains outstanding under [issue #84](https://github.com/gitehr/gitehr/issues/84); immunisation state is already implemented by [`gitehr vaccinations`](vaccinations.md), including `immunisations` and `immunizations` aliases.

`gitehr medications` manages typed medication state in `state/medications.md`. It follows the same pattern as `gitehr allergies` and `gitehr vaccinations`: the state file is the current queryable view; the journal entry written with each mutation is the audit narrative.

## Storage

`state/medications.md` is YAML front matter with a top-level `medications` array. Each entry is one medication or supplement. A Markdown body and unrecognised YAML fields are preserved when the typed command updates a record, allowing later provenance, coding, and import fields to coexist with the v1 model.

Updates atomically replace the state file and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

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
- `supplement` - `true` distinguishes a supplement from a prescribed medication; `false` means only that it was not explicitly marked as a supplement.
- `status` - `active` or `stopped`.
- `started` - optional start date, `YYYY-MM-DD`.
- `stopped` / `stopped_reason` - set when a medication is stopped.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `note` - optional clinical note.

The stable JSON output uses the same field names and nullability as this YAML model.

## FHIR R4

Expected mapping to FHIR R4 `MedicationStatement`:

| GitEHR field | FHIR R4 mapping |
|---|---|
| `status` | `MedicationStatement.status` (`active` / `stopped`) |
| `name` | `MedicationStatement.medicationCodeableConcept.text` |
| `dose` / `route` / `frequency` | `MedicationStatement.dosage[0]` text or structured dosage fields once dm+d coding lands |
| `indication` | `MedicationStatement.reasonCode` |
| `started` / `stopped` | `MedicationStatement.effectivePeriod` |
| `stopped_reason` | `MedicationStatement.statusReason` where a suitable code or text representation is available |
| `recorded_at` | `MedicationStatement.dateAsserted` |
| `recorded_by` | `MedicationStatement.informationSource` when the recorded contributor is the source of the statement |
| `prescriber` | No direct `MedicationStatement` field; a linked `MedicationRequest.requester` represents the prescribing actor |
| `note` | `MedicationStatement.note` |

`supplement` has no direct FHIR R4 `MedicationStatement` field; it is a GitEHR-native flag pending a coded category. dm+d coding for `name` is planned but not yet implemented (v1 stores display text only). A future importer must preserve the source FHIR resource and provenance rather than treating this projection as the complete source record.

## Commands

```bash
gitehr medications list [--json] [--all]
gitehr medications add --name <name> [OPTIONS]
gitehr medications stop <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

`list` hides `stopped` entries by default. `--all` includes them.

`add` writes `state/medications.md` and a journal entry in one commit. `--started`, when given, must use `YYYY-MM-DD`.

`stop` never deletes a medication entry. It changes the current state row and writes a journal entry, preserving prior belief in Git history. `--date` defaults to today, must use `YYYY-MM-DD`, and cannot precede `started`. Repeated stopping is rejected so the original stop date and reason cannot be overwritten.

Mutation commits contain only `state/medications.md` and the generated journal entry, leaving unrelated staged work untouched. A mutation refuses a dirty medication state file and restores the prior file and index state if writing or committing fails.

## Safety boundary

This command records patient- or clinician-supplied medication information. It does not prescribe, recommend, reconcile, validate dose appropriateness, infer adherence, or establish that an omitted medication is absent. Consumers must display provenance and distinguish missing or unextracted data from an authoritative empty medication list.
