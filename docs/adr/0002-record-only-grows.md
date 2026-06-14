# The record only grows: Documents are immutable, history is never rewritten, full imaging lives in the repo

Documents are immutable and write-once; the SHA-256 recorded in each referencing journal entry is a verifiability proof, and "updating" a Document means adding a new one. Deleting a Document only ever removes it from the working tree - Git history retains every Document permanently, and there is no compaction or history-rewrite story. Raw imaging studies (DICOM etc.) are stored in the repo in full rather than as references to external systems.

## Considered Options

- **Compaction ceremony** (rare, explicit history rewrite via filter-repo): rejected because rewriting history invalidates every clone held by every organisation that has ever cared for the patient, breaks commit signatures, and sits uneasily next to an audit-first design.
- **Pointer files for bulky Documents**: rejected for self-containment (see ADR-0001).
- **Derived artifacts only for imaging** (report plus key images in the repo, raw study referenced in a PACS): rejected because the record would no longer be complete - completeness and portability win over repo size.

## Consequences

A patient record with a normal imaging history will reach tens of GB. The accepted mitigation is Git's native partial clone (`git clone --filter=blob:none`) and sparse checkout, which let a contributor work with a large record while fetching blobs lazily - an operational measure that requires no change to the plain-files model. The wrong-upload case is handled by strict immutability: the erroneous Document stays in history and a later journal entry marks it as entered in error.
