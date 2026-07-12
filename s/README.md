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
