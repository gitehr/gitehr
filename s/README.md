# `s/`

The `s/` directory contains some simple convenience scripts to speed up and standardise working with this repository.

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
