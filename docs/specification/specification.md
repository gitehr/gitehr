# GitEHR Specification

## Purpose and Scope
GitEHR is a decentralised, Git-backed electronic health record format and CLI designed to let multiple contributors maintain a single patient's record with strong provenance, portability, and long-term accessibility.
This specification summarises the behaviour described in the codebase and documentation for the current CLI implementation and repository structure.

## Core Principles
- **Git-based storage and auditability:** Every change is version-controlled so history is preserved and auditable across contributors (see [README.md](../../README.md)).
- **Immutable journal chain:** Clinical entries are chronological files that link to their parent via a cryptographic hash, forming a tamper-evident chain seeded with random data on repository creation (see [src/commands/init.rs](../../src/commands/init.rs)).
- **Clear separation of concerns:** Standard folders divide immutable journal entries, mutable clinical state, imaging assets, and internal configuration data (see [README.md](../../README.md) and [gitehr-folder-structure/README.md](../../gitehr-folder-structure/README.md)).
- **Security and portability:** Entries are designed to be cryptographically verifiable with future support for encryption and signatures, enabling secure transport and redundancy across sites (see [README.md](../../README.md)).

## Repository Layout
When `gitehr init` runs, it copies the template from `gitehr-folder-structure` into the current directory and creates a `.gitehr` marker directory used to detect a valid repository (see [src/commands/init.rs](../../src/commands/init.rs)). Key directories:
- `/journal` – Chronological Markdown entries named with timestamp and GUID; each represents a single clinical event and is immutable after creation (see [gitehr-folder-structure/journal/README.md](../../gitehr-folder-structure/journal/README.md)).
- `/state` – Mutable current clinical state (allergies, medications, problems, vitals, etc.), with all updates version-controlled and paired with journal explanations (see [gitehr-folder-structure/state/README.md](../../gitehr-folder-structure/state/README.md)).
- `/imaging` – Imaging files and metadata such as DICOM, reports, scan metadata, and analyses (see [gitehr-folder-structure/imaging/README.md](../../gitehr-folder-structure/imaging/README.md)).
- `/.gitehr` – Internal configuration directory created at init time (template currently empty; see [src/commands/init.rs](../../src/commands/init.rs)).

## CLI Overview
Running `gitehr` with no subcommand prints the tool version defined in Cargo metadata and exits successfully (see [src/main.rs](../../src/main.rs)). The CLI currently provides `init`, `add`, and `journal verify` subcommands.

### `gitehr init`
Initialises a new GitEHR repository in the current working directory. Behaviour (see [src/commands/init.rs](../../src/commands/init.rs)):
1. Fails if `.gitehr` already exists to avoid overwriting an existing record.
2. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the working directory; creates `.gitehr` locally.
3. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.
4. Prints confirmation: “Initialized empty GitEHR repository.”

### `gitehr add <content>`
Appends a new clinical journal entry containing the provided text content.
- Requires the current directory to already be a GitEHR repository (presence of `.gitehr`); otherwise, the command aborts with guidance to run `gitehr init` (see [src/main.rs](../../src/main.rs)).
- Determines the most recent journal entry by filename ordering (timestamps in filenames). If found, calculates its SHA-256 hash and sets that as the new entry’s `parent_hash`; the matching filename is stored as `parent_entry` (see [src/main.rs](../../src/main.rs) and [src/commands/journal.rs](../../src/commands/journal.rs)).
- Creates a new Markdown file named `journal/<timestamp>-<uuid>.md` using the current UTC time down to milliseconds plus a random UUID (see [src/commands/journal.rs](../../src/commands/journal.rs)).
- Prepends YAML front matter containing `parent_hash`, `parent_entry`, the creation timestamp, and (currently optional) `author`, followed by the user-provided content. Prints the created filename on success (see [src/commands/journal.rs](../../src/commands/journal.rs)).

### `gitehr journal verify`
Validates the integrity of the journal chain (see [src/commands/verify.rs](../../src/commands/verify.rs) and [src/main.rs](../../src/main.rs)).
- Requires a GitEHR repository and the existence of the `journal` directory; otherwise, it returns an error.
- Sorts all journal files by filename, computes each entry’s SHA-256 hash, and builds a map of hash → filename for lookup.
- For each entry, parses YAML front matter into a `JournalEntry`; errors if the front matter is missing or invalid.
- For non-genesis entries, ensures the declared `parent_hash` exists in the map and the recorded `parent_entry` matches the expected filename; otherwise, it reports a broken chain or missing parent.
- On success, prints “Journal verification successful: N entries verified.”

## Journal Data Model
- Each entry file starts with YAML front matter representing `JournalEntry` with fields `parent_hash` (optional), `parent_entry` (optional for genesis), `timestamp` (UTC), and `author` (optional, reserved for future identity management) (see [src/commands/journal.rs](../../src/commands/journal.rs)).
- The file content after the front matter holds the clinical narrative or data supplied to `gitehr add` or the genesis message (see [src/commands/journal.rs](../../src/commands/journal.rs) and [src/commands/init.rs](../../src/commands/init.rs)).
- File naming embeds chronological ordering and uniqueness via timestamp and UUID, enabling simple sorting to reconstruct history (see [src/commands/journal.rs](../../src/commands/journal.rs)).

## Repository Lifecycle Summary
- **Initialization:** `gitehr init` seeds a new record with the template folders and a genesis journal entry anchored to a random hash, establishing the start of the chain (see [src/commands/init.rs](../../src/commands/init.rs)).
- **Adding entries:** `gitehr add` appends immutable journal files that link back to the latest entry, preserving a verifiable chain of custody and chronological ordering (see [src/main.rs](../../src/main.rs) and [src/commands/journal.rs](../../src/commands/journal.rs)).
- **Verification:** `gitehr journal verify` recomputes hashes to ensure every non-genesis entry’s declared parent exists and matches, flagging missing links or mismatched filenames (see [src/commands/verify.rs](../../src/commands/verify.rs)).

## Conceptual Positioning
GitEHR targets patient-centric, portable health records that multiple organisations can contribute to while keeping data integrity and provenance verifiable. It leverages distributed version control and plain-text files to avoid vendor lock-in and to remain usable even as surrounding software evolves (see [docs/index.md](../index.md) and [README.md](../../README.md)).
