<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr import`

A way to import files between gitehr instances. There are multiple modes.

+ `--mode journal` should handle well formed gitehr journal entries.
  - Imported entries are preserved **verbatim**: the original filename, timestamp, author, and UUID are kept (provenance is carried across instances). If an entry with the same filename already exists, it is skipped (treated as already-imported).
+ `--mode documents` should import scanned documents in any file format. Each document goes into the documents folder, and gets a journal entry containing only a reference (just a markdown directory/link, /documents/<filename>) to one document - body link only, with no `documents:` frontmatter metadata. It is up to the GUI implementation to decide if they should follow markdown links when there is no other content, so we wont handle that.
  - Any file format is accepted by default. When `document_whitelist` is set in the machine config ([`spec/commands/config.md`](config.md)), only files whose extension (case-insensitive) appears in it are imported; files with no extension never match a configured whitelist. Non-matching files are skipped and counted, same as other skip reasons.
+ Each mode should receive a file or directory. A directory is walked **recursively**; files that don't match the mode (non journal-entry files for `journal`) are silently skipped, and a summary count is reported.

We can add other modes later, like an imaging-scanned mode, but for now just the journal and documents mode.
