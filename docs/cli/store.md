# gitehr store

A GitEHR install is always a **Store**: a directory holding one or more subject **repos** (each a complete record) plus a small `gitehr-mpi.json` index, the Main Patient Index (MPI). One model fits everyone - a single self-hoster, a family (yourself, your children, an elderly parent you care for), a **pet owner** keeping vet records, or a clinic of any size. See [the repository structure](../design/repository-structure.md).

```text
gitehr store init [name]        Create a Store + MPI + the first subject's repo
gitehr store add [name]         Create and register another subject's repo
gitehr store remove <id|name>   De-register a subject (its files are kept)
gitehr store list               List the subjects in the Store
```

Each subject gets a stable canonical id - a time-ordered UUIDv7 in Crockford base32 - recorded in the MPI and in the repo's `.gitehr/ID`. Its directory is a friendly slug when you give a name (`rex/`, `mum/`), or the canonical id when you don't, so a self-hoster gets readable folders while a large store needs no manual naming.

## gitehr store init

```text
gitehr store init [name]
```

Run in an empty directory to make it a Store. Creates `gitehr-mpi.json` and the first subject's repo - scaffolded with `.gitehr/` (the bundled binary, version, and canonical id), the template folders, and a git repo. With no `name`, it prompts on a terminal, or uses an auto-generated id when scripted.

```bash
mkdir my-records && cd my-records
gitehr store init rex          # a Store whose first subject is "rex"
```

## gitehr store add

```text
gitehr store add [name] [--identifier type:value]...
```

Creates and registers another subject. Run from anywhere inside the Store. `--identifier` records a real-world identifier (NHS number, microchip, MRN) against the subject in the MPI; repeatable.

```bash
gitehr store add mum
gitehr store add fluffy --identifier microchip:985112000000000
gitehr store add               # auto-id subject (no friendly name)
```

## gitehr store remove

```text
gitehr store remove <id-or-name>
```

De-registers a subject from the MPI by canonical id or friendly name. The subject's repository files are **not** deleted - removal only unlinks it from the index.

## gitehr store list

```text
gitehr store list
```

Lists the subjects: friendly name (or id), canonical id, and any recorded identifiers.

## Working inside a subject

Repo-level commands (`journal`, `document`, `status`, ...) run inside a subject's repo - like git, you `cd` into the record you want:

```bash
cd rex
gitehr journal add "Annual booster vaccination."
```

When a Store has exactly one subject, repo commands **auto-target** it, so a lone self-hoster can skip the `cd`:

```bash
gitehr store init me      # a Store of one
gitehr journal add "..."  # runs inside "me" automatically
```
