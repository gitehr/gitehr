<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr observations`

Status: implemented v1, a single `state/observations.md` file, mirroring the shipped `gitehr conditions`/`gitehr medications` pattern. Completes [R61](../roadmap.md) alongside medications and conditions.

`gitehr observations` manages typed observation state in `state/observations.md`: vital signs, laboratory results, and other point-in-time measurements. The state file is the current queryable view; the journal entry written with each mutation is the audit narrative.

An **observation** is a single recorded measurement or finding - a blood pressure reading, a lab result, a body weight. Unlike conditions (which persist and evolve) or medications (which are started and stopped), an observation is a discrete fact tied to the moment it was made; `correct` exists only to fix an error in how that fact was recorded, not to describe a changing state.

## Storage

`state/observations.md` is YAML front matter with a top-level `observations` array. Each entry is one observation. A Markdown body and unrecognised YAML fields are preserved when the typed command updates a record, allowing later provenance, coding, and import fields to coexist with the v1 model.

All subcommands reject non-empty YAML without the `observations` array, malformed records, blank or whitespace-only stored IDs, names, or values, and duplicate IDs before filtering or mutation. Missing or empty state files and empty front matter are treated as empty state; invalid populated state is not silently treated as empty.

Updates atomically replace the state file and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

The shared persistence path requires one writer per patient worktree: no transaction-wide lock protects the load/modify/commit sequence, and concurrent writers can lose updates or interfere with rollback. Ordinary write/commit failures are rolled back; crash recovery and concurrent-writer coordination remain follow-up work.

Minimum useful record:

```yaml
---
observations:
  - id: OBS-20260907T044051Z-09df3e36
    name: Blood pressure
    code: "loinc:85354-9"
    category: vital-signs
    status: final
    value: "128/82"
    unit: mmHg
    interpretation: null
    effective_at: "2026-06-03"
    previous_value: null
    previous_unit: null
    correction_reason: null
    recorded_at: 2026-09-07T04:40:51Z
    recorded_by: dr-example
    note: null
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `OBS-<timestamp>-<random>`.
- `name` - human-readable display text (FHIR `Observation.code.text`).
- `code` - optional free-text terminology code, for example `loinc:85354-9`.
- `category` - optional `vital-signs` | `laboratory` | `social-history` | `imaging` | `procedure` | `survey` | `exam` | `therapy` | `activity`.
- `status` - `registered` | `preliminary` | `final` | `amended` | `corrected` | `cancelled` | `entered-in-error` | `unknown`. Defaults to `final`.
- `value` - the recorded result as free text, for example `128/82` or `37.1`. Panel/component observations (multiple sub-results under one observation) are not supported in v1; each result is its own observation.
- `unit` - optional unit of measurement.
- `interpretation` - optional free text, for example `high`, `low`, `normal`, `critical`.
- `effective_at` - free text: an ISO date or date-time describing when the observation was made. Not strictly validated as a date. Defaults to the recording time if not given.
- `previous_value` / `previous_unit` / `correction_reason` - set when an observation is corrected; null otherwise.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.
- `note` - optional clinical note.

The stable JSON output uses the same field names and nullability as this YAML model.

## FHIR R4

Expected mapping to FHIR R4 `Observation`:

| GitEHR field | FHIR R4 mapping |
|---|---|
| `status` | `Observation.status` |
| `category` | `Observation.category` |
| `name` | `Observation.code.text` |
| `code` | `Observation.code.coding` once a structured terminology binding lands (LOINC/SNOMED CT primarily; free text for v1) |
| `value` / `unit` | `Observation.valueQuantity` / `valueString` depending on kind, once typed values land (free text for v1) |
| `interpretation` | `Observation.interpretation` |
| `effective_at` | `Observation.effectiveDateTime` |
| `previous_value` / `correction_reason` | not directly represented in FHIR; GitEHR keeps the prior value alongside the corrected one so the journal-independent state view still shows what changed |
| `recorded_at` | `Observation.issued` |
| `recorded_by` | `Observation.performer` |
| `note` | `Observation.note` |

LOINC/SNOMED coding for `code`, and typed (numeric/coded) `value`, are planned but not yet implemented (v1 stores display text only). A future importer must preserve the source FHIR resource and provenance rather than treating this projection as the complete source record.

## Commands

```bash
gitehr observations list [--json] [--all] [--category <category>]
gitehr observations add --name <name> --value <value> [OPTIONS]
gitehr observations correct <id> --value <value> [--unit <text>] [--reason <text>]
gitehr observations show <id> [--json]
```

`list` shows current observations by default: `status` is neither `cancelled` nor `entered-in-error`. `--all` includes every status. `--category` filters to a single category; combine with `--all` to include cancelled/entered-in-error observations of that category.

`add` writes `state/observations.md` and a journal entry in one commit. `--name` and `--value` must not be blank or whitespace-only. `--status` defaults to `final`. `--effective` is free text and is not strictly validated as a date; when omitted it defaults to the recording timestamp. The journal contains the action, observation identity, optional note, and a complete YAML snapshot of the newly recorded observation so the original assertion remains reconstructable independently of later state changes.

`correct` never deletes an observation entry. It replaces `value` (and, if given, `unit`), records the previous value/unit and an optional reason, sets `status` to `corrected`, and writes a journal entry, preserving prior belief in Git history. An observation that is already `corrected`, `cancelled`, or `entered-in-error` cannot be corrected again, so the original correction (or the reason it was voided) cannot be overwritten.

`show` finds an observation by id across every status, not only current ones, and prints its full recorded detail.

Mutation commits contain only `state/observations.md` and the generated journal entry, leaving unrelated staged work untouched. A mutation refuses a dirty observation state file and restores the prior file and index state if writing or committing fails.

## Safety boundary

This command records what was measured or asserted about an observation. It does not validate clinical accuracy, plausibility, or coding, and does not adjudicate whether a value is correct. `status` reflects what was recorded, not adjudicated fact - a `final` status means someone finalised the entry, not that GitEHR checked it. Consumers must display provenance and distinguish an empty or absent entry from a clinically confirmed absence of the observation.
