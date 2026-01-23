# GitEHR Specification

## Purpose and Scope

GitEHR is a decentralised, Git-backed, 'batteries-included' electronic health record designed to let multiple contributors maintain a single patient's record losslessly over a patient's lifetime. It is designed for portability, simplicity, and interoperable standards.

This specification summarises the behaviour described in the codebase and documentation for the current CLI implementation and repository structure.

## Core Principles

- **Git-based storage and auditability:** Every change is version-controlled so history is preserved and auditable across contributors (see [README.md](../../README.md)).

- **Immutable journal chain:** Clinical entries are chronological files that link to their parent via a cryptographic hash, forming a tamper-evident chain seeded with random data on repository creation (see [src/commands/init.rs](../../src/commands/init.rs)).

- **Clear separation of concerns:** Standard folders divide immutable journal entries, mutable clinical state, imaging assets, and internal configuration data (see [README.md](../../README.md) and [gitehr-folder-structure/README.md](../../gitehr-folder-structure/README.md)).

- **Security and portability:** Entries are designed to be cryptographically verifiable with future support for encryption and signatures, enabling secure transport and redundancy across sites (see [README.md](../../README.md)).

---

## Layout of a GitEHR Repository

When `gitehr init` runs, it copies the template from `gitehr-folder-structure` into the current (or otherwise specified)directory and creates a `.gitehr` marker directory used to detect a valid repository (see [src/commands/init.rs](../../src/commands/init.rs)). Key directories:

- `/journal` – Chronological Markdown entries named with timestamp and GUID; each represents a single clinical event and is immutable after creation (see [gitehr-folder-structure/journal/README.md](../../gitehr-folder-structure/journal/README.md)).

- `/state` – Mutable current clinical state (allergies, medications, problems, vitals, etc.), with all updates version-controlled and paired with journal explanations (see [gitehr-folder-structure/state/README.md](../../gitehr-folder-structure/state/README.md)).

- `/imaging` – Imaging files and metadata such as DICOM, reports, scan metadata, and analyses (see [gitehr-folder-structure/imaging/README.md](../../gitehr-folder-structure/imaging/README.md)).

- `/.gitehr` – Internal configuration directory created at init time (template currently empty; see [src/commands/init.rs](../../src/commands/init.rs)).

---

## CLI Overview

Running `gitehr` with no subcommand prints the version and help on the available subcommands and exits successfully (see [src/main.rs](../../src/main.rs)).

The CLI currently provides the following commands.

### `gitehr init`

Initializes a new GitEHR repository in the current directory, creating the necessary folder structure, and including a copy of the gitehr binary in the `.gitehr` folder.

### `gitehr journal`

Adds a new clinical document to the GitEHR repository.

### `gitehr state`

Manages the mutable clinical state files within the GitEHR repository.

### `gitehr remote`

Manages remote GitEHR repositories for synchronization

### `gitehr encrypt` and `gitehr decrypt`

Encrypts or decrypts the repository using a local or remote key.

### `gitehr status`

Displays the current status of the GitEHR repository, including any uncommitted changes and the status of the encryption.

### `gitehr transport`

Converts the repository into a single-file format for easier transport

### `gitehr contributor`

Adds, enables, disables, activates, or deactivates contributors to the GitEHR record.


---

## Repository Lifecycle Summary

- **Verification:** `gitehr journal verify` recomputes hashes to ensure every non-genesis entry’s declared parent exists and matches, flagging missing links or mismatched filenames (see [src/commands/verify.rs](../../src/commands/verify.rs)).



## GitEHR repository lifecycle

### Initialization

### Adding entries

### Journal file contents

Each new file should have YAML front matter with the following fields:

- date: the date of the entry
- time: the time of the entry
- location: the location of the entry
- provider: the provider of the entry
- type: the type of the entry
- tags: a list of tags for the entry

`.gitehr` contains hidden files including internal GitEHR config.

All other folders are for clinical information.
