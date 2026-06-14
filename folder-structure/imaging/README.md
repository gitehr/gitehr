# Imaging

This directory contains imaging-related clinical Documents and metadata. The content should preferably be stored in open, non-proprietary formats. The clinical information should be imported into structured data in the journal where possible, with a link to the source Document. These source Documents serve as reference, provenancing, and backups.

Imaging is added with `gitehr document add --imaging`. A multi-file study (e.g. a DICOM series) is stored as a directory Document named `YYYY-MM-DD-<slug>-<hash8>/` containing a `manifest.json` that hashes every file in the study; the journal entry references the directory and the manifest's SHA-256. Documents are immutable and write-once; integrity can be checked with `gitehr document verify`.

Examples of content:

- DICOM files
- Imaging reports
- Scan metadata
- Image analysis results
