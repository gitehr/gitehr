# gitehr demographics

Manage the current patient demographic summary in `state/demographics.md`.

This is typed state for GUI headers and automation. Updates also create a journal
entry, so the current snapshot and audit trail stay together.

## gitehr demographics show

```text
gitehr demographics show [--json]
```

Shows the current demographics. Use `--json` for GUI/automation output.

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

Example:

```bash
gitehr demographics set --title Mr --full-name "Alex Smith" --date-of-birth 1970-01-01 --nhs-number 1234567890
gitehr demographics show --json
```
