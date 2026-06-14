# Repository Structure

GitEHR stores the complete patient record as a folder on disk. The layout is designed for long-term readability and portability.

## Top-level layout

```
.
├── .gitehr/
├── documents/
├── imaging/
├── journal/
└── state/
```

## .gitehr

Internal configuration and metadata used by the CLI and GUI.

Common files include:

- `GITEHR_VERSION` - repository version string
- `gitehr` - bundled CLI binary
- `contributors.json` - contributor roster and active author
- `remotes.json` - named remotes
- `ENCRYPTED` - encryption marker (placeholder)

## journal/

Append-only clinical entries in markdown.

Entry naming:

```
journal/YYYYMMDDTHHMMSS.mmmZ-<UUID>.md
```

Each file contains YAML front matter followed by the clinical note body.

Typical metadata fields:

- `parent_hash`
- `parent_entry`
- `timestamp`
- `author` (optional)
- `documents` (optional) - references to Documents this entry relates to:

```yaml
documents:
  - path: documents/2026-06-07-ct-head-report-7f3a9c1b.pdf
    sha256: 7f3a9c...
```

## state/

Mutable, current clinical summaries. Examples:

- `allergies.md`
- `medications.md`
- `demographics.md`
- `about-me.md`

State files are expected to change over time and are designed to be easy to read.

## documents/

Clinical source Documents: PDFs, scanned letters, care plans, Word documents, and other supporting files. Documents are immutable and write-once - the SHA-256 recorded in each referencing journal entry is a verifiability proof, and "updating" a Document means adding a new one (see ADR-0001 and ADR-0002).

On add, the CLI names each Document:

```
documents/YYYY-MM-DD-<descriptive-slug>-<first-8-hex-of-sha256>.<ext>
```

The original filename is preserved in the referencing journal entry's metadata. The hash suffix makes name collisions between offline contributors impossible.

## imaging/

Imaging Documents: DICOM studies, scans, and photographs, stored in full. A multi-file study is a directory Document named with the same date-slug-hash scheme, containing a manifest that lists the SHA-256 of every file within it; journal entries reference the directory path plus the manifest's hash (see ADR-0003).

## Notes on portability

Because the record is a normal directory, it can be synced and transported using standard tools. The CLI also provides `gitehr transport create` and `gitehr transport extract` for packaging and transfer.
