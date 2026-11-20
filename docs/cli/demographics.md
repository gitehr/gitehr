# gitehr demographics

Manage the current patient demographic summary in `state/demographics.md`.

This is typed state for GUI headers and automation. Updates also create a journal
entry in the same isolated commit, so the current snapshot and audit trail stay
together. The command refuses to overwrite a state file with uncommitted changes,
and restores the previous state if the commit fails.

## gitehr demographics show

```text
gitehr demographics show [--json]
```

Shows the current demographics. The human-readable view lists name, preferred
name, date of birth, NHS number and identifiers; use `--json` for every field,
including title and address.

## gitehr demographics set

```text
gitehr demographics set [OPTIONS]
```

Options:

- `--title <title>`
- `--full-name <name>`
- `--preferred-name <name>`
- `--address <address>`
- `--date-of-birth <YYYY-MM-DD>`
- `--nhs-number <number>`
- `--identifier <type:value>` repeatable
- `--note <text>` optional journal narrative

Only the fields you supply change; blank values are ignored, so a field cannot
be cleared with this command. `--nhs-number` also sets the `NHS` identifier, and
an `--identifier` replaces any existing identifier of the same type (matched
case-insensitively).

Example:

```bash
gitehr demographics set --title Mr --full-name "Alex Smith" --date-of-birth 1970-01-01 --nhs-number 1234567890
gitehr demographics show --json
```
