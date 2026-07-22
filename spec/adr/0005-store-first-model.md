# GitEHR is Store-first: every install is a Store of one or more subject repos, indexed by an MPI

A GitEHR deployment is always a **Store**: a directory containing one or more subject **repos** (each a complete patient/subject record) plus a single `gitehr-mpi.json` index - the Main Patient Index (MPI) - at its root. There is no standalone single-repo mode. `gitehr store init` bootstraps everything (the Store, the MPI, and the first subject repo) in one step; `gitehr store add` creates each further subject repo and registers it; there is no top-level `gitehr store init`. The repo is the git-like unit you `cd` into to work on a record; the Store is the thin organising layer a few commands operate on. All operations on the MPI live under `gitehr store` - the MPI is an index file, not its own command.

## Who this is for

Store-first is not a clinic-only feature; it fits the self-hoster too, because **a self-hoster is usually already multi-subject**:

- **Individuals and families** - themselves plus children, or an elderly parent they care for. The MPI generalises to a *Main Subject Index*.
- **Pet owners** - vaccinations, vet visits, a major surgery, kept in one place the owner owns forever. A low-stakes, high-volume, emotionally compelling on-ramp that exercises the identical multi-subject model. The docs treat this as a first-class audience, not an afterthought.
- **Practitioners** running a clinic Store of arbitrary size.

One model serves all three, and a family or pet-owner self-hoster grows into a clinic-shaped Store with no restructuring.

## The two levels and how commands find them

| Level | What it is | Marker | Found by |
|---|---|---|---|
| **Repo** | one subject's complete record | `.gitehr/` | walk up from cwd, exactly as git finds `.git/` |
| **Store** | a directory of repos + the MPI index | `gitehr-mpi.json` at the root | walk up from cwd, then configured Store |

- **Repo-level** commands (`journal`, `document`, `import`, `state`, `status`, `encrypt`, `decrypt`, `transport`) run from inside a repo - you `cd` into the subject you want, the git way.
- **Store-level** commands (`store …`) run at the Store root.
- Outside both → use the configured Store if set; otherwise `Not a GitEHR repository or Store`.
- A repo command at a bare Store root → a helpful "you are at a Store root; cd into a subject repo" rather than a confusing failure.
- **Single-subject ergonomics:** when a Store holds exactly one repo, repo-level commands auto-target it, so a lone self-hoster (one person, or one pet) runs `gitehr journal add "…"` from the Store root with no cd and no subject name. As soon as a second subject exists, you `cd` (or `-C <repo>`) like git.

## All multi-subject operations live under `gitehr store`

- `gitehr store init` - bootstrap the Store, MPI, and first subject repo.
- `gitehr store add` - create and register a new subject repo.
- `gitehr store remove` - de-register a subject.
- `gitehr store list` - list subjects.
- (future) identifier resolution - `search`, `link`, `merge`, `path` - folded in as `gitehr store` subcommands, **not** a separate `gitehr mpi` command.

## Considered Options

- **Repo-first, Store as an optional overlay** (a lone record is just a repo; add a Store later): rejected because it special-cases the two personas, needs a "promote a standalone repo into a Store" migration, and the multi-subject reality (families, pets, clinics) means the Store is wanted by default anyway. One code path beats two.
- **A separate `gitehr mpi` command** for the index: rejected as needless surface and jargon. "Store" is friendlier and less technical, and every MPI operation is conceptually a Store operation, so they merge under `gitehr store`.
- **Run everything from the Store level** (repo commands take a `--patient`): rejected as un-git-like and verbose - you work on one subject at a time, so `cd` into it. Single-subject auto-targeting removes the only case where store-level execution would have helped.
- **Keep `gitehr store init` as a standalone-repo bootstrap**: rejected. With no existing users there is no back-compat cost, and a second entry point undercuts the single model. `gitehr store init` is the one front door.

## Consequences

- `store init` grows from "write an empty MPI" to a real bootstrap (Store + MPI + first repo), reusing the repo-scaffolding logic that currently lives in `gitehr store init`; `add` changes from "register an existing repo" to "create and register a new repo"; the top-level `init` command is removed. One shared repo-scaffolding implementation.
- Commands gain Store/repo **context detection** (walk up for `.gitehr/` and `gitehr-mpi.json`, then fall back to `GITEHR_STORE_PATH`/`store_path` config) plus the single-subject auto-target, replacing today's bare `.gitehr`-in-cwd checks.
- **Contributor scoping is deliberately left open** (per-repo `.gitehr/contributors.json` today vs a Store-level staff directory): parked until multi-subject workflows are fleshed out.
- Selection between independent Stores belongs to the GUI; the CLI continues to resolve one Store from its explicit execution context. See [ADR-0006](0006-multiple-stores-are-a-gui-concern.md).
- Docs gain a first-class self-hoster story - **families and pets** - alongside clinics, on the homepage and audience pages.
- No data migration: there are no existing users.
