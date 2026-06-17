# Multi-file Documents are directories anchored by a hash manifest

A DICOM study is hundreds to thousands of files, but a Document reference is a single `path` plus `sha256`. We decided a Document may be a directory (e.g. `imaging/2026-06-07-ct-head-a3f29c1b/`) containing a manifest file that lists the SHA-256 of every file within it; the journal entry references the directory path and the manifest's own sha256, which transitively anchors the whole tree (a shallow Merkle structure). The study stays unpacked and viewable in place by standard tools, and the same mechanism covers any future multi-file artifact.

## Considered Options

- **Archive per study** (one zip, hash the zip): fits the single-file reference format trivially but is opaque on disk and cuts against the human-readable layout. Rejected.
- **One Document per file**: journal entries referencing a study would carry thousands of hash lines and the study would have no single identity. Rejected.
