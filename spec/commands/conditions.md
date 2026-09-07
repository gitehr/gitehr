<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr conditions`

Status: implemented v1, representation (A) from [`problem-condition-list.md`](../problem-condition-list.md): a single `state/conditions.md` file. Observations remain outstanding under [R61](../roadmap.md).

`gitehr conditions` manages typed condition and problem-list state in `state/conditions.md`. It follows the same pattern as `gitehr allergies` and `gitehr medications`: the state file is the current queryable view; the journal entry written with each mutation is the audit narrative.

A **condition** is any recorded health state - symptomatic or not, concerning or not. A **problem** is a condition that is, or might be, a current concern: the concern-filtered view over conditions where `category = problem-list-item` and the condition is current. Every problem is a condition; not every condition is a problem.

## Storage

`state/conditions.md` is YAML front matter with a top-level `conditions` array. Each entry is one condition. A Markdown body and unrecognised YAML fields are preserved when the typed command updates a record, allowing later provenance, coding, and import fields to coexist with the v1 model.

Updates atomically replace the state file and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

Minimum useful record:

```yaml
---
conditions:
  - id: COND-20260907T044051Z-09df3e36
    name: Type 2 diabetes mellitus
    clinical_status: active
    verification_status: confirmed
    category: problem-list-item
    onset: "2020-03-01"
    abatement: null
    abatement_reason: null
    body_site: null
    laterality: null
    code: "snomed:44054006"
    severity: moderate
    recorded_at: 2026-09-07T04:40:51Z
    recorded_by: dr-example
    note: null
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `COND-<timestamp>-<random>`.
- `name` - human-readable display text (mutable; the `id` is the stable handle, so renaming as understanding changes does not break links).
- `clinical_status` - `active` | `recurrence` | `relapse` | `inactive` | `remission` | `resolved`.
- `verification_status` - `unconfirmed` | `provisional` | `differential` | `confirmed` | `refuted` | `entered-in-error`. Gives a working diagnosis a lifecycle, and gives "entered in error" a home without deletion.
- `category` - `problem-list-item` | `encounter-diagnosis`; the problem-vs-condition switch.
- `onset` - free text: an ISO date, a year, "childhood", or another approximate description. Not strictly validated as a date.
- `abatement` / `abatement_reason` - set when a condition is resolved.
- `body_site` - optional free-text anatomical location.
- `laterality` - optional `left` | `right` | `bilateral` | `midline`.
- `code` - optional free-text terminology code, for example `snomed:44054006`.
- `severity` - optional free text (no fixed value set, unlike allergy severity).
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `note` - optional clinical note.

The stable JSON output uses the same field names and nullability as this YAML model.

## FHIR R4

Expected mapping to FHIR R4 `Condition`:

| GitEHR field | FHIR R4 mapping |
|---|---|
| `clinical_status` | `Condition.clinicalStatus` |
| `verification_status` | `Condition.verificationStatus` |
| `category` | `Condition.category` (`problem-list-item` / `encounter-diagnosis`) |
| `name` | `Condition.code.text` |
| `code` | `Condition.code.coding` once a structured terminology binding lands (SNOMED CT primarily; free text for v1) |
| `onset` | `Condition.onsetString` / `onsetDateTime` / `onsetAge` depending on precision |
| `abatement` / `abatement_reason` | `Condition.abatementDateTime` / a linked note explaining cessation |
| `body_site` / `laterality` | `Condition.bodySite` |
| `severity` | `Condition.severity` |
| `recorded_at` | `Condition.recordedDate` |
| `recorded_by` | `Condition.recorder` |
| `note` | `Condition.note` |

dm+d/SNOMED coding for `code` is planned but not yet implemented (v1 stores display text only). A future importer must preserve the source FHIR resource and provenance rather than treating this projection as the complete source record.

## Commands

```bash
gitehr conditions list [--json] [--all] [--problems]
gitehr conditions add --name <name> [OPTIONS]
gitehr conditions resolve <id> [--date <YYYY-MM-DD>] [--reason <text>]
gitehr conditions show <id> [--json]
```

`list` hides `inactive` and `resolved` entries by default (a condition is "current" while `clinical_status` is `active`, `recurrence`, `relapse`, or `remission`). `--all` includes every status. `--problems` further restricts to `category = problem-list-item`, giving the problem-list projection described in [`problem-condition-list.md`](../problem-condition-list.md).

`add` writes `state/conditions.md` and a journal entry in one commit. `--status` defaults to `active`, `--verification` to `unconfirmed`, `--category` to `problem-list-item`.

`resolve` never deletes a condition entry. It changes the current state row and writes a journal entry, preserving prior belief in Git history. `--date` defaults to today, must use `YYYY-MM-DD` when given, and (when the stored `onset` itself parses as a clean `YYYY-MM-DD` date) cannot precede it. Repeated resolution is rejected so the original abatement date and reason cannot be overwritten.

`show` finds a condition by id across every status, not only current ones, and prints its full recorded detail.

Mutation commits contain only `state/conditions.md` and the generated journal entry, leaving unrelated staged work untouched. A mutation refuses a dirty condition state file and restores the prior file and index state if writing or committing fails.

## Safety boundary

This command records what a patient or clinician asserts about a health state. It does not diagnose, does not validate clinical accuracy or coding, and does not adjudicate whether a condition is real, resolved, or correctly categorised. `category` (`problem-list-item` vs `encounter-diagnosis`) and `clinical_status` / `verification_status` reflect what was recorded, not adjudicated fact - a `confirmed` verification status means someone asserted confirmation, not that GitEHR checked it. Consumers must display provenance and distinguish an empty or absent entry from a clinically confirmed absence of the condition.
