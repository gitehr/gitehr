# gitehr medications

Manage current and past medications (including supplements) in
`state/medications.md`.

This is typed state for GUI/PHR display and automation. Mutations update the
state file and create a journal entry in the same commit.

## gitehr medications list

```text
gitehr medications list [--json] [--all]
```

Lists active medications by default. Use `--all` to include stopped entries
and `--json` for GUI/automation output.

## gitehr medications add

```text
gitehr medications add --name <name> [--dose <dose>] [--route <route>] [--frequency <frequency>] [--indication <text>] [--prescriber <name>] [--started <YYYY-MM-DD>] [--supplement] [--note <text>]
```

`--supplement` marks the entry as a supplement rather than a prescribed
medication, so PHR/GUI views can distinguish the two. `--started`, when
given, must be `YYYY-MM-DD`.

## gitehr medications stop

```text
gitehr medications stop <id> [--date <YYYY-MM-DD>] [--reason <text>]
```

Marks a medication stopped without deleting it. `--date` defaults to today
and, when given, must be `YYYY-MM-DD`.

Example:

```bash
gitehr medications add --name Atorvastatin --dose 20mg --route oral --frequency "once daily at night" --indication Hypercholesterolaemia --started 2026-01-15
gitehr medications list --json
gitehr medications stop MED-20260115T090000Z-1a2b3c4d --reason "Statin intolerance"
```
