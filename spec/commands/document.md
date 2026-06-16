<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr document`

Manage clinical source [Documents](../../CONTEXT.md) - PDFs, scanned letters, photographs, Word documents, and imaging studies (e.g. DICOM). A Document is stored as an ordinary file under `documents/` (or `imaging/`), or as a directory anchored by a hash manifest for multi-file studies. Documents are immutable and write-once, and are linked from journal entries via the `documents:` front-matter field. The design rationale is recorded in [ADR-0001](../adr/0001-documents-as-plain-files.md), [ADR-0002](../adr/0002-record-only-grows.md), and [ADR-0003](../adr/0003-directory-documents-with-manifest.md).

Alias: `gitehr attach` is a hidden alias for `gitehr document` retained during transition.

All subcommands require the current directory to be a GitEHR repository (presence of `.gitehr`).

### `gitehr document add <path> [OPTIONS]`

Adds a file or directory to the record as a Document and creates a new journal entry that references it.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `path` | Path to the file (or directory, e.g. a DICOM study) to add |

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--imaging` | | Store under `imaging/` instead of `documents/` |
| `--title <title>` | `-t` | Title used to build the stored filename slug (defaults to the original filename) |
| `--message <text>` | `-m` | Journal entry text describing the Document (defaults to `Added Document: <original filename>`) |

**Behavior:**

- Computes the SHA-256 of the source. For a single file this hashes the file bytes; for a directory it builds a `manifest.json` listing the SHA-256 of every file within, and hashes the manifest bytes (a shallow Merkle anchor).
- Copies the source into `documents/` (or `imaging/` with `--imaging`) under the name `YYYY-MM-DD-<slug>-<first-8-hex-of-sha256>.<ext>`. The date is UTC; the slug is derived from `--title` or the original filename. Directory Documents use the same naming scheme without an extension and gain a `manifest.json`.
- The hash suffix makes name collisions between offline contributors cryptographically impossible. Adding identical content on the same day yields the same name and is rejected (Documents are write-once).
- Stages the stored Document with `git add`, then creates a journal entry whose front matter records the reference under `documents:` (`path`, `sha256`, `original_filename`).
- Prints the stored path and SHA-256.

**Examples:**

```bash
# Add a PDF, titling the stored file
gitehr document add ~/Downloads/Scan0001.pdf --title "CT head report"
# -> documents/2026-06-14-ct-head-report-7f3a9c1b.pdf

# Add a clinical photograph under imaging/
gitehr document add ./knee.jpg --imaging -t "Left knee" -m "Photo taken in clinic"

# Add a multi-file DICOM study (becomes a directory Document with a manifest)
gitehr document add ./ct-study/ --imaging -t "CT head"
```

### `gitehr document list`

Lists Documents referenced by journal entries, with the SHA-256, original filename, and how many journal entries reference each. Also reports any files present under `documents/`/`imaging/` that are not referenced by any journal entry.

**Behavior:**

- The list is derived by scanning journal front matter; there is no stored index. Reverse lookup (which entries reference a Document) is always computed, never persisted.

### `gitehr document info <path>`

Shows which journal entries reference the Document at `path`, plus its recorded SHA-256 and original filename. If the Document has been removed from the working tree but is still referenced, this is reported (the bytes remain in Git history).

### `gitehr document verify [path]`

Verifies Document integrity against the hashes recorded in journal entries. With no argument, verifies every referenced Document; with a `path`, verifies just that one.

**Behavior:**

- For a file Document, recomputes the SHA-256 and compares it to the recorded hash.
- For a directory Document, checks that the `manifest.json` hashes to the recorded value, that every manifest entry matches its file, and that no unlisted files have been added (write-once).
- A Document that has been removed from the working tree is reported as `MISSING` but is **not** a failure - deletion only ever touches the working tree, and Git history retains every Document (see [ADR-0002](../adr/0002-record-only-grows.md)).
- Exits non-zero if any integrity failure (hash mismatch, manifest tampering, or an unlisted file in a directory Document) is found.

## Document Data Model

- A Document reference (`DocumentRef`) carries `path` (the stored location within the record), `sha256` (the verifiability proof), and optional `original_filename` (see [src/commands/document.rs](../../src/commands/document.rs)).
- The `sha256` of a single-file Document is the hash of its bytes; for a directory Document it is the hash of the `manifest.json`, which transitively anchors every file in the directory (see [src/commands/document.rs](../../src/commands/document.rs)).
- Documents are immutable and write-once. "Updating" a Document means adding a new one and referencing it from a new journal entry; an erroneous Document is corrected by a later entry marking it as entered in error, not by deletion or replacement.
