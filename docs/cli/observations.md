<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr observations

Manage typed observation state - vital signs, laboratory results, and similar measurements - in `state/observations.md`.

This is typed state for GUI/PHR display and automation. Each successful mutation updates the state file and creates a journal entry in the same isolated commit. The command refuses to overwrite a state file with uncommitted changes, and restores the previous state if the commit fails.

An **observation** is a single recorded measurement or finding - a blood pressure reading, a lab result, a body weight - tied to the moment it was made. `correct` fixes an error in how that fact was recorded; it does not describe an evolving state the way `gitehr conditions` or `gitehr medications` do.

State updates use atomic file replacement and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

Use only one writer per patient worktree. There is no transaction-wide lock across reading state and committing changes, so concurrent writers can overwrite each other's updates. Rollback handles ordinary write/commit errors, not recovery after a process crash or power loss.

All subcommands reject non-empty YAML without an `observations` array, malformed records, blank or whitespace-only stored IDs, names, or values, and duplicate IDs. Missing or empty state files and empty front matter are treated as empty state; invalid populated state is not silently treated as empty. Unrecognised YAML fields and the Markdown body are preserved on updates.

## gitehr observations list

```text
gitehr observations list [--json] [--all] [--category <category>]
```

Lists current observations by default: `status` is neither `cancelled` nor `entered-in-error`. Use `--all` to include every status. `--category` filters to one category (`vital-signs`, `laboratory`, `social-history`, `imaging`, `procedure`, `survey`, `exam`, `therapy`, `activity`); combine with `--all` to include cancelled/entered-in-error observations of that category too. Use `--json` for GUI/automation output.

## gitehr observations add

```text
gitehr observations add --name <name> --value <value> [--unit <text>] [--code <code>] [--category <category>] [--status <status>] [--effective <text>] [--interpretation <text>] [--note <text>]
```

`--name` and `--value` must not be blank or whitespace-only. `--status` defaults to `final`. `--effective` is free text (an ISO date or date-time) describing when the observation was made, and is not strictly validated; it defaults to the recording timestamp when omitted. An optional note is appended to the generated audit narrative; it does not replace the action and observation identity. The add journal entry also contains a complete YAML snapshot of the newly recorded observation, preserving the original assertion independently of later state changes.

## gitehr observations correct

```text
gitehr observations correct <id> --value <value> [--unit <text>] [--reason <text>]
```

Corrects a previously recorded observation without deleting it. Records the prior `value`/`unit` and sets `status` to `corrected`, writing a journal entry with the old and new values. If `--unit` is omitted, the previous unit is kept. An observation that is already `corrected`, `cancelled`, or `entered-in-error` cannot be corrected again, so the original correction (or the reason it was voided) cannot be overwritten.

## gitehr observations show

```text
gitehr observations show <id> [--json]
```

Shows a single observation by id, searching all observations regardless of status.

Example:

```bash
gitehr observations add --name "Blood pressure" --value "128/82" --unit mmHg --category vital-signs --code loinc:85354-9 --effective 2026-06-03
gitehr observations list --category vital-signs --json
gitehr observations correct OBS-20260907T044051Z-09df3e36 --value "134/88" --reason "Original reading was transcribed incorrectly"
gitehr observations show OBS-20260907T044051Z-09df3e36
```
