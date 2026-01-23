<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr journal`

Aliases:

### `gitehr journal add <content>`

Appends a new clinical journal entry containing the provided content.

- Requires the current directory to already be a GitEHR repository (presence of `.gitehr`); otherwise, the command aborts with guidance to run `gitehr init` (see [src/main.rs](../../src/main.rs)).
- Determines the most recent journal entry by filename ordering (timestamps in filenames). If found, calculates its SHA-256 hash and sets that as the new entry’s `parent_hash`; the matching filename is stored as `parent_entry` (see [src/main.rs](../../src/main.rs) and [src/commands/journal.rs](../../src/commands/journal.rs)).
- Creates a new Markdown file named `journal/<timestamp>-<uuid>.md` using the current UTC time down to milliseconds plus a random UUID (see [src/commands/journal.rs](../../src/commands/journal.rs)).
- Prepends YAML front matter containing `parent_hash`, `parent_entry`, the creation timestamp, and (currently optional) `author`, followed by the user-provided content. Prints the created filename on success (see [src/commands/journal.rs](../../src/commands/journal.rs)).

### `gitehr journal verify`

Validates the integrity of the journal chain (see [src/commands/verify.rs](../../src/commands/verify.rs) and [src/main.rs](../../src/main.rs)).

- Requires a GitEHR repository and the existence of the `journal` directory; otherwise, it returns an error.
- Sorts all journal files by filename, computes each entry’s SHA-256 hash, and builds a map of hash → filename for lookup.
- For each entry, parses YAML front matter into a `JournalEntry`; errors if the front matter is missing or invalid.
- For non-genesis entries, ensures the declared `parent_hash` exists in the map and the recorded `parent_entry` matches the expected filename; otherwise, it reports a broken chain or missing parent.
- On success, prints “Journal verification successful: N entries verified.”

> TODO: Front matter parsing currently assumes the YAML is delimited by `---` and that non-genesis entries include `parent_entry`. If this changes, update verification logic accordingly.

<!-- - REVIEW/ADD **Adding entries:** `gitehr add` appends immutable journal files that link back to the latest entry, preserving a verifiable chain of custody and chronological ordering (see [src/main.rs](../../src/main.rs) and [src/commands/journal.rs](../../src/commands/journal.rs)). -->

## Journal Data Model

- Each entry file starts with YAML front matter representing `JournalEntry` with fields `parent_hash` (optional), `parent_entry` (optional for genesis), `timestamp` (UTC), and `author` (optional, reserved for future identity management) (see [src/commands/journal.rs](../../src/commands/journal.rs)).

- The file content after the front matter holds the clinical narrative or data supplied to `gitehr add` or the genesis message (see [src/commands/journal.rs](../../src/commands/journal.rs) and [src/commands/init.rs](../../src/commands/init.rs)).

- File naming embeds chronological ordering and uniqueness via timestamp and UUID, enabling simple sorting to reconstruct history (see [src/commands/journal.rs](../../src/commands/journal.rs)).
