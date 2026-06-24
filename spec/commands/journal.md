<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr journal`

Aliases:

### `gitehr journal add [content] [OPTIONS]`

Appends a new clinical journal entry containing the provided content.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `content` | The content to add to the journal (optional if using --file) |

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--file <path>` | `-f` | Read content from a file (use `-` for stdin) |

**Input modes:**

1. **Inline content**: `gitehr journal add "Your clinical note here"`
2. **File input**: `gitehr journal add --file /path/to/note.md`
3. **Stdin**: `cat note.md | gitehr journal add --file -`

**Examples:**

```bash
# Add inline content
gitehr journal add "Patient presented with fever. Prescribed paracetamol."

# Add from a file
gitehr journal add --file ~/notes/consultation.md

# Add from stdin (useful for piping)
echo "Quick note about follow-up" | gitehr journal add --file -

# Add multi-line content from heredoc
gitehr journal add --file - << 'EOF'
## Consultation Notes

Patient reports improvement in symptoms.
Continue current medication.
EOF
```

**Behavior:**

- Requires the current directory to already be a GitEHR repository (presence of `.gitehr`).
- Must provide either inline content OR --file, but not both (and not neither).
- Determines the most recent journal entry by filename ordering. If found, calculates its SHA-256 hash and sets that as the new entry's `parent_hash`.
- Creates a new Markdown file named `journal/<timestamp>-<uuid>.md`.
- Prepends YAML front matter containing `parent_hash`, `parent_entry`, the creation timestamp, and (currently optional) `author`.
- Runs `git add` on the new file and creates a git commit with message `Journal entry: <filename>`.
- Prints the created filename on success.

### `gitehr journal show [OPTIONS]`

Lists journal entries in chronological order (oldest first by default), with optional pagination.

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--limit <N>` | `-n` | Maximum number of entries to display (default: 10) |
| `--offset <N>` | `-o` | Number of entries to skip from the start (default: 0) |
| `--reverse` | `-r` | Show newest entries first instead of oldest first |
| `--all` | `-a` | Show all entries (ignores --limit) |

**Output format:**

Each entry is displayed as:
```
[N] <filename>
    Timestamp: <ISO 8601 timestamp>
    Parent: <parent_entry or "(genesis)">
    Preview: <first 80 characters of content>...
```

**Examples:**

```bash
# Show first 10 entries (default)
gitehr journal show

# Show 20 entries starting from the 10th
gitehr journal show --limit 20 --offset 10

# Show all entries, newest first
gitehr journal show --all --reverse

# Show just the 5 most recent entries
gitehr journal show -n 5 -r
```

**Behavior:**

- Requires the current directory to be a GitEHR repository (presence of `.gitehr`).
- Reads all files from the `journal/` directory and sorts them by filename (which contains the timestamp).
- Parses each entry's YAML front matter to extract metadata for display.
- Truncates content preview at 80 characters to keep output readable.
- Prints a summary line showing "Showing X of Y entries" at the end.

### `gitehr journal cat [OPTIONS]`

Prints the full body of every journal entry in chronological order. This is the "play back the record" view, intended for reading the journal end to end.

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--reverse` | `-r` | Show newest entries first |

**Output format:**

Each entry is rendered as:

```
--- Entry N | <ISO 8601 timestamp> | author: <author or (unknown)> ---
<filename>

<full body of the entry>

```

The final line shows the total entry count: `(<N> entries)`.

**Examples:**

```bash
# Read the whole record, oldest first
gitehr journal cat

# Read the whole record, newest first
gitehr journal cat --reverse
```

**Behavior:**

- Requires the current directory to be a GitEHR repository.
- Reads every file in `journal/` whose name matches the timestamp-uuid pattern.
- Sorts by filename (chronological by virtue of the timestamp prefix).
- Skips entries with invalid front matter, printing a one-line error on stderr but continuing.

### `gitehr journal verify`

Validates the integrity of the journal chain (see [src/commands/verify.rs](../../src/commands/verify.rs) and [src/main.rs](../../src/main.rs)).

- Requires a GitEHR repository and the existence of the `journal` directory; otherwise, it returns an error.
- Sorts all journal files by filename, computes each entry’s SHA-256 hash, and builds a map of hash → filename for lookup.
- For each entry, parses YAML front matter into a `JournalEntry`; errors if the front matter is missing or invalid.
- For non-genesis entries, ensures the declared `parent_hash` exists in the map and the recorded `parent_entry` matches the expected filename; otherwise, it reports a broken chain or missing parent.
- On success, prints “Journal verification successful: N entries verified.”

> TODO: Front matter parsing currently assumes the YAML is delimited by `---` and that non-genesis entries include `parent_entry`. If this changes, update verification logic accordingly.

TODO: gitehr journal verify needs an option for increased verbosity to show details of any verification failures (e.g., which entry is broken, expected vs actual parent hash/entry). This will be crucial for debugging integrity issues in the journal chain.

## Entry references

Commands that act on a single journal entry accept a **relative entry reference** anywhere a filename is accepted, so you rarely need to copy a full `<timestamp>-<uuid>.md` filename. The reference is resolved by `resolve_entry` (see [src/commands/journal/mod.rs](../../cli/src/commands/journal/mod.rs)) and is used by `gitehr journal show` and `gitehr journal commit`.

| Expression | Meaning |
|---|---|
| `LATEST` | the most recent entry |
| `LATEST^` | one entry older than the most recent (the second most recent) |
| `LATEST^^^^` | N carets = N entries older than the most recent |
| `LATEST~N` | N entries older than the most recent |
| `<filename>^` | one entry older than the named file |
| `<filename>~N` | N entries older than the named file |

Rules:

- The offset always moves toward **older** entries (back in time). `~N` and a run of `^` carets are equivalent: `LATEST~3` is the same as `LATEST^^^`.
- A bare filename with no `^` or `~N` suffix is used as-is (it is not resolved against the entry list).
- The reference resolves against committed entries by default, or against drafts when `--drafts` is given. `gitehr journal commit` always resolves against drafts, since it operates on `tmp/journal/`.
- Resolution fails with a clear error when there are no entries (`no entries found`), when a named anchor file is not in the set (`entry not found`), or when the offset runs past the oldest entry (`out of range`).

### Examples

```bash
gitehr journal show LATEST                            # most recent committed entry
gitehr journal show LATEST^^^                          # three entries before the most recent
gitehr journal show LATEST~10                          # ten entries back
gitehr journal show 20260619T143012.123Z-<uuid>.md^    # one entry older than the named file
gitehr journal commit LATEST                           # commit the most recent draft
gitehr journal show --drafts LATEST^                   # the second most recent draft
```

## Journal Data Model

- Each entry file starts with YAML front matter representing `JournalEntry` with fields `parent_hash` (optional), `parent_entry` (optional for genesis), `timestamp` (UTC), `author` (optional, automatically set to the currently active user ID via `gitehr user activate <id>`), and `documents` (optional, a list of references to [Documents](document.md) this entry relates to, each with `path`, `sha256`, and optional `original_filename`) (see [src/commands/journal.rs](../../src/commands/journal.rs)).

- The file content after the front matter holds the clinical narrative or data supplied to `gitehr add` or the genesis message (see [src/commands/journal.rs](../../src/commands/journal.rs) and [src/commands/init.rs](../../src/commands/init.rs)).

- File naming embeds chronological ordering and uniqueness via timestamp and UUID, enabling simple sorting to reconstruct history (see [src/commands/journal.rs](../../src/commands/journal.rs)).
