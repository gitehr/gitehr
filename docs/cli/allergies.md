# gitehr allergies

Manage current allergies and adverse reactions in `state/allergies.md`.

This is typed state for GUI warning bars and automation. Mutations update the
state file and create a journal entry in the same commit.

## gitehr allergies list

```text
gitehr allergies list [--json] [--all]
```

Lists active allergies by default. Use `--all` to include inactive entries and
`--json` for GUI/automation output.

## gitehr allergies add

```text
gitehr allergies add --agent <agent> --reaction <reaction> [--severity <severity>] [--note <text>]
```

Severity is one of `low`, `moderate`, `high`, or `critical`; default is
`moderate`.

## gitehr allergies inactive

```text
gitehr allergies inactive <id> [--reason <text>]
```

Marks an allergy inactive without deleting it.

Example:

```bash
gitehr allergies add --agent Penicillin --reaction Rash --severity high
gitehr allergies list --json
```
