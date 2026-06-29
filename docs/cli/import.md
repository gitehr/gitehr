# gitehr import

Bring records **into** a GitEHR repository from a file or directory - either well-formed journal entries from another GitEHR instance, or arbitrary documents (scans, letters, photos) that each become a journal entry. The repository must be the current directory (presence of `.gitehr/`).

```text
gitehr import --mode <journal|documents> <path>
```

`<path>` may be a single file or a directory. Directories are walked **recursively**; hidden files and directories (`.git`, dotfiles) are skipped. Files that don't match the chosen mode are skipped, and a summary count is printed at the end.

## --mode journal

Imports well-formed GitEHR journal entries **verbatim**. Each entry keeps its original filename - and therefore its timestamp, UUID, author, and full provenance - so a record carries across instances unchanged.

- Files that are not valid journal entries are skipped.
- An entry whose filename already exists in `journal/` is skipped (treated as already-imported), so re-running is safe (idempotent).
- Each imported entry is staged and committed (`Import journal entry: <filename>`).

```bash
# Import every entry from a colleague's exported journal directory
gitehr import --mode journal ~/from-the-gp/journal

# Import a single entry file
gitehr import --mode journal ./20260614T101500Z-1a2b3c4d-....md
```

## --mode documents

Imports documents of **any** format. Each file is copied into `documents/` and given its own journal entry whose body is just a markdown link to the document - there is no `documents:` front matter, so the entry is a lightweight pointer the GUI can choose to follow.

- Any file format is accepted (no filtering yet).
- A file already present in `documents/` is skipped.

```bash
# Bulk-import a folder of scanned letters and photos
gitehr import --mode documents ~/scans/

# Import one document
gitehr import --mode documents ./discharge-summary.pdf
```

## When to use this vs `gitehr document add`

[`gitehr document add`](document.md) is the careful Document path: it content-hashes each file, deduplicates, and records structured `documents:` references (`path`, `sha256`, `original_filename`) in the journal entry's front matter. Reach for it when provenance and integrity verification matter.

`gitehr import --mode documents` is the **bulk, low-friction** path: point it at a folder and every file lands as a document with a simple linking entry. It is built for getting a pile of records in quickly, not for per-document curation.
