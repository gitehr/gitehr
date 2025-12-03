# GitEHR Specification

## Purpose and Scope
GitEHR is a decentralised, Git-backed electronic health record format and CLI designed to let multiple contributors maintain a single patient's record with strong provenance, portability, and long-term accessibility.
This specification summarises the behaviour described in the codebase and documentation for the current CLI implementation and repository structure.

## Core Principles
- **Git-based storage and auditability:** Every change is version-controlled so history is preserved and auditable across contributors.【F:README.md†L5-L38】
- **Immutable journal chain:** Clinical entries are chronological files that link to their parent via a cryptographic hash, forming a tamper-evident chain seeded with random data on repository creation.【F:README.md†L14-L20】【F:src/commands/init.rs†L40-L49】
- **Clear separation of concerns:** Standard folders divide immutable journal entries, mutable clinical state, imaging assets, and internal configuration data.【F:README.md†L21-L33】【F:gitehr-folder-structure/README.md†L1-L3】
- **Security and portability:** Entries are designed to be cryptographically verifiable with future support for encryption and signatures, enabling secure transport and redundancy across sites.【F:README.md†L28-L38】

## Repository Layout
When `gitehr init` runs, it copies the template from `gitehr-folder-structure` into the current directory and creates a `.gitehr` marker directory used to detect a valid repository.【F:src/commands/init.rs†L10-L38】 Key directories:
- `/journal` – Chronological Markdown entries named with timestamp and GUID; each represents a single clinical event and is immutable after creation.【F:gitehr-folder-structure/journal/README.md†L1-L11】
- `/state` – Mutable current clinical state (allergies, medications, problems, vitals, etc.), with all updates version-controlled and paired with journal explanations.【F:gitehr-folder-structure/state/README.md†L1-L19】
- `/imaging` – Imaging files and metadata such as DICOM, reports, scan metadata, and analyses.【F:gitehr-folder-structure/imaging/README.md†L1-L10】
- `/.gitehr` – Internal configuration directory created at init time (template currently empty).【F:src/commands/init.rs†L10-L24】

## CLI Overview
Running `gitehr` with no subcommand prints the tool version defined in Cargo metadata and exits successfully.【F:src/main.rs†L42-L50】 The CLI currently provides `init`, `add`, and `journal verify` subcommands.

### `gitehr init`
Initialises a new GitEHR repository in the current working directory. Behaviour:
1. Fails if `.gitehr` already exists to avoid overwriting an existing record.【F:src/commands/init.rs†L10-L14】
2. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the working directory; creates `.gitehr` locally.【F:src/commands/init.rs†L16-L38】
3. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.【F:src/commands/init.rs†L40-L47】
4. Prints confirmation: “Initialized empty GitEHR repository.”【F:src/commands/init.rs†L48-L49】

### `gitehr add <content>`
Appends a new clinical journal entry containing the provided text content.
- Requires the current directory to already be a GitEHR repository (presence of `.gitehr`); otherwise, the command aborts with guidance to run `gitehr init`.【F:src/main.rs†L38-L76】
- Determines the most recent journal entry by filename ordering (timestamps in filenames). If found, calculates its SHA-256 hash and sets that as the new entry’s `parent_hash`; the matching filename is stored as `parent_entry`.【F:src/main.rs†L65-L68】【F:src/commands/journal.rs†L17-L75】
- Creates a new Markdown file named `journal/<timestamp>-<uuid>.md` using the current UTC time down to milliseconds plus a random UUID.【F:src/commands/journal.rs†L35-L47】
- Prepends YAML front matter containing `parent_hash`, `parent_entry`, the creation timestamp, and (currently optional) `author`, followed by the user-provided content. Prints the created filename on success.【F:src/commands/journal.rs†L8-L55】

### `gitehr journal verify`
Validates the integrity of the journal chain.
- Requires a GitEHR repository and the existence of the `journal` directory; otherwise, it returns an error.【F:src/main.rs†L70-L83】【F:src/commands/verify.rs†L42-L46】
- Sorts all journal files by filename, computes each entry’s SHA-256 hash, and builds a map of hash → filename for lookup.【F:src/commands/verify.rs†L48-L60】
- For each entry, parses YAML front matter into a `JournalEntry`; errors if the front matter is missing or invalid.【F:src/commands/verify.rs†L62-L83】
- For non-genesis entries, ensures the declared `parent_hash` exists in the map and the recorded `parent_entry` matches the expected filename; otherwise, it reports a broken chain or missing parent.【F:src/commands/verify.rs†L84-L110】
- On success, prints “Journal verification successful: N entries verified.”【F:src/commands/verify.rs†L113-L117】

## Journal Data Model
- Each entry file starts with YAML front matter representing `JournalEntry` with fields `parent_hash` (optional), `parent_entry` (optional for genesis), `timestamp` (UTC), and `author` (optional, reserved for future identity management).【F:src/commands/journal.rs†L8-L40】
- The file content after the front matter holds the clinical narrative or data supplied to `gitehr add` or the genesis message.【F:src/commands/journal.rs†L49-L55】【F:src/commands/init.rs†L40-L47】
- File naming embeds chronological ordering and uniqueness via timestamp and UUID, enabling simple sorting to reconstruct history.【F:src/commands/journal.rs†L42-L47】【F:src/commands/journal.rs†L58-L69】

## Repository Lifecycle Summary
- **Initialization:** `gitehr init` seeds a new record with the template folders and a genesis journal entry anchored to a random hash, establishing the start of the chain.【F:src/commands/init.rs†L10-L49】
- **Adding entries:** `gitehr add` appends immutable journal files that link back to the latest entry, preserving a verifiable chain of custody and chronological ordering.【F:src/main.rs†L57-L69】【F:src/commands/journal.rs†L17-L55】
- **Verification:** `gitehr journal verify` recomputes hashes to ensure every non-genesis entry’s declared parent exists and matches, flagging missing links or mismatched filenames.【F:src/commands/verify.rs†L42-L110】

## Conceptual Positioning
GitEHR targets patient-centric, portable health records that multiple organisations can contribute to while keeping data integrity and provenance verifiable. It leverages distributed version control and plain-text files to avoid vendor lock-in and to remain usable even as surrounding software evolves.【F:docs/index.md†L1-L29】【F:README.md†L5-L38】
