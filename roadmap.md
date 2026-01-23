# GitEHR Roadmap (Spec Completion)

## Core CLI (Rust)
- [ ] Implement `gitehr init` binary bundling into `.gitehr/` (replace placeholder file created in `src/commands/init.rs`).
- [ ] Decide and implement final journal layout (current: `journal/<timestamp>-<uuid>.md`; draft spec: `journal/patients/<patient_id>/...`). Update spec and code together.
- [ ] Finalize YAML front matter schema for journal entries (current: `parent_hash`, `parent_entry`, `timestamp`, optional `author` in `src/commands/journal.rs`).
- [ ] Update journal verification to match final schema and delimiter rules (currently assumes `---` and `parent_entry`; see `src/commands/verify.rs`).
- [ ] Implement `gitehr journal add` file-based input if desired (spec currently uses content argument).
- [ ] Add author management for journal entries (currently TODO in `src/commands/journal.rs`).
- [ ] Implement `gitehr version` output to explicitly show shared CLI/GUI version if required beyond current `GitEHR <version>` string.

## Unimplemented CLI Commands (stubs)
- [ ] Implement `gitehr state` (currently stubbed in `src/main.rs`).
- [ ] Implement `gitehr remote` (stub).
- [ ] Implement `gitehr encrypt` (stub).
- [ ] Implement `gitehr decrypt` (stub).
- [ ] Implement `gitehr status` (stub).
- [ ] Implement `gitehr transport` (stub).
- [ ] Implement `gitehr contributor` (stub).
- [ ] Implement `gitehr gui` launcher (stub).
- [ ] Implement `gitehr upgrade` (stub; should include migrations + journal record).

## Repository Template & Structure
- [ ] Ensure `gitehr init` copies all template directories (`gitehr-folder-structure/`) and that template matches spec (including `/documents`).
- [ ] Decide on `.gitehr` internal files beyond `GITEHR_VERSION` and document them.

## GUI (Tauri + Mantine)
- [ ] Connect GUI actions to CLI commands (spec says GUI wraps CLI).
- [ ] Implement patient timeline, summaries, and stateful panels tied to on-disk `journal/` and `state/` data.
- [ ] Resolve layout overlap issues in Mantine UI across breakpoints (see `gui/gitehr-gui/src/App.tsx` + `App.css`).
- [ ] Add GUI “New Entry” workflow that writes journal entries using the finalized schema.

## Documentation & Spec Alignment
- [ ] Update `spec/spec.md` journal layout and YAML schema once finalized (draft section currently conflicts with implementation).
- [ ] Add/expand spec sections for repository lifecycle (“Initialization”, “Adding entries”, “Journal file contents”).
- [ ] Keep command specs aligned with actual CLI behavior (stubs vs implemented). 

## Tests & QA
- [ ] Add tests for new commands and finalized journal schema (existing tests only cover journal parsing/creation).
- [ ] Add tests for `gitehr init` artifact creation (template copy, version file, binary bundle).
- [ ] Add GUI smoke tests (manual checklist or automated) once data flows are implemented.

## Packaging & Distribution
- [ ] Decide how CLI + GUI are packaged together for end users (spec implies bundling inside repo).
- [ ] Document upgrade/migration strategy and version compatibility rules.
