<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr store`

Manage a GitEHR **Store**: a directory of subject repos plus a `gitehr-mpi.json` index (the MPI). GitEHR is Store-first - see [ADR-0005](../adr/0005-store-first-model.md). Implemented in `cli/src/commands/store/` and `cli/src/commands/scaffold.rs`.

## `gitehr store init [name]`

Bootstrap a new Store in the current (empty) directory. Behaviour:

1. Fail if the directory is already a Store (`gitehr-mpi.json` exists) or a repo (`.gitehr/` exists).
2. Mint the first subject's canonical id: a UUIDv7 encoded in Crockford Base32.
3. Choose the subject's directory: a de-duplicated slug of `name` if given (else the canonical id). With no `name`, prompt on a TTY; non-interactively, use the id.
4. Scaffold the subject repo in that directory: create `.gitehr/`, `git init`, write `.gitehr/GITEHR_VERSION` and `.gitehr/ID` (the canonical id), bundle the binary at `.gitehr/gitehr`, and copy the `folder-structure/` template (journal, state, imaging, documents, READMEs).
5. Write `gitehr-mpi.json` (v1 schema) with the subject recorded (`patient_id` = canonical id, `repo_path` = directory, `status = active`, empty identifiers).

## `gitehr store add [name] [--identifier type:value]...`

Create and register a further subject. Requires a Store (walks up to `gitehr-mpi.json`, then falls back to the configured Store). Same id/directory/scaffold logic as `init` step 2-4, then appends the subject to the MPI with any `--identifier` values.

## `gitehr store remove <id-or-name>`

Remove a subject from the MPI, matching the argument against the canonical id or the directory name. The subject's repository files are **not** deleted (the record only grows, [ADR-0002](../adr/0002-record-only-grows.md)); this only unlinks it from the index.

## `gitehr store list`

List the subjects: directory/friendly name, canonical id, and recorded identifiers.

## Planned

The MPI identifier-resolution operations - `search`, `link`, `unlink`, `merge`, `path` (see [mpi.md](mpi.md)) - and the `GITEHR_MPI_PATH` override fold in here as further `gitehr store` subcommands.

## Binary bundling

Each repo bundles the `gitehr` binary at `.gitehr/gitehr`, so a record is self-contained (portable, version-pinned). Update it with [`gitehr upgrade-binary`](upgrade-binary.md).
