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

### [`gitehr init`](commands/init.md)

Initializes a new GitEHR repository in the current directory, creating the necessary folder structure, and including a copy of the gitehr binary in the `.gitehr` folder.

### [`gitehr journal`](commands/journal.md)

Adds a new clinical document to the GitEHR repository.

### [`gitehr state`](commands/state.md)

Manages the mutable clinical state files within the GitEHR repository.

### [`gitehr remote`](commands/remote.md)

Manages remote GitEHR repositories for synchronization

### [`gitehr encrypt`](commands/encrypt.md) and [`gitehr decrypt`](commands/decrypt.md)

Encrypts or decrypts the repository using a local or remote key.

### [`gitehr status`](commands/status.md)

Displays the current status of the GitEHR repository, including any uncommitted changes and the status of the encryption.

### [`gitehr transport`](commands/transport.md)

Converts the repository into a single-file format for easier transport. Additional encryption layers can be applied at this stage.

### [`gitehr contributor`](commands/contributor.md)

### [`gitehr gui`](commands/gui.md)

Opens the GitEHR graphical user interface for easier interaction with the repository.

### [`gitehr upgrade`](commands/upgrade.md)

Upgrades the GitEHR repository to the latest version, applying any necessary migrations.

### [`gitehr version`](commands/version.md)

Displays the current version of the GitEHR CLI.

Adds, enables, disables, activates, or deactivates contributors to the GitEHR record.

---

## The GitEHR Journal

The journal is an append-only logbook of clinical interactions organised per patient.

## Layout

```
journal/
  patients/
    <patient_id>/
      <ISO8601-event-timestamp>--<event_id>.md
```

- `patient_id` should be a stable GitEHR identifier (for example `pat-1234`).
- `ISO8601-event-timestamp` must represent when the interaction happened (UTC), e.g. `2025-11-28T12-34-56Z`.
- `event_id` is a short GUID/slug (`c01`, `evt-uuid`) to guarantee uniqueness.

## File format

Each file describes one interaction and uses **YAML front matter + Markdown body**:
Need to strongly consider the cons of using markdown through out

Proposed yaml header structure:

```yaml
---
gitehr_event_id: c01
gitehr_patient_id: pat-1234
recorded_at: 2025-11-28T12:34:56Z
author:
  id: user-dr-osutuk
  name: "Akan Osutuk"
  role: "ST2 Paediatrics"
context:
  location: "Ward 3B"
  encounter_id: "enc-987"
  episode_id: "ep-ALL-block1"
links:
  related_imaging:
    - "imaging/pat-1234/studies/study-abc/meta.json"
  related_files:
    - "attachments/pat-1234/letters/2025-11-28-discharge.pdf"
codes:
  diagnoses:
    - system: "snomed"
      code: "123456"
      display: "Acute lymphoblastic leukaemia"
  procedures: []
correction_of: null
---
Markdown narrative of the clinical interaction...
```

- YAML captures machine-readable metadata and references.
- The Markdown body is the human narrative, free-form but diffable.

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
