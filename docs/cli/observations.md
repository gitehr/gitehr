<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr observations

Manage typed observation state - vital signs, laboratory results, and similar measurements - in `state/observations.md`.

Run from the selected patient's repository (or a single-subject Store). New repositories include an empty observations template; existing repositories need no migration, because the first `add` creates the file if it is missing.

This is typed state for GUI/PHR display and automation. Each successful mutation updates the state file and creates a journal entry in the same isolated commit. The command refuses to overwrite a state file with uncommitted changes, and restores the previous state if the commit fails.

An **observation** is a single recorded measurement or finding - a blood pressure reading, a lab result, a body weight - tied to the moment it was made. `correct` fixes an error in how that fact was recorded; it does not describe an evolving state the way `gitehr conditions` or `gitehr medications` do.

This is a GitEHR-native model, not a FHIR resource or a FHIR import/export interface. Values, units, codes, and interpretations are unvalidated text; structured quantities, reference ranges, panels/components, and source-resource import are not implemented. GitEHR does not check clinical accuracy or plausibility. An empty list is not evidence that no observations exist, and the default `final` status is not evidence of clinical verification.

State updates use atomic file replacement and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

Use only one writer per patient worktree. There is no transaction-wide lock across reading state and committing changes, so concurrent writers can overwrite each other's updates. Rollback handles ordinary write/commit errors, not recovery after a process crash or power loss.

All subcommands reject non-empty YAML without an `observations` array, malformed records, blank or whitespace-only stored IDs, names, values, recording timestamps, or non-null previous values, and duplicate IDs. Missing or empty state files and empty front matter are treated as empty state; invalid populated state is not silently treated as empty. Unrecognised YAML fields and the Markdown body are preserved on updates.

## gitehr observations list

```text
gitehr observations list [--json] [--all] [--category <category>]
```

Lists current observations by default: `status` is neither `cancelled` nor `entered-in-error`. Use `--all` to include every status. `--category` filters to one category (`vital-signs`, `laboratory`, `social-history`, `imaging`, `procedure`, `survey`, `exam`, `therapy`, `activity`); combine with `--all` to include cancelled/entered-in-error observations of that category too. Use `--json` for GUI/automation output.

Here, "current" means not cancelled or entered in error, not recent or clinically verified: historical, preliminary, and unknown-status readings remain visible. Results follow stored order; this is not a latest-per-measurement or date-sorted view.

## gitehr observations add

```text
gitehr observations add --name <name> --value <value> [--unit <text>] [--code <code>] [--category <category>] [--status <status>] [--effective <text>] [--interpretation <text>] [--note <text>]
```

`--name` and `--value` must not be blank or whitespace-only. `--status` accepts `registered`, `preliminary`, `final`, `amended`, `corrected`, `cancelled`, `entered-in-error`, or `unknown`, and defaults to `final`. `--effective` is free text describing when the observation was made, such as an ISO date, date-time, or approximate date, and is not strictly validated as a date; it must not be blank when supplied and defaults to the recording timestamp only when omitted. For historical readings, supply the source measurement time or an explicit description of uncertainty rather than accepting that default. An optional note is appended to the generated audit narrative; it does not replace the action and observation identity. The add journal entry also contains a complete YAML snapshot of the newly recorded observation, preserving the original assertion independently of later state changes.

## gitehr observations correct

```text
gitehr observations correct <id> --value <value> [--unit <text>] [--reason <text>]
```

Corrects a previously recorded observation without deleting it. Records the prior `value`/`unit` and sets `status` to `corrected`, writing a journal entry with the old and new values and complete before/after YAML snapshots. If `--unit` is omitted, the previous unit is kept; a supplied unit must not be blank. The correction must change the value or unit. An observation that is already `corrected`, `cancelled`, or `entered-in-error`, or has any non-null correction metadata, cannot be corrected, so existing correction history cannot be overwritten even if its status was changed externally. The old interpretation is preserved as `previous_interpretation` and `interpretation` is cleared, not recalculated for the new value. Measurement time and original recording metadata are unchanged.

## gitehr observations show

```text
gitehr observations show <id> [--json]
```

Shows a single observation by id, searching all observations regardless of status.

Example (use the ID printed by `add` in place of `OBS-ID`):

```bash
gitehr observations add --name "Blood pressure" --value "128/82" --unit mmHg --category vital-signs --code loinc:85354-9 --effective 2026-06-03
gitehr observations list --category vital-signs --json
gitehr observations correct OBS-ID --value "134/88" --reason "Original reading was transcribed incorrectly"
gitehr observations show OBS-ID
```
