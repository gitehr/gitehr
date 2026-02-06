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

- `/documents` – Non-imaging clinical documents such as reports, correspondence, and lab results (see [gitehr-folder-structure/documents/README.md](../../gitehr-folder-structure/documents/README.md)).

- `/.gitehr` – Internal configuration directory created at init time (template currently empty; see [src/commands/init.rs](../../src/commands/init.rs)).

---

## CLI Overview

Running `gitehr` with no subcommand prints the version and help on the available subcommands and exits successfully (see [src/main.rs](../../src/main.rs)).

The CLI currently provides the following commands.

### [`gitehr init`](commands/init.md)

Initializes a new GitEHR repository **from the store root**, creating a new repo directory named with a Crockford Base32 UUIDv7, recording it in the MPI, and then creating the necessary folder structure and bundled binary within that new repo.

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

### [`gitehr user`](commands/user.md)

Adds, enables, disables, activates, or deactivates users for the GitEHR record.

### [`gitehr mpi`](commands/mpi.md)

Resolves and manages patient identifiers against a local Master Patient Index (MPI).

### [`gitehr gui`](commands/gui.md)

Opens the GitEHR graphical user interface for easier interaction with the repository.

### [`gitehr upgrade`](commands/upgrade.md)

Upgrades the GitEHR repository to the latest version, applying any necessary migrations and updating the bundled binary.

### [`gitehr upgrade-binary`](commands/upgrade-binary.md)

Updates the bundled binary in `.gitehr/gitehr` to match the current CLI version.

### [`gitehr version`](commands/version.md)

Displays the current GitEHR version shared by the CLI and GUI.

---

## The GitEHR Journal

The journal is an append-only logbook of clinical interactions. Each GitEHR repository represents a single patient's complete medical record.

## Layout

```
journal/
  <ISO8601-timestamp>-<uuid>.md
```

- Each file is named with its creation timestamp (UTC, millisecond precision) followed by a UUID to guarantee uniqueness.
- Example: `20260205T032720.630Z-dab47f45-f5ff-45a2-b6b4-6f2285b173ac.md`
- Files are sorted chronologically by filename.
- The first entry (genesis) is created automatically by `gitehr init` and anchors the chain with a random seed hash.

## File format

Each file describes one clinical interaction and uses **YAML front matter + Markdown body**:

```yaml
---
parent_hash: '<SHA-256 hash of parent entry content, or random seed for genesis>'
parent_entry: '<filename of parent entry, or null for genesis>'
timestamp: '<ISO 8601 UTC timestamp>'
author: '<optional user ID>'
---

Markdown narrative of the clinical interaction...
```

- `parent_hash` links this entry to its parent, forming a tamper-evident chain.
- `parent_entry` records the filename of the parent for human readability.
- `timestamp` is the creation time of this entry.
- `author` (optional) identifies the user who created this entry.
- The Markdown body contains the human-readable clinical narrative.

---

## Repository Lifecycle

### Initialization

Running `gitehr init` creates a new repository with:
- The folder structure from the template (`journal/`, `state/`, `imaging/`, `documents/`)
- A `.gitehr/` configuration directory with version information
- A genesis journal entry that anchors the hash chain with a random seed

### Adding Entries

Use `gitehr journal add "<content>"` to append new entries. Each entry:
- Links to the previous entry via `parent_hash` and `parent_entry`
- Gets a unique filename based on timestamp and UUID
- Becomes immutable once committed to Git

### Verification

`gitehr journal verify` validates the integrity of the journal chain by:
- Checking that every non-genesis entry's `parent_hash` exists in the journal
- Verifying that `parent_entry` matches the expected filename
- Confirming the hash chain is unbroken from genesis to the latest entry

### Documentation site

The GitEHr repo includes a Material for MkDocs documentation site that provides an overview of the repository structure and usage. It is generated from the `docs/` directory and can be served locally with `mkdocs serve` or built into static files with `mkdocs build`.

The documsntation site has nav sections for GUI and CLI usage, as well as detailed explanations of the journal and state structures. It serves as a user-friendly reference for both clinicians and developers interacting with GitEHR repositories.

---

## Scaling Many Repos, Sharding, and Patient Indexing

This section captures practical limits and architectural patterns for hosting **hundreds of thousands** of GitEHR repositories on shared storage, and outlines a recommended sharding strategy plus a Master Patient Index (MPI).

### Filesystem Limits and Practical Constraints

Hard limits vary by filesystem and OS, but **practical performance** is usually dictated by:
- Directory fan-out (too many entries in a single directory slows lookup and listing).
- Inode availability (for inode-based filesystems like ext4/XFS).
- Path length, filename length, and per-volume maximum size.

