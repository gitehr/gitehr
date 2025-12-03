<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr initialise`

Aliases: `init`, `initialize`

### `gitehr init`
Initialises a new GitEHR repository in the current working directory. Behaviour (see [src/commands/init.rs](../../src/commands/init.rs)):
1. Fails if `.gitehr` already exists to avoid overwriting an existing record.
2. Copies the template structure (including journal, state, imaging, and README files) from `gitehr-folder-structure` into the working directory; creates `.gitehr` locally.
3. Generates a 32-byte random seed, hashes it with SHA-256, and writes a genesis journal entry whose `parent_hash` references that seed to anchor the chain.
4. Prints confirmation: “Initialized empty GitEHR repository.”



<!-- REVIEW/ADD - **Initialization:** `gitehr init` seeds a new record with the template folders and a genesis journal entry anchored to a random hash, establishing the start of the chain (see [src/commands/init.rs](../../src/commands/init.rs)). `gitehr init` creates a folder structure inside the current directory. The folders are copied from the `gitehr-folder-structure` directory. On creation of the repository, the first file is created with random data and a timestamp.
 -->