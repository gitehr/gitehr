# Documents

This directory contains non-imaging clinical Documents and metadata. The content should preferably be stored in open, non-proprietary formats. The clinical information should be imported into structured data in the journal where possible, with a link to the source Document. These source Documents serve as reference, provenancing, and backups.

Documents are added with `gitehr document add`, which stores each file as `YYYY-MM-DD-<slug>-<hash8>.<ext>` and records a reference (path and SHA-256) in a journal entry. Documents are immutable and write-once; integrity can be checked with `gitehr document verify`.

Examples of content:

- PDF medical reports
- Correspondence
- Lab results
- Referral letters
