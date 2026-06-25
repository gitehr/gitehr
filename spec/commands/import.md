<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr import`

A way to import files between gitehr instances. There are multiple modes.

+ `--mode journal` should handle well formed gitehr journal entries.
  - Imported entries are preserved **verbatim**: the original filename, timestamp, author, and UUID are kept (provenance is carried across instances). If an entry with the same filename already exists, it is skipped (treated as already-imported).
+ `--mode documents` should import scanned documents in any file format. Each document goes into the documents folder, and gets a journal entry containing only a reference (just a markdown directory/link, /documents/<filename>) to one document - body link only, with no `documents:` frontmatter metadata. It is up to the GUI implementation to decide if they should follow markdown links when there is no other content, so we wont handle that.
  - For now any file format is accepted (no whitelist filtering).
  - TODO: Add a comment to the "config file?" issue saying that supported file formats should be handled in the config file. Once that exists, `--mode documents` should filter against the configured whitelist.
+ Each mode should receive a file or directory. A directory is walked **recursively**; files that don't match the mode (non journal-entry files for `journal`) are silently skipped, and a summary count is reported.

We can add other modes later, like an imaging-scanned mode, but for now just the journal and documents mode.
