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

## state/

Mutable, current clinical summaries. Examples:

- `allergies.md`
- `medications.md`
- `demographics.md`
- `about-me.md`

State files are expected to change over time and are designed to be easy to read.

## documents/

Long-form documents and attachments. Use for PDFs, care plans, and other supporting files.

## imaging/

Imaging and binary data, such as scans or photographs.

## Notes on portability

Because the record is a normal directory, it can be synced and transported using standard tools. The CLI also provides `gitehr transport create` and `gitehr transport extract` for packaging and transfer.
