# Documents are plain files in human-readable folders, linked from journal front matter

GitEHR needs to associate source artifacts (PDFs, scanned letters, photographs, imaging studies) with journal entries. We decided that a Document lives as an ordinary file under `documents/` or `imaging/`, named `YYYY-MM-DD-<descriptive-slug>-<first-8-hex-of-sha256>.<ext>` (the CLI renames on add and records the original filename in the journal entry metadata), and journal entries reference it in YAML front matter under a `documents:` key listing `path` and `sha256`. The canonical term is "Document" everywhere - glossary, CLI (`gitehr document ...`), and file format.

## Considered Options

- **Content-addressed store** (`.gitehr/attachments/ab/cd/<hash>`, as prototyped in `cli/src/commands/attach.rs`): gives deduplication and integrity for free, but the record stops being readable by a human browsing the folder, which defeats GitEHR's longevity and no-lock-in principles. Rejected; the prototype store is to be removed.
- **LFS/annex-style pointer files**: would keep the Git history small, but the record would no longer be self-contained plain files, and a missing object means a dangling pointer in a medical record. Rejected.
- **Keeping original filenames**: collisions between offline contributors become binary merge conflicts, and names like `Scan0001.pdf` carry no meaning. The hash suffix makes collisions cryptographically impossible without sacrificing readability.

## Consequences

Reverse lookup (which entries reference a Document) is always derived by scanning journal front matter, never stored - a stored index would duplicate truth and create merge conflicts. `gitehr verify` extends to check every `documents:` reference: the path exists (or was deliberately removed from the working tree, see ADR-0002), the hash matches, and directory manifests are valid (see ADR-0003).
