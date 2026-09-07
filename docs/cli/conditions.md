<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr conditions

Manage typed condition and problem-list state in `state/conditions.md`.

This is typed state for GUI/PHR display and automation. Each successful mutation updates the state file and creates a journal entry in the same isolated commit. The command refuses to overwrite a state file with uncommitted changes, and restores the previous state if the commit fails.

A **condition** is any recorded health state, symptomatic or not; a **problem** is a condition category-tagged `problem-list-item` that is current. `gitehr conditions list --problems` is the problem-list view.

State updates use atomic file replacement and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

## gitehr conditions list

```text
gitehr conditions list [--json] [--all] [--problems]
```

Lists current conditions by default (any `clinical_status` other than `inactive` or `resolved`). Use `--all` to include resolved and inactive conditions, `--problems` to show only active `problem-list-item` entries, and `--json` for GUI/automation output.

## gitehr conditions add

```text
gitehr conditions add --name <name> [--status <status>] [--verification <verification>] [--category <category>] [--onset <text>] [--code <code>] [--body-site <text>] [--laterality <laterality>] [--severity <text>] [--note <text>]
```

`--status` defaults to `active`, `--verification` defaults to `unconfirmed`, `--category` defaults to `problem-list-item`. `--onset` is free text (an ISO date, a year, "childhood", etc.) and is not strictly validated as a date. An optional note is appended to the generated audit narrative; it does not replace the action and condition identity.

## gitehr conditions resolve

```text
gitehr conditions resolve <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

Marks a condition resolved without deleting it. `--date` defaults to today and must use `YYYY-MM-DD`. If the stored `onset` parses as `YYYY-MM-DD`, the resolve date cannot precede it; a free-text onset (a year, "childhood", etc.) skips that check. A resolved condition cannot be resolved again because that would overwrite its original abatement details. An optional reason is appended to the generated audit narrative.

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
