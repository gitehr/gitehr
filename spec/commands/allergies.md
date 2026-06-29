<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr allergies`

Manages the current allergy and adverse-reaction summary stored in
`state/allergies.md`. This is typed state for GUI headers, clinical warning
strips, and automation. The journal remains the audit trail for why the current
state changed.

All subcommands require the current directory to be a GitEHR repository.

### `gitehr allergies list [--json] [--all]`

Lists active allergies by default. With `--all`, includes inactive entries.
With `--json`, emits a stable JSON array for GUI and automation callers.

The JSON shape for each allergy is:

```json
{
  "id": "ALG-20260629T120000Z-4f2a9c1b",
  "agent": "Penicillin",
  "reaction": "Rash",
  "severity": "high",
  "status": "active",
  "recorded_at": "2026-06-29T12:00:00Z",
  "recorded_by": "dr-smith",
  "inactive_at": null,
  "inactive_reason": null,
  "note": "Patient reports childhood reaction."
}
```

Severity values are `low`, `moderate`, `high`, and `critical`.
Status values are `active` and `inactive`.

### `gitehr allergies add --agent <agent> --reaction <reaction> [OPTIONS]`

Adds an active allergy or adverse reaction.

**Options:**

| Option | Description |
|---|---|
| `--agent <agent>` | Allergen or causative agent |
| `--reaction <reaction>` | Reaction text |
| `--severity <severity>` | `low`, `moderate`, `high`, or `critical` (default `moderate`) |
| `--note <text>` | Optional clinical note |

Behavior:

- Appends a new active entry to `state/allergies.md`.
- Generates a stable allergy id.
- Stages `state/allergies.md`, writes a journal entry describing the addition,
  and commits both together.

### `gitehr allergies inactive <id> [--reason <text>]`

Marks an allergy inactive without deleting it.

Behavior:

- Finds the matching allergy id in `state/allergies.md`.
- Sets `status: inactive`, `inactive_at`, and optional `inactive_reason`.
- Stages `state/allergies.md`, writes a journal entry describing the inactivation,
  and commits both together.

Examples:

```bash
gitehr allergies add --agent Penicillin --reaction Rash --severity high
gitehr allergies list --json
gitehr allergies inactive ALG-20260629T120000Z-4f2a9c1b --reason "Entered in error."
```
