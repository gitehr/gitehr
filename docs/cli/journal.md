# gitehr journal

Manage the append-only journal of clinical entries. Every entry is content-addressed by SHA-256 and chained to its parent, building a [Merkle DAG](../design/redundancy.md#tamper-resistance) for the patient's record.

All subcommands require the current directory to be a GitEHR repository.

## gitehr journal add

```text
gitehr journal add [<content>] [-f|--file <path>]
```

Adds a new clinical journal entry.

Input modes:

- **Inline:** `gitehr journal add "Patient reviewed and plan updated."`
- **From file:** `gitehr journal add --file note.md`
- **From stdin:** `cat note.md | gitehr journal add --file -`

Behavior:

- Reads the latest journal entry, computes its SHA-256, and writes that as the new entry's `parent_hash`.
- Creates `journal/<ISO8601-timestamp>-<uuid>.md` with YAML front matter (`parent_hash`, `parent_entry`, `timestamp`, optional `author`) and the supplied content as the body.
- `author` is the user activated by `gitehr user activate <id>`; absent if no user is active.
- Stages the new file with `git add` and commits with message `Journal entry: <filename>`.
- Rejects passing both inline content and `--file`.

## gitehr journal show

```text
gitehr journal show [-n|--limit N] [-o|--offset N] [-r|--reverse] [-a|--all]
```

Lists journal entries with metadata and an 80-character preview.

Options:

- `-n, --limit <N>`: maximum entries (default 10)
- `-o, --offset <N>`: skip this many entries from the start (default 0)
- `-r, --reverse`: newest first
- `-a, --all`: show every entry (ignores `--limit`)

Output per entry:

```
[N] <filename>
    Timestamp: <ISO 8601 timestamp>
    Parent: <parent filename or (genesis)>
    Preview: <first 80 chars of body>...
```

A summary line at the end reports `Showing X of Y entries.`.

## gitehr journal cat

```text
gitehr journal cat [-r|--reverse]
```

Prints the full body of every journal entry in chronological order. Use this to read the record end to end, or for piping into other tools.

Options:

- `-r, --reverse`: newest first

Each entry is preceded by a header line of the form:

```
--- Entry N | <timestamp> | author: <author> ---
<filename>
```

A final line reports the total entry count: `(<N> entries)`.

## gitehr journal verify

```text
gitehr journal verify
```

Walks the journal from genesis to the latest entry and confirms the tamper-evident chain is intact.

Behavior:

- Sorts every file in `journal/` by filename (chronological).
- For each non-genesis entry, recomputes its parent's SHA-256 and confirms it matches the declared `parent_hash`, and that `parent_entry` matches the expected filename.
- Reports any broken link, then exits with a non-zero status on failure.

On success, prints `Journal verification successful: N entries verified.`.
