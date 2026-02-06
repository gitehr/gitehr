<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr initialise`

Aliases: `init`, `initialize`

### `gitehr init`

Initialises a new GitEHR repository **from the store root** (the directory that contains `gitehr-mpi.json`). The command creates a new repo directory named with a **Crockford Base32 UUIDv7**, adds the record to the MPI, and then runs the standard initialization steps inside that new directory. Behaviour (see [src/commands/init.rs](../../src/commands/init.rs)):

1. If `gitehr-mpi.json` is not found in the current directory (store root), creates a new MPI file using the v1 schema.
2. Generates a new UUIDv7 and encodes it using Crockford Base32 to create the repo directory name.
3. Creates the new repo directory and fails if it already exists.
4. Adds a new patient record to the MPI (with `patient_id` = UUIDv7, `repo_path`, `status = active`, empty identifiers).
5. Creates `.gitehr` directory inside the new repo.
6. Runs `git init` to initialize a git repository.
7. Records the version of GitEHR used for initialization in `.gitehr/GITEHR_VERSION`.
8. Copies the current `gitehr` binary to `.gitehr/gitehr` for portability (the repository is self-contained).
9. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the new repo directory.
10. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.
11. Prints confirmation: "Initialized empty GitEHR repository"

## Binary Bundling

Each GitEHR repository includes a copy of the `gitehr` binary in `.gitehr/gitehr`. This ensures:

- **Portability**: The repository can be used on any compatible system without requiring GitEHR to be installed.
- **Version consistency**: The bundled binary matches the version used to create or last upgrade the repository.
- **Self-contained records**: A patient's complete medical record includes everything needed to read and modify it.

The bundled binary can be updated using `gitehr upgrade-binary` (see [upgrade-binary.md](upgrade-binary.md)).
