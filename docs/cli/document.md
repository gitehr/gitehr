# gitehr document

Manage clinical source **Documents**: PDFs, scanned letters, photographs, Word documents, and imaging studies such as DICOM. A Document is stored as an ordinary file under `documents/` (or `imaging/`), or as a directory anchored by a hash manifest for multi-file studies. Documents are immutable and write-once, and are linked from journal entries.

`gitehr attach` is a hidden alias for `gitehr document`.

All subcommands require the current directory to be a GitEHR repository.

## gitehr document add

```text
gitehr document add <path>... [--imaging] [-t|--title <title>] [-m|--message <text>]
```

Adds one or more files or directories to the record as Documents and creates one journal entry that references them.

Options:

- `--imaging`: store under `imaging/` instead of `documents/`
- `-t, --title <title>`: title used to build the stored filename (defaults to the original filename; only valid with one path)
- `-m, --message <text>`: journal entry text (defaults to `Added Document: <original filename>` or `Added Documents: <filenames>`)

Behavior:

- Computes the SHA-256 of each source. A single file hashes its bytes; a directory builds a `manifest.json` listing the hash of every file within and hashes that manifest.
- Copies each source into `documents/` (or `imaging/`) as `YYYY-MM-DD-<slug>-<hash8>.<ext>`. The hash suffix makes name collisions between offline contributors impossible.
- Adding identical content on the same day produces the same name and is rejected - Documents are write-once.
- Stages the Document(s) with `git add` and writes one journal entry recording every reference (`path`, `sha256`, `original_filename`) under `documents:` in the front matter.

Examples:

```bash
# Add a PDF with an explicit title
gitehr document add ~/Downloads/Scan0001.pdf --title "CT head report"
# -> documents/2026-06-14-ct-head-report-7f3a9c1b.pdf

# Add a clinical photograph under imaging/
gitehr document add ./knee.jpg --imaging -t "Left knee" -m "Photo taken in clinic"

# Add narrative text with two attached Documents in one journal entry
gitehr document add ./letter.pdf ./clinic-photo.jpg -m "Reviewed letter and attached clinical photo."

# Add a multi-file DICOM study (stored as a directory with a manifest)
gitehr document add ./ct-study/ --imaging -t "CT head"
```

## gitehr document list

```text
gitehr document list
```

Lists Documents referenced by journal entries - SHA-256, original filename, and how many entries reference each - plus any files on disk under `documents/`/`imaging/` that no journal entry references. The list is derived from the journal each time; there is no stored index.

## gitehr document info

```text
gitehr document info <path>
```

Shows which journal entries reference the Document at `<path>`, along with its recorded SHA-256 and original filename. If the Document has been removed from the working tree but is still referenced, that is reported - the bytes remain in Git history.

## gitehr document verify

```text
gitehr document verify [<path>]
```

Verifies Document integrity against the hashes recorded in journal entries. With no argument it checks every referenced Document; with a `<path>` it checks just that one.

- A file Document's bytes are re-hashed and compared to the recorded hash.
- A directory Document is checked end to end: the `manifest.json` must hash to the recorded value, every listed file must match, and no unlisted file may have been added.
- A Document removed from the working tree is reported as `MISSING` but is **not** a failure: deletion only ever touches the working tree, and Git history retains every Document.
- Exits non-zero if any integrity failure is found.
