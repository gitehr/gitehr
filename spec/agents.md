# GitEHR Agents / LLMs Instructions

* When setting versions of dependencies do not rely on training data 'memory' for latest versions as these will be out of date. Always look up the latest versions from official sources.


## Big picture
- Monorepo with three main surfaces: Rust CLI (`cli/`), documentation site (`docs/` + `mkdocs.yml`), and GUI app (`gui/`, Tauri + React/Mantine).
- CLI manages on-disk EHR repos: a `.gitehr` marker + template folder structure from `folder-structure/` copied by `gitehr store init`.
- Journal is append-only: entries live in `journal/` with YAML front matter (`timestamp`, optional `author`, optional `documents`) and one Git commit per entry. Integrity derives from Git's own history, not a per-entry hash chain (see `cli/src/commands/journal/`, and `spec/repository-verification.md` for the planned policy checker / server-side guardian).

## Key paths & patterns
- CLI entrypoint: `src/main.rs` (clap subcommands). No-args prints version and help.
- Init flow: `cli/src/commands/scaffold.rs` copies the template and creates the first journal entry.
- Journal format: `journal/<YYYYMMDDTHHMMSS.mmmZ>-<UUID>.md` and YAML front matter delimited by `---`.
- YAML serialization: uses `serde_yaml_ng` (a maintained fork of the now-deprecated `serde_yaml`; we previously used `serde_yml`, which was withdrawn under RUSTSEC-2025-0068). Keep this consistent.
- GUI layout: Mantine `AppShell` in `gui/src/App.tsx` with styling in `App.css`.

## Dev workflows (project-specific)
- CLI build: `cargo build` from repo root; install via `cargo install --path .` (see `docs/developers/developers.md`).
- Manual CLI tests: create a throwaway repo, run `gitehr store init`, then `gitehr journal add "..."` (see `docs/developers/developers.md`).
- GUI dev: `cd gui && npm install && npm run tauri dev` (requires Tauri system deps; documented in developers guide).
- Docs: `docker compose up` runs Zensical on :8766 (see `docker-compose.yml`). Or locally: `pip install -r requirements.txt && zensical serve`.

## Conventions
- Repository template lives under `folder-structure/` and is copied verbatim on `gitehr store init`; update both template + CLI logic when needed.
- Use SHA-256 hashes for journal chain verification (`sha2` crate); verify scans all entries and maps hash -> filename.
- Keep command specs aligned with current CLI behavior.

## Integration points
- Docs theming is in `mkdocs.yml` + `docs/stylesheets/extra.css`.
- GUI uses Mantine components; visual tweaks typically go in `App.css` rather than inline styles.

## Testing

- All CLI commands should have unit tests.
- Typical GitEHR workflows should have integration tests.

## Overnight agents - task queue

When an agent session is left to pick up "suitable" work (e.g. overnight runs), start from these tasks, in this order. Each is self-contained, has an existing proven pattern or spec to follow, and needs no product decisions:

1. **#84 - Complete the typed-state primitives: medications, immunisations, family history.** Follow the shipped `allergies`/`vaccinations` typed-state pattern exactly (typed state file, subcommands, journal entry per mutation, integration tests). FHIR mappings are in the issue. Update `docs/cli/` and `spec/commands/` to match, and tick the corresponding part of R61/R84 on commit.
2. **#83 - Problem/condition list.** Design sketch already in `spec/problem-condition-list.md`; same typed-state shape plus a lifecycle (active/inactive/resolved/entered-in-error). Coordinate with #84 if both land the same night - separate branches, whichever merges second rebases.
3. **R31 - MCP prompt templates** (after #84/#83): `prompts/list` currently returns an empty array and `prompts/get` errors (see `cli/src/commands/mcp/server_impl/server.rs:189-216`). Implement the five templates named in `spec/mcp.md` (soap_note, discharge_summary, referral_letter, medication_review, consultation) as static templates with variable substitution - text-only, no clinical-content generation. Follow the audit.rs / resources.rs module pattern; tests per prompt.

Do NOT start these without Marcus: #86 (provenance - needs evidence-level vocabulary decisions), #88/#87 (depend on #85 primitives), #94 (external collaborator mid-experiment), #29 (typed-error architecture decision pending), #63/#81 (awaiting Marcus close/reply - comments are posted on each). #62 is closable after Marcus confirms `help` behaves consistently on his side.

Distribution blockers for Marcus, not agents: R54 needs an APT hosting decision (GitHub Pages vs Hetzner) plus a GPG signing key; R55 needs the `gitehr` package created in the AUR under his account (the release job is wired and will fail loudly at `git clone` until then); R56 signing needs Apple Developer + Microsoft EV accounts.

Rules of engagement for such runs: commit straight to main per repo convention, `s/lint` + `s/test` green before every push, conventional commits, and never claim a file was reviewed unless its contents actually arrived in your tool results (see memory: ACP tool results unreliable - trust `git diff` over narration).
