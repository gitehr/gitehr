<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr initialise`

Aliases: `init`, `initialize`

### `gitehr init`

Initialises a new GitEHR repository in the current working directory. Behaviour (see [src/commands/init.rs](../../src/commands/init.rs)):

1. Fails if `.gitehr` already exists to avoid overwriting an existing record.
2. Creates `.gitehr` directory.
3. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the working directory
4. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.
5. Includes a copy of the `gitehr` binary in the `.gitehr` folder for portability.
6. Records the version of GitEHR used for initialization in a `GITEHR_VERSION` file inside `.gitehr`.
7. Prints confirmation: “Initialized empty GitEHR repository.”

