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

## `gitehr store search <query>`

Find subjects. Matching is: exact `type:value` identifier, exact canonical id or friendly name (case-insensitive), substring match on identifier values, and substring match on the directory name for queries of three characters or more. Errors with no matches.

## `gitehr store link <id-or-name> <type:value>`

Link an identifier to a subject. Refuses if the identifier is already linked to a *different* subject (an identifier resolves to at most one subject); re-linking the same subject is a no-op. Updates both the subject's and the MPI's `updated_at`.

## `gitehr store unlink <type:value>`

Remove the identifier link from whichever subject holds it. Fails if no subject holds it. The subject's record itself is untouched.

## `gitehr store merge <from> <into>`

Merge subject `from` into subject `into`: `from` is marked `status = merged` with `merged_into` set, its identifiers move to the target, and its identifier list is cleared. Refuses a self-merge, a re-merge of an already-merged source, and any identifier clash with the target (conflicts must be resolved explicitly rather than silently dropped). Repository files are never touched.

## `gitehr store path <id-or-name>`

Print the repository path for a subject - the scripting-friendly counterpart to `search`.

## MPI location

All subcommands read the MPI through one resolver: `GITEHR_MPI_PATH` (when set to a non-empty path) overrides the default of `gitehr-mpi.json` in the current directory. `store init` writes the bootstrap MPI to the resolved path too. The walk-up-to-Store-root behaviour of other commands is unchanged.

## Binary bundling

Each repo bundles the `gitehr` binary at `.gitehr/gitehr`, so a record is self-contained (portable, version-pinned). Update it with [`gitehr upgrade-binary`](upgrade-binary.md).
