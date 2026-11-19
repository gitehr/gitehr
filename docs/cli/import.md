# gitehr import

Bring records **into** a GitEHR repository from a file or directory - either well-formed journal entries from another GitEHR instance, or arbitrary documents (scans, letters, photos) that each become a journal entry. Like other repository commands, it targets the nearest enclosing GitEHR repository (the nearest ancestor containing `.gitehr/`), or the sole subject when run at a Store root; a relative `<path>` is resolved against the directory you ran the command from.

```text
gitehr import --mode <journal|documents> <path>
```

`<path>` may be a single file or a directory. Directories are walked **recursively**; hidden files and directories (`.git`, dotfiles) are ignored, and symbolic links are not followed. Files that don't match the chosen mode are skipped, and a summary count of imported and skipped files is printed at the end (ignored hidden files and symlinks are not counted). If copying a file fails, the import stops with an error; files imported before that point stay imported and committed.

## --mode journal

Imports well-formed GitEHR journal entries **verbatim**. Each entry keeps its original filename - and therefore its timestamp, UUID, author, and full provenance - so a record carries across instances unchanged.

- Files whose names are not journal-entry filenames are skipped silently; files with a journal-entry filename that fail to parse are skipped with a warning on stderr.
- An entry whose filename already exists in `journal/` is skipped (treated as already-imported), so re-running is safe (idempotent).
- Each imported entry is committed on its own (`Import journal entry: <filename>`); other changes you have staged are not swept into that commit.

```bash
# Import every entry from a colleague's exported journal directory
gitehr import --mode journal ~/from-the-gp/journal

# Import a single entry file
gitehr import --mode journal ./20260614T101500Z-1a2b3c4d-....md
```

## --mode documents

Imports documents of **any** format. Each file is copied into `documents/` and given its own journal entry whose body is just a markdown link to the document - there is no `documents:` front matter, so the entry is a lightweight pointer the GUI can choose to follow.

- Any file format is accepted by default. Set `document_whitelist` in [`gitehr config`](config.md) to restrict imports to specific extensions (case-insensitive); files that don't match are skipped and counted like any other skip reason. An unusable whitelist fails the import with a configuration error rather than skipping everything. The whitelist applies to this mode only, not to [`gitehr document add`](document.md).
- Files are copied flat into `documents/` under their original filename, so a file whose name is already present in `documents/` is skipped - even if its content differs, or it came from a different subdirectory of `<path>`.
- Each document is committed together with its linking journal entry, in one commit per document.

```bash
# Bulk-import a folder of scanned letters and photos
gitehr import --mode documents ~/scans/

# Import one document
gitehr import --mode documents ./discharge-summary.pdf
```

## When to use this vs `gitehr document add`

[`gitehr document add`](document.md) is the careful Document path: it content-hashes each file, deduplicates, and records structured `documents:` references (`path`, `sha256`, `original_filename`) in the journal entry's front matter. Reach for it when provenance and integrity verification matter.

`gitehr import --mode documents` is the **bulk, low-friction** path: point it at a folder and every file lands as a document with a simple linking entry. It is built for getting a pile of records in quickly, not for per-document curation.
