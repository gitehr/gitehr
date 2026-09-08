<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr conditions

Manage typed condition and problem-list state in `state/conditions.md`.

This is typed state for GUI/PHR display and automation. Each successful mutation updates the state file and creates a journal entry in the same isolated commit. The command refuses to overwrite a state file with uncommitted changes, and restores the previous state if the commit fails.

A **condition** is any recorded health state, symptomatic or not; a **problem** is a condition category-tagged `problem-list-item` that is current. `gitehr conditions list --problems` is the problem-list view.

State updates use atomic file replacement and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

Use only one writer per patient worktree. There is no transaction-wide lock across reading state and committing changes, so concurrent writers can overwrite each other's updates. Rollback handles ordinary write/commit errors, not recovery after a process crash or power loss.

All subcommands reject non-empty YAML without a `conditions` array, malformed records, blank or whitespace-only stored IDs or names, and duplicate IDs. Missing or empty state files and empty front matter are treated as empty state; invalid populated state is not silently treated as empty. Unrecognised YAML fields and the Markdown body are preserved on updates.

## gitehr conditions list

```text
gitehr conditions list [--json] [--all] [--problems]
```

Lists current conditions by default: `clinical_status` is `active`, `recurrence`, `relapse`, or `remission`, and `verification_status` is neither `refuted` nor `entered-in-error`. Use `--all` to include every clinical and verification status. `--problems` is a category filter for `problem-list-item`: alone it shows current problems; combined with `--all` it includes their history too. Use `--json` for GUI/automation output.

## gitehr conditions add

```text
gitehr conditions add --name <name> [--status <status>] [--verification <verification>] [--category <category>] [--onset <text>] [--code <code>] [--body-site <text>] [--laterality <laterality>] [--severity <text>] [--note <text>]
```

`--status` defaults to `active`, `--verification` defaults to `unconfirmed`, `--category` defaults to `problem-list-item`. `--name` must not be blank or whitespace-only. `--onset` is free text (an ISO date, a year, "childhood", etc.) and is not strictly validated as a date. An optional note is appended to the generated audit narrative; it does not replace the action and condition identity. The add journal entry also contains a complete YAML snapshot of the newly recorded condition, preserving the original assertion independently of later state changes.

## gitehr conditions resolve

```text
gitehr conditions resolve <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

Marks a condition resolved without deleting it. `--date` defaults to today (UTC) and must be a valid calendar date in exact, zero-padded `YYYY-MM-DD` format. If the stored `onset` parses as `YYYY-MM-DD`, the resolve date cannot precede it; a free-text onset (a year, "childhood", etc.) skips that check. A resolved condition cannot be resolved again because that would overwrite its original abatement details. Resolving a `refuted` or `entered-in-error` condition is also refused. An optional reason is appended to the generated audit narrative.

## gitehr conditions show

```text
gitehr conditions show <id> [--json]
```

Shows a single condition by id, searching all conditions regardless of status.

Example:

```bash
gitehr conditions add --name "Type 2 diabetes mellitus" --category problem-list-item --verification confirmed --onset 2020-03-01 --code snomed:44054006 --severity moderate
gitehr conditions list --problems --json
gitehr conditions resolve COND-20260907T044051Z-09df3e36 --reason "Achieved remission via lifestyle change"
gitehr conditions show COND-20260907T044051Z-09df3e36
```
