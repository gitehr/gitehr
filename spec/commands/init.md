<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr initialise`

Aliases: `init`, `initialize`

### `gitehr init`

Initialises a new GitEHR repository in the current working directory. Behaviour (see [src/commands/init.rs](../../src/commands/init.rs)):

1. Fails if `.gitehr` already exists to avoid overwriting an existing record.
2. Creates `.gitehr` directory.
3. Records the version of GitEHR used for initialization in `.gitehr/GITEHR_VERSION`.
4. Copies the current `gitehr` binary to `.gitehr/gitehr` for portability (the repository is self-contained).
5. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the working directory.
6. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.
7. Prints confirmation: "Initialized empty GitEHR repository"

## Binary Bundling

Each GitEHR repository includes a copy of the `gitehr` binary in `.gitehr/gitehr`. This ensures:

- **Portability**: The repository can be used on any compatible system without requiring GitEHR to be installed.
- **Version consistency**: The bundled binary matches the version used to create or last upgrade the repository.
- **Self-contained records**: A patient's complete medical record includes everything needed to read and modify it.

The bundled binary can be updated using `gitehr upgrade-binary` (see [upgrade-binary.md](upgrade-binary.md)).
