<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr journal`

Create, commit, list, and read journal entries. A journal entry is a single, immutable clinical record event. Every subcommand requires the current directory to be a GitEHR repository (presence of `.gitehr`).

Writing an entry is a two-step flow: `new-entry` opens a draft in your editor, and `commit` finalises that draft into the immutable `journal/`. This keeps half-written notes out of the permanent record until they are deliberately committed.

### `gitehr journal new-entry`

Aliases: `new`, `touch`.

Creates an empty draft and opens it in your editor.

- Writes an empty file `tmp/journal/draft-<timestamp>-<uuid>.md`.
- Opens it in `$EDITOR` (falling back to `$VISUAL`, then `vi`).
- On a successful editor exit, prints `Draft saved: <path>`. The draft is not part of the record until it is committed.

### `gitehr journal commit <entry>`

Finalises a draft into the immutable journal.

`<entry>` is a draft filename relative to `tmp/journal/`, an absolute path, or a [relative entry reference](#entry-references) such as `LATEST` (resolved against drafts).

- Reads the draft and prepends YAML front matter: `timestamp`, `author` (the currently active contributor, if any), and optional `documents`.
- Writes `journal/<timestamp>-<uuid>.md` and removes the draft from `tmp/journal/`.
- Runs `git add` and `git commit` with the message `Journal entry: <filename>`.
- Prints `Committed: <filename>`.

```bash
gitehr journal commit LATEST                 # commit the most recent draft
gitehr journal commit draft-20260619T...-<uuid>.md
```

### `gitehr journal list-entry [--drafts]`

Aliases: `list`, `ls`.

Lists journal entry filenames, one per line, sorted oldest-first, followed by a count (`(N entries)`).

- `--drafts` lists draft filenames in `tmp/journal/` instead, with a `(N drafts)` count.
- Prints `No journal entries found.` (or `No drafts found.`) when there are none.

### `gitehr journal show <entry> [OPTIONS]`

Alias: `cat`.

Shows a single journal entry. `<entry>` is a filename or a [relative entry reference](#entry-references) such as `LATEST` or `LATEST^`.

| Option | Description |
|--------|-------------|
| `--drafts` | Resolve and read from drafts in `tmp/journal/` instead of committed entries |
| `--raw` | Print the raw file, including the YAML front matter |
| `--metadata` | Print only the YAML front matter block |

By default (no flags) it prints just the entry body (the clinical narrative). `--raw` prints the whole file; `--metadata` prints only the front matter. The two flags are mutually exclusive in practice (`--raw` takes precedence).

```bash
gitehr journal show LATEST            # body of the most recent entry
gitehr journal show LATEST^ --raw     # the previous entry, full file with front matter
gitehr journal show --drafts LATEST   # the most recent draft's body
```

### Integrity

There is no `gitehr journal verify` subcommand. Each committed entry is its own git commit, so the journal's history, ordering, and tamper-evidence derive from the underlying git history rather than from a per-entry hash chain in the front matter. (An earlier `parent_hash`/`parent_entry` chain was removed; see [Planned refinements](#planned-refinements).)

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

- Each entry file is YAML front matter followed by a Markdown body. The front matter is a `JournalEntry` with `timestamp` (UTC), `author` (optional, set from the currently active contributor via `gitehr user activate <id>`), and `documents` (optional, a list of references to [Documents](document.md) this entry relates to, each with `path`, `sha256`, and optional `original_filename`). See [cli/src/commands/journal/mod.rs](../../cli/src/commands/journal/mod.rs).
- The body after the front matter holds the clinical narrative or data written in the draft (or the genesis message created by `gitehr init`).
- File naming embeds chronological ordering and uniqueness via the timestamp and UUID, so a simple filename sort reconstructs history.

## Planned refinements

- **Genesis without a false-genesis claim.** GitEHR has dropped the per-entry `parent_hash`/`parent_entry` linkage (chaining is derived from git history instead). A future refinement is to embed a random seed in the genesis entry's content together with a URL to an external genesis-registration record, so that the seed plus a registered, timestamped registration makes it computationally hard for anyone to fabricate an earlier "first" entry and backdate a false genesis claim.
- **Shorter filename uniqueness token.** The `journal/<timestamp>-<uuid>.md` filename pairs a millisecond timestamp with a full UUID. Because the millisecond timestamp already provides strong uniqueness, the UUID could be shortened to a fragment of a hash or a short random suffix for shorter, more readable filenames.
