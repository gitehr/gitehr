# gitehr state

Manage the mutable clinical state files under `state/`. State holds current information that changes over time: medications, allergies, problems, vitals, demographics. Compare with [`gitehr journal`](journal.md), which is append-only.

`gitehr state` is the raw file-level interface. Prefer typed commands such as
[`gitehr demographics`](demographics.md), [`gitehr allergies`](allergies.md),
[`gitehr conditions`](conditions.md), [`gitehr family-history`](family-history.md), [`gitehr medications`](medications.md),
[`gitehr observations`](observations.md), and [`gitehr vaccinations`](vaccinations.md) for GUI-facing or
automation-facing clinical state.

All subcommands run against the subject repository resolved from the current directory: the nearest enclosing directory containing `.gitehr/` (so they work from any subdirectory of a record), or the only subject when run at a Store root with exactly one subject.

## gitehr state list

```text
gitehr state list
```

Lists files recursively in `state/`, excluding its `README.md`, with their last-modified timestamps when available. A binary or non-UTF-8 file is still listed. A symbolic link anywhere under `state/` is refused rather than followed.

Run with no subcommand for the same effect: `gitehr state`.

## gitehr state get

```text
gitehr state get <filename>
```

Prints the contents of the named state file. Fails if the file does not exist in `state/`, is not valid UTF-8 text, or is reached through a symbolic link.

## gitehr state set

```text
gitehr state set <filename> <content>
gitehr state set <filename> --file <path|->
gitehr state set <filename> --file <path|-> --commit
```

Writes content to the named state file, creating `state/` (and any subdirectory in `filename`) if needed. Overwrites any existing file of the same name. Use `--file -` to read content from standard input rather than exposing sensitive content in process arguments; a relative `--file` path is resolved against the directory you ran the command from. Pass `--commit` to create a commit containing only the updated state file, leaving any other staged work staged; it refuses to overwrite an uncommitted target file, and if the new content matches what is already committed it reports that there is nothing to commit and succeeds without creating a commit. If the commit itself fails, the file is unstaged but the new content stays in the working tree.

`filename` may include subdirectories, for example `calculations/feverpain-latest.json`; an absolute path or any `..` component is rejected.

!!! warning "Audit trail"
    State mutations are tracked by Git history alone. For high-significance changes, also write a [`gitehr journal add`](journal.md#gitehr-journal-add) entry that explains the change. The journal is the canonical audit trail; state is the current snapshot.

Typed state commands write that companion journal entry automatically.
