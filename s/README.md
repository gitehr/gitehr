# `s/`

The `s/` directory contains some simple convenience scripts to speed up and standardise working with this repository.

## `s/test`

Run the test suite (`cargo test` across the `gitehr` CLI and `gitehr-mcp`). Arguments pass through to `cargo test`.

- `s/test` - run everything
- `s/test -p gitehr-mcp` - just the MCP crate
- `s/test journal` - only tests matching `journal`

The clinical calculators live in the separate [clincalc](https://github.com/pacharanero/clincalc) repo and are tested there.

## `s/lint`

Run the formatting and clippy checks CI enforces (`cargo fmt --all --check`, then `cargo clippy --all-targets -- -D warnings`). Run it with `s/test` before committing. Arguments forward to clippy.

- `s/lint` - check everything
- `s/lint -p gitehr-mcp` - clippy just the MCP crate

To auto-apply fixes: `cargo fmt --all && cargo clippy --fix --all-targets --allow-dirty`.

## `s/install-hooks`

Install the tracked Git hooks for this checkout:

```
s/install-hooks
```

This sets `core.hooksPath=.githooks`. The current pre-commit hook runs `s/lint`, so formatting and clippy failures are caught before the commit is created. Hooks stay local to the checkout; they are not forced on contributors until they opt in.

## `s/version++`

Ship a release from a clean `main` checkout:

```
s/version++          # patch
s/version++ minor
s/version++ major
```

The script bumps the Rust workspace, GUI, and Tauri versions; regenerates lockfiles
and `CHANGELOG.md`; commits; and pushes to `main`. It never tags locally. The
`auto-tag.yml` workflow creates the `vX.Y.Z` tag after the bump commit lands and
then invokes the cargo-dist release workflow.

## `s/docs`

Serve the Zensical documentation site through Docker Compose, open it in the default browser, and follow its logs.

```
s/docs
GITEHR_DOCS_PORT=8010 s/docs
```

The script selects the first free port from 8000 to 8030 unless `GITEHR_DOCS_PORT` is set. It passes the host user and group IDs to Compose, so generated site files are owned by the user rather than `root`.

## `s/size`

Print a tidy table of GitEHR's disk footprint: the size of each release binary (as built and stripped - the real "what ships" figure), the `target/` build cache split into debug/release/total (your `cargo clean` / `cargo sweep` signal), and the repo on disk (`.git`, `gui/` node_modules, whole repo). Works from any directory with standard tools.

```
s/size            # the report
s/size --bloat    # also run `cargo bloat --release --crates` (needs cargo-bloat)
```

It flags when `target/` grows past 2 GB and points at `cargo sweep`/`cargo clean`. For deeper views, optionally `cargo install cargo-bloat` (what's in the binary) and `cargo install du-dust` (a visual `dust` tree).

## `s/generate`

Generate many GitEHR repos for performance testing. Runs from the store root and will prompt for confirmation.

Example:
```
s/generate -repos 10000 -journal-entries 1000
```

Optional:
```
s/generate -repos 100 -journal-entries 10 --gitehr ./target/debug/gitehr
```

Skip journal creation:
```
s/generate -repos 10000 -journal-entries 1 --no-journal
```

Parallel repo creation (use with care):
```
s/generate -repos 1000 -journal-entries 100 --parallel 4
```

## `s/demo-store`

Build a synthetic Store for demos, screenshots, and manual testing. Where `s/generate` makes bulk repos for benchmarking, this makes a small number of clinically coherent records: a multi-year adult record (hypertension, type 2 diabetes, a 2017 appendicectomy, a penicillin allergy) and a child record (eczema, asthma, the UK childhood immunisation schedule), with typed state, a document, and three contributors.

```
s/demo-store              # build .private/demo-store (gitignored)
s/demo-store --force      # replace an existing demo Store
s/demo-store --output /tmp/demo
```

It prints the Store path on stdout, so `cd $(s/demo-store)` works. Every invocation sets `GITEHR_STORE_PATH`, and the script refuses to build inside your configured Store or to delete a directory that is not a GitEHR Store.

All the data is invented, and deliberately impossible. People are named for a personality trait plus a British animal - Stoical Pipistrelle and her son Intrepid, seen by Dr Candid Kestrel - and live on a street named for a native plant, in a Leeds postcode district that does not exist, with NHS numbers from the reserved 999 range. The hospital is fictional too. Nobody is called this, so no demo record can be mistaken for a real person, and each record says so in its first journal entry. The scheme is borrowed from an Australian synthetic dataset whose patients are called things like "Resilient Brushtail Possum" of Eucalyptus Avenue.

Two things to know before using it for a demo:

- Journal entries are backdated through `gitehr import --mode journal`, which preserves each entry's original timestamp, so the timeline genuinely spans years.
- Typed-state commands write their own journal entries and cannot record them at the time the event clinically happened, so they are stamped today. About two thirds of the adult record's entries therefore carry today's date. Any view showing only the most recent entries will show none of the history until it can page back.