Representative limits (non-exhaustive):
- **exFAT**: large volume and file size limits, and a per-directory maximum of 2,796,202 entries. citeturn0search48
- **NTFS (Windows)**: maximum volume and file sizes depend on cluster size, with modern Windows supporting up to 8 PB volumes in current releases. citeturn0search0
- **ext4 (Linux)**: very large theoretical file system sizes; limits depend on block size and whether the 64‑bit feature is enabled. citeturn0search1turn0search2
- **XFS (Linux)**: a 64‑bit filesystem with very large theoretical limits (2^64 bytes); OS distributions often publish smaller tested/supported limits. citeturn1search3turn1search4

**Guidance**:
- Avoid placing many repositories in a single directory. Use deterministic sharding (see below).
- Keep directory fan‑out modest and stable to reduce metadata hot‑spots.
- For large multi‑tenant stores, prefer filesystems optimized for large metadata workloads (XFS or ext4 on Linux; NTFS on Windows Server). Always validate with workload‑specific benchmarks.

### Sharding Strategy for “Whole Hospital” Stores

The goal is to avoid pathological directory sizes while keeping repo paths deterministic and easy to resolve.

Recommended pattern:
- Use a **stable patient identifier hash** to distribute repos across directories.
- Split by **2–3 levels** of prefix directories.

Example (SHA‑256 prefix):
```
repos/
  3a/
    7f/
      3a7f9c.../
        <gitehr-repo>
```

Notes:
- 2 prefix bytes yields 65,536 shards (256×256), which is generally sufficient.
- 3 prefix bytes yields 16,777,216 shards; only needed at very large scale.
- Keep the **repo directory name** a deterministic function of the canonical patient ID so it is reproducible and collision‑resistant.
- For `gitehr init`, the repo directory name is the **Crockford Base32 UUIDv7** generated for the patient.

### Master Patient Index (MPI)

At scale, a Master Patient Index is strongly recommended to map multiple identifiers to a single GitEHR repo. The default, "batteries included" approach is a **single local MPI file** stored in the directory above all repos (the "store root"). More sophisticated deployments can replace or mirror this with a service or API.

MPI responsibilities:
- **Cross‑reference** identifiers (e.g., NHS number, hospital MRN, national IDs).
- Handle **identifier changes** and merges (merge/alias history).
- Provide a **canonical patient ID** used for repo naming and sharding.

Recommended MPI data model (minimal):
```
patient_id (canonical UUID)
  identifiers:
    - type: "NHS"
      value: "..."
    - type: "MRN"
      value: "..."
  repo_path: "repos/3a/7f/3a7f9c.../"
  status: active | merged | inactive
  merged_into: <patient_id> | null
  updated_at: <timestamp>
```

MPI file format (v1, JSON):
```
{
  "version": 1,
  "updated_at": "2026-02-06T12:00:00Z",
  "patients": [
    {
      "patient_id": "018f0e2c-89f4-7c2d-8f7e-4a20cfd90123",
      "repo_path": "repos/01/8f/01/8f0e2c.../",
      "status": "active",
      "merged_into": null,
      "updated_at": "2026-02-05T18:22:10Z",
      "identifiers": [
        { "type": "NHS", "value": "943-476-5919" },
        { "type": "MRN", "value": "HOSP-001122" }
      ]
    }
  ]
}
```
`repo_path` is derived from the canonical patient ID (UUIDv7), using the Crockford Base32 form for the repo directory name and its hash‑based shard prefixes.

Operational guidance:
- The default MPI is a **single file** at the store root (e.g., `gitehr-mpi.json`), no API required.
- GitEHR repo structure remains deterministic even if identifiers change.
- All imports and lookups resolve identifiers through the MPI to find the canonical repo.
- For large deployments, the MPI file can be mirrored or replaced by a service with equivalent semantics.

Performance notes (100k patients, typical SSD):
- A naive per‑command scan (read + parse + linear search) is generally acceptable for operator use but can be hundreds of milliseconds depending on file size and CPU.
- Building an **in‑memory index** per command makes lookups effectively O(1) and fast enough for interactive use.
- If needed, add an **optional cached index file** (e.g., `gitehr-mpi.index.json`) to avoid repeated parsing for heavy workloads.

### Open Questions / Follow‑ups

- Define the **canonical patient ID** source (generated UUID vs national ID hash).
- Define the **MPI file format** and versioning strategy (JSON vs YAML, schema evolution).
- Validate filesystem performance with realistic repo sizes and file counts.
