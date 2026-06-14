# GitEHR

The domain language used throughout the GitEHR codebase, CLI, GUI, and documentation. Terms here are the ones a clinician-developer needs to understand to read any other GitEHR document or run the demo.

## Language

**Patient record:**
A single Git repository representing one patient's complete medical history across all organisations that have ever cared for them. Multiple contributors write to the same patient record by cloning it, adding entries, and syncing via Git remotes.
_Avoid_: Chart, file (ambiguous), EHR (means the system, not the record), record (alone — too generic)

**Contributor:**
A person who can author journal entries in a patient record. Tracked in `.gitehr/contributors.json`; exactly one is "active" at any given time and is stamped as the `author` of new entries. Clinicians, the patient themselves, and other authorised parties are all contributors.
_Avoid_: User (technical, not clinical), author (alone — ambiguous with entry-level metadata), clinician (too narrow)

**Document:**
A clinical source artifact stored as a file in the patient record - a PDF, scanned letter, photograph, Word document, or imaging study (e.g. DICOM). A first-class record object, referenced by one or more journal entries; it is not itself a journal entry. Documents are immutable and write-once: the SHA-256 recorded in each referencing journal entry is a verifiability proof, and "updating" a Document means adding a new one. Deleting a Document only removes it from the current working tree; Git history retains every Document ever added, permanently.
_Avoid_: Attachment (UI metaphor; at most a casual CLI synonym), file (ambiguous), upload
