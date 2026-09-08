<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr conditions`

Status: implemented v1, representation (A) from [`problem-condition-list.md`](../problem-condition-list.md): a single `state/conditions.md` file. Observations remain outstanding under [R61](../roadmap.md).

`gitehr conditions` manages typed condition and problem-list state in `state/conditions.md`. It follows the same pattern as `gitehr allergies` and `gitehr medications`: the state file is the current queryable view; the journal entry written with each mutation is the audit narrative.

A **condition** is any recorded health state - symptomatic or not, concerning or not. A **problem** is a condition that is, or might be, a current concern: the concern-filtered view over conditions where `category = problem-list-item` and the condition is current. Every problem is a condition; not every condition is a problem.

## Storage

`state/conditions.md` is YAML front matter with a top-level `conditions` array. Each entry is one condition. A Markdown body and unrecognised YAML fields are preserved when the typed command updates a record, allowing later provenance, coding, and import fields to coexist with the v1 model.

All subcommands reject non-empty YAML without the `conditions` array, malformed records, blank or whitespace-only stored IDs or names, and duplicate IDs before filtering or mutation. Missing or empty state files and empty front matter are treated as empty state; invalid populated state is not silently treated as empty.

Updates atomically replace the state file and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

The shared persistence path requires one writer per patient worktree: no transaction-wide lock protects the load/modify/commit sequence, and concurrent writers can lose updates or interfere with rollback. Ordinary write/commit failures are rolled back; crash recovery and concurrent-writer coordination remain follow-up work.

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

`list` shows current conditions by default: `clinical_status` is `active`, `recurrence`, `relapse`, or `remission`, and `verification_status` is neither `refuted` nor `entered-in-error`. `--all` includes every clinical and verification status. `--problems` only adds the category filter `category = problem-list-item`: alone it shows current problems; combined with `--all` it includes their history too.

`add` writes `state/conditions.md` and a journal entry in one commit. `--name` must not be blank or whitespace-only. `--status` defaults to `active`, `--verification` to `unconfirmed`, `--category` to `problem-list-item`. The journal contains the action, condition identity, optional note, and a complete YAML snapshot of the newly recorded condition so the original assertion remains reconstructable independently of later state changes.

`resolve` never deletes a condition entry. It sets `clinical_status` to `resolved`, records abatement details, and writes a journal entry, preserving prior belief in Git history. `--date` defaults to today (UTC), must be a valid calendar date in exact, zero-padded `YYYY-MM-DD` format when given, and cannot precede the stored `onset` when that parses as a `YYYY-MM-DD` date. Repeated resolution is rejected so the original abatement date and reason cannot be overwritten. Resolving a condition with `verification_status` of `refuted` or `entered-in-error` is also rejected.

`show` finds a condition by id across every status, not only current ones, and prints its full recorded detail.

Mutation commits contain only `state/conditions.md` and the generated journal entry, leaving unrelated staged work untouched. A mutation refuses a dirty condition state file and restores the prior file and index state if writing or committing fails.

## Safety boundary

This command records what a patient or clinician asserts about a health state. It does not diagnose, does not validate clinical accuracy or coding, and does not adjudicate whether a condition is real, resolved, or correctly categorised. `category` (`problem-list-item` vs `encounter-diagnosis`) and `clinical_status` / `verification_status` reflect what was recorded, not adjudicated fact - a `confirmed` verification status means someone asserted confirmation, not that GitEHR checked it. Consumers must display provenance and distinguish an empty or absent entry from a clinically confirmed absence of the condition.
