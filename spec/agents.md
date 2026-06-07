# GitEHR Agents / LLMs Instructions

* When setting versions of dependencies do not rely on training data 'memory' for latest versions as these will be out of date. Always look up the latest versions from official sources.


## Big picture
- Monorepo with three main surfaces: Rust CLI (`cli/`), documentation site (`docs/` + `mkdocs.yml`), and GUI app (`gui/`, Tauri + React/Mantine).
- CLI manages on-disk EHR repos: a `.gitehr` marker + template folder structure from `folder-structure/` copied by `gitehr init`.
- Journal is append-only: entries live in `journal/` with YAML front matter and SHA-256 hash chaining (see `src/commands/journal.rs`, `src/commands/verify.rs`).

## Key paths & patterns
- CLI entrypoint: `src/main.rs` (clap subcommands). No-args prints version and help.
- Init flow: `src/commands/init.rs` copies template + creates genesis entry with random seed hash.
- Journal format: `journal/<YYYYMMDDTHHMMSS.mmmZ>-<UUID>.md` and YAML front matter delimited by `---`.
- YAML serialization: uses `serde_yaml_ng` (a maintained fork of the now-deprecated `serde_yaml`; we previously used `serde_yml`, which was withdrawn under RUSTSEC-2025-0068). Keep this consistent.
- GUI layout: Mantine `AppShell` in `gui/src/App.tsx` with styling in `App.css`.

## Dev workflows (project-specific)
- CLI build: `cargo build` from repo root; install via `cargo install --path .` (see `docs/developers/developers.md`).
- Manual CLI tests: create a throwaway repo, run `gitehr init`, then `gitehr journal add "..."` (see `docs/developers/developers.md`).
- GUI dev: `cd gui && npm install && npm run tauri dev` (requires Tauri system deps; documented in developers guide).
- Docs: `docker compose up` runs Zensical on :8766 (see `docker-compose.yml`). Or locally: `pip install -r requirements.txt && zensical serve`.

## Conventions
- Repository template lives under `folder-structure/` and is copied verbatim on `gitehr init`; update both template + CLI logic when needed.
- Use SHA-256 hashes for journal chain verification (`sha2` crate); verify scans all entries and maps hash -> filename.
- Keep command specs aligned with current CLI behavior.

## Integration points
- Docs theming is in `mkdocs.yml` + `docs/stylesheets/extra.css`.
- GUI uses Mantine components; visual tweaks typically go in `App.css` rather than inline styles.

## Testing

- All CLI commands should have unit tests.
- Typical GitEHR workflows should have integration tests.
