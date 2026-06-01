# gitehr state

Manage the mutable clinical state files under `state/`. State holds current information that changes over time: medications, allergies, problems, vitals, demographics. Compare with [`gitehr journal`](journal.md), which is append-only.

All subcommands require the current directory to be a GitEHR repository.

## gitehr state list

```text
gitehr state list
```

Lists files in `state/`, excluding `README.md`, with their last-modified timestamps when available.

Run with no subcommand for the same effect: `gitehr state`.

## gitehr state get

```text
gitehr state get <filename>
```

Prints the contents of the named state file. Fails if the file does not exist in `state/`.

## gitehr state set

```text
gitehr state set <filename> <content>
```

Writes content to the named state file, creating `state/` if needed. Overwrites any existing file of the same name.

!!! warning "Audit trail"
    State mutations are tracked by Git history alone. For high-significance changes, also write a [`gitehr journal add`](journal.md#gitehr-journal-add) entry that explains the change. The journal is the canonical audit trail; state is the current snapshot.
