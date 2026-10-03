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
gitehr allergies add --agent <agent> --reaction <reaction> [--severity <severity>] [--note <text>] [--source-type <type>] [--source-detail <text>] [--acquired-via <acquisition-id>] [--evidence-level <level>] [--confidence <level>]
```

Severity is one of `low`, `moderate`, `high`, or `critical`; default is
`moderate`.

The optional `--source-type` (`self-reported`, `clinician-asserted`, `portal-extracted`, `sar`, `paper-transcribed`, `device`, `inferred`), `--source-detail`, `--acquired-via`, `--evidence-level` (`documented`, `inferred`, `assumed`) and `--confidence` (`high`, `medium`, `low`) flags record a `provenance` block on the entry. `--source-type` is required when supplying any other provenance flag. The block is omitted when none is given.

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
