<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr import`

A way to import files between GitEHR instances. There are multiple modes.

+ `--mode journal` should handle well-formed GitEHR journal entries.
  - Imported entries are preserved **verbatim**: the original filename, timestamp, author, and UUID are kept (provenance is carried across instances). If an entry with the same filename already exists, it is skipped (treated as already-imported). Each imported entry is committed on its own, without sweeping in unrelated staged changes. If copying, staging or committing fails, the copied entry is unstaged and removed, so a retry is not skipped as already-imported.
+ `--mode documents` should import scanned documents in any file format. Each document goes into the documents folder, and gets a journal entry containing only a reference (a Markdown link, `/documents/<filename>`) to one document - body link only, with no `documents:` frontmatter metadata. The GUI decides whether to follow Markdown links when there is no other content. The document file and its journal entry are committed together in a single commit, so history never holds an entry whose document is missing. If copying, staging or committing fails, both are unstaged and removed. A destination occupied by anything, including a dangling symlink, counts as already present.
  - Any file format is accepted by default. When `document_whitelist` is set in the machine config ([`spec/commands/config.md`](config.md)), only files whose extension (case-insensitive) appears in it are imported; files with no extension never match a configured whitelist. Non-matching files are skipped and counted, same as other skip reasons. An unusable whitelist (empty, or containing an entry that is not a bare extension) fails the import with a configuration error instead of skipping every file. The whitelist governs `--mode documents` only; `gitehr document add` is unaffected.
+ Each mode should receive a file or directory. A directory is walked **recursively**; hidden files and directories are ignored and symbolic links are not followed. Files that don't match the mode are skipped and a summary count is reported: non journal-entry filenames for `journal` are skipped silently, while a journal-entry filename that fails to parse is skipped with a warning.

We can add other modes later, like an imaging-scanned mode, but for now just the journal and documents mode.
