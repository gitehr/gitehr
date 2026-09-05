<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr medications

Manage current and past medications, including supplements, in `state/medications.md`.

This is typed state for GUI/PHR display and automation. Each successful mutation updates the state file and creates a journal entry in the same isolated commit. The command refuses to overwrite a state file with uncommitted changes, and restores the previous state if the commit fails.

State updates use atomic file replacement and preserve Unix mode bits. File-specific ACLs, extended attributes, and Windows file attributes are not preserved. Configure required ACLs on the `state/` directory so replacement files inherit them; other per-file metadata is unsupported.

## gitehr medications list

```text
gitehr medications list [--json] [--all]
```

Lists active medications by default. Use `--all` to include stopped entries and `--json` for GUI/automation output.

## gitehr medications add

```text
gitehr medications add --name <name> [--dose <dose>] [--route <route>] [--frequency <frequency>] [--indication <text>] [--prescriber <name>] [--started <YYYY-MM-DD>] [--supplement] [--note <text>]
```

`--supplement` marks the entry as a supplement rather than a prescribed medication, so PHR/GUI views can distinguish the two. `--started`, when given, must be `YYYY-MM-DD`. An optional note is appended to the generated audit narrative; it does not replace the action and medication identity.

## gitehr medications stop

```text
gitehr medications stop <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

Marks an active medication stopped without deleting it. `--date` defaults to today, must use `YYYY-MM-DD`, and cannot precede the medication's start date. A stopped medication cannot be stopped again because that would overwrite its original cessation details. An optional reason is appended to the generated audit narrative.

Example:

```bash
gitehr medications add --name Atorvastatin --dose 20mg --route oral --frequency "once daily at night" --indication Hypercholesterolaemia --started 2026-01-15
gitehr medications list --json
gitehr medications stop MED-20260115T090000Z-1a2b3c4d --reason "Statin intolerance"
```
