<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr journal`

Add, list, and read journal entries. A journal entry is a single, immutable clinical record event. Every subcommand requires the current directory to be a GitEHR repository (presence of `.gitehr`).

### `gitehr journal add [<text>] [--file <path>]`

Add a new entry in one step. The body comes from, in order of precedence: `--file <path>` (or `--file -` for stdin), the inline `<text>` argument, piped stdin, or - on a terminal with none of those - the user's `$EDITOR` (falling back to `$VISUAL`, then `vi`) on an ephemeral temp file. An empty body aborts.

- Prepends YAML front matter: `timestamp`, `author` (the currently active contributor, if any), and optional `documents`.
- Writes `journal/<timestamp>-<uuid>.md`, then `git add` + `git commit` with message `Journal entry: <filename>`.
- Inline `<text>` and `--file` are mutually exclusive (enforced by clap).

```bash
gitehr journal add "Patient reviewed; plan updated."
gitehr journal add --file note.md
cat note.md | gitehr journal add --file -
echo "Quick note" | gitehr journal add        # piped stdin
gitehr journal add                             # opens $EDITOR
```

### `gitehr journal list-entry`

Aliases: `list`, `ls`.

Lists journal entry filenames, one per line, sorted oldest-first, followed by a count (`(N entries)`). Prints `No journal entries found.` when there are none.

### `gitehr journal show <entry> [OPTIONS]`

Alias: `cat`.

Shows a single journal entry. `<entry>` is a filename or a [relative entry reference](#entry-references) such as `LATEST` or `LATEST^`.

| Option | Description |
|--------|-------------|
| `--raw` | Print the raw file, including the YAML front matter |
| `--metadata` | Print only the YAML front matter block |

By default (no flags) it prints just the entry body (the clinical narrative). `--raw` prints the whole file; `--metadata` prints only the front matter. The two flags are mutually exclusive in practice (`--raw` takes precedence).

```bash
gitehr journal show LATEST            # body of the most recent entry
gitehr journal show LATEST^ --raw     # the previous entry, full file with front matter
```

### Integrity

There is no journal-specific verification subcommand. Each committed entry is its own Git commit, so the journal's history, ordering, and tamper-evidence derive from the underlying Git history rather than from a per-entry front-matter chain. A future repository policy checker may enforce the append-only and authorship invariants described in [`repository-verification.md`](../repository-verification.md).

## Entry references

Commands that act on a single journal entry accept a **relative entry reference** anywhere a filename is accepted, so you rarely need to copy a full `<timestamp>-<uuid>.md` filename. The reference is resolved by `resolve_entry` (see [src/commands/journal/mod.rs](../../cli/src/commands/journal/mod.rs)) and is used by `gitehr journal show`.

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
- The reference resolves against the committed entries in `journal/`.
- Resolution fails with a clear error when there are no entries (`no entries found`), when a named anchor file is not in the set (`entry not found`), or when the offset runs past the oldest entry (`out of range`).

### Examples

```bash
gitehr journal show LATEST                            # most recent committed entry
gitehr journal show LATEST^^^                          # three entries before the most recent
gitehr journal show LATEST~10                          # ten entries back
gitehr journal show 20260619T143012.123Z-<uuid>.md^    # one entry older than the named file
```

## Journal Data Model

- Each entry file is YAML front matter followed by a Markdown body. The front matter is a `JournalEntry` with `timestamp` (UTC), `author` (optional, set from the currently active contributor via `gitehr user activate <id>`), and `documents` (optional, a list of references to [Documents](document.md) this entry relates to, each with `path`, `sha256`, and optional `original_filename`). See [cli/src/commands/journal/mod.rs](../../cli/src/commands/journal/mod.rs).
- The body after the front matter holds the clinical narrative or data supplied when the entry was added.
- File naming embeds chronological ordering and uniqueness via the timestamp and UUID, so a simple filename sort reconstructs history.

## Planned refinements

- **Genesis without a false-genesis claim.** GitEHR has dropped per-entry front-matter linkage; tamper-evidence derives from Git history instead. A future refinement is to embed a random seed in the genesis entry's content together with a URL to an external genesis-registration record, so that the seed plus a registered, timestamped registration makes it computationally hard for anyone to fabricate an earlier "first" entry and backdate a false genesis claim.
- **Shorter filename uniqueness token.** The `journal/<timestamp>-<uuid>.md` filename pairs a millisecond timestamp with a full UUID. Because the millisecond timestamp already provides strong uniqueness, the UUID could be shortened to a fragment of a hash or a short random suffix for shorter, more readable filenames.
