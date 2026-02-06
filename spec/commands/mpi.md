<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr mpi`

Resolve and manage patient identifiers against a local Master Patient Index (MPI).

Default MPI location:
- A single file at the store root (directory above all repos), named `gitehr-mpi.json`.
- The path can be overridden by an environment variable (e.g., `GITEHR_MPI_PATH`).

All subcommands operate on the MPI file and do not require a running service.

## `gitehr mpi search <identifier>`

Resolve an identifier (e.g., NHS number, MRN) to a canonical patient ID and repo path.

Behavior:
- Returns the canonical `patient_id` and `repo_path`.
- If multiple matches exist, returns all matches (or fails with a disambiguation error).

## `gitehr mpi link <patient_id> <type> <value>`

Link a new identifier to a canonical patient.

Behavior:
- Fails if the identifier is already linked to a different patient.
- Updates `updated_at`.

## `gitehr mpi unlink <type> <value>`

Remove an identifier link.

Behavior:
- Fails if the identifier does not exist.
- Does not delete the patient record if other identifiers remain.

## `gitehr mpi create <patient_id>`

Create a new patient record in the MPI.

Behavior:
- Fails if `patient_id` already exists.
- Creates an empty identifier list and default status `active`.

## `gitehr mpi merge <from_patient_id> <to_patient_id>`

Merge one patient into another.

Behavior:
- Marks `from_patient_id` as `merged` and sets `merged_into`.
- Moves identifiers to `to_patient_id` unless conflicts exist.

## `gitehr mpi list`

List patient records, optionally filtered by identifier type.

## `gitehr mpi search <text>`

Search the MPI for identifiers containing the provided text.

Behavior:
- Returns matching identifiers and their patient records.
- Intended for operator use; not a substitute for exact `resolve`.

## `gitehr mpi path <patient_id>`

Return the repo path for a canonical patient ID.
