# gitehr journal

The append-only journal of clinical entries. Each entry is an immutable record event - YAML front matter plus a Markdown body - written as `journal/<timestamp>-<uuid>.md` and recorded as its own git commit. Ordering and tamper-evidence come from the git history.

Repo-level: run inside a subject's repo. In a single-subject Store, `gitehr` auto-targets it, so you can run these from the Store root too.

## gitehr journal add

```text
gitehr journal add [<text>] [--file <path>]
```

Add an entry. The body comes from, in order of precedence: `--file <path>` (or `--file -` for stdin), the inline `<text>` argument, piped stdin, or - on a terminal with none of those - your `$EDITOR`. An empty body aborts. The entry is written and git-committed immediately.

```bash
gitehr journal add "Patient reviewed; plan updated."   # inline
gitehr journal add --file note.md                       # from a file
cat note.md | gitehr journal add --file -               # explicit stdin
echo "Quick note" | gitehr journal add                  # piped stdin
gitehr journal add                                      # opens $EDITOR
```

Inline text and `--file` are mutually exclusive. The recorded `author` is the contributor set by `gitehr user activate <id>` (absent if none is active).

## gitehr journal list

```text
gitehr journal list
```

Aliases: `list-entry`, `ls`. Lists entry filenames, oldest first, with a `(N entries)` count.

## gitehr journal show

```text
gitehr journal show <entry> [--raw | --metadata]
```

Alias: `cat`. Prints a single entry: by default just the body, `--raw` the whole file (including front matter), `--metadata` only the front matter. `<entry>` is a filename or a relative reference (below).

```bash
gitehr journal show LATEST            # body of the most recent entry
gitehr journal show LATEST^ --raw     # the previous entry, full file
```

## Entry references

Anywhere an entry is named, you can use a relative reference instead of a full `<timestamp>-<uuid>.md` filename:

| Expression | Meaning |
|---|---|
| `LATEST` | the most recent entry |
| `LATEST^`, `LATEST^^^` | N carets = N entries older than the most recent |
| `LATEST~N` | N entries older than the most recent |
| `<filename>^`, `<filename>~N` | older than the named file |

The offset always moves toward **older** entries; `LATEST~3` equals `LATEST^^^`. A bare filename with no suffix is used as-is.

## Data model

Each entry is YAML front matter followed by a Markdown body. The front matter (`JournalEntry`) holds `timestamp` (UTC), optional `author`, and optional `documents` - references to [Documents](document.md), each with `path`, `sha256`, and optional `original_filename`. The `<timestamp>-<uuid>.md` filename sorts chronologically, so a filename sort reconstructs history. Tamper-evidence derives from Git history, not a per-entry front-matter chain.
