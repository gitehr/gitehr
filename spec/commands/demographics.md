<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr demographics`

Manages the current patient demographic summary stored in `state/demographics.md`.
This is typed state: it is the current snapshot used by GUI headers, summaries,
and automation. It complements the append-only journal, which remains the audit
trail explaining how the state changed.

All subcommands require the current directory to be a GitEHR repository.

### `gitehr demographics show [--json]`

Shows the current demographic summary. With `--json`, emits a stable JSON object
for GUI and automation callers.

The JSON shape is:

```json
{
  "title": "Mr",
  "full_name": "Alex Smith",
  "preferred_name": "Alex",
  "address": "1 High Street, Anytown",
  "date_of_birth": "1970-01-01",
  "nhs_number": "1234567890",
  "identifiers": [
    { "type": "NHS", "value": "1234567890" },
    { "type": "MRN", "value": "A12345" }
  ]
}
```

Missing optional fields are `null` or empty arrays in JSON.

### `gitehr demographics set [OPTIONS]`

Updates the current demographic summary.

**Options:**

| Option | Description |
|---|---|
| `--title <title>` | Title or honorific, e.g. `Mr`, `Mrs`, `Dr` |
| `--full-name <name>` | Full display name |
| `--preferred-name <name>` | Preferred or known-as name |
| `--address <address>` | Current address as display text |
| `--date-of-birth <date>` | Date of birth in `YYYY-MM-DD` format |
| `--nhs-number <number>` | NHS number as display text |
| `--identifier <type:value>` | Additional identifier; repeatable |
| `--note <text>` | Optional journal narrative for the state change |

Behavior:

- Reads any existing `state/demographics.md`, updates only fields supplied on the
  command line, then writes the file back as YAML front matter.
- Validates `--date-of-birth` is parseable as `YYYY-MM-DD`.
- Stages `state/demographics.md`, writes a journal entry describing the state
  change, and commits both together.
- The raw `gitehr state get|set` commands remain available, but GUI-facing code
  should prefer this typed command.

Examples:

```bash
gitehr demographics set \
  --title Mr \
  --full-name "Alex Smith" \
  --date-of-birth 1970-01-01 \
  --nhs-number 1234567890

gitehr demographics set --identifier MRN:A12345 --note "Added hospital MRN from referral letter."

gitehr demographics show --json
```
