# GitEHR Roadmap (Spec Completion)

## Core CLI (Rust)

- [x] ~~Implement `gitehr init` binary bundling~~ - Copies current executable to `.gitehr/gitehr`.
- [x] ~~Decide and implement final journal layout~~ - Finalized: `journal/<timestamp>-<uuid>.md`. One repo = one patient.
- [x] ~~Finalize YAML front matter schema~~ - Finalized: `parent_hash`, `parent_entry`, `timestamp`, optional `author`.
- [x] ~~Update journal verification to match final schema~~ - Genesis entry detection implemented.
- [x] ~~Implement `gitehr journal add` file-based input~~ - Supports inline content, `--file <path>`, and stdin (`--file -`).
- [x] ~~Add author management for journal entries~~ - Journal entries now include active contributor as `author` field.
- [ ] Implement `gitehr version` output to explicitly show the Git version that is available as well as the CLI version.

## CLI Commands (formerly stubs - now implemented)

- [x] Implement `gitehr state` - subcommands: list, get, set for managing state files.
- [x] Implement `gitehr remote` - subcommands: add, remove (rm), list; stores in `.gitehr/remotes.json`.
- [x] Implement `gitehr encrypt` - placeholder implementation with encryption marker file.
- [x] Implement `gitehr decrypt` - removes encryption marker.
- [x] Implement `gitehr status` - shows repo version, encryption status, journal entries, state files, git changes.
- [x] Implement `gitehr transport` - subcommands: create (tar.gz), extract.
- [x] Implement `gitehr user` (alias: `gitehr contributor`) - subcommands: add, enable, disable, activate, deactivate, list.
- [x] Implement `gitehr gui` launcher - launches bundled or PATH GUI binary.
- [x] Implement `gitehr upgrade` - updates version file, bundled binary, records upgrade in journal.
- [x] Implement `gitehr upgrade-binary` - updates only the bundled binary.
- [x] Implement `gitehr journal show` - lists journal entries with pagination.

## Repository Template & Structure

- [x] ~~Ensure `gitehr init` copies all template directories~~ - Verified: all directories from `gitehr-folder-structure/` are copied correctly (documents, imaging, journal, state, .gitehr).
- [x] ~~Decide on `.gitehr` internal files~~ - Contains: `GITEHR_VERSION`, `gitehr` (bundled binary), `remotes.json`, `contributors.json`.

## GUI (Tauri + Mantine)

- [x] ~~Connect GUI actions to CLI commands~~ - Tauri backend wraps gitehr crate, React frontend uses API layer.
- [x] ~~Implement timeline, summaries, and stateful panels tied to on-disk `journal/` and `state/` data~~ - Real data flows from CLI to GUI.
- [x] ~~Resolve layout overlap issues in Mantine UI across breakpoints~~ - Removed conflicting padding-top CSS, added responsive breakpoints.
- [x] ~~Add GUI "New Entry" workflow that writes journal entries using the finalized schema~~ - Modal with textarea, saves via CLI.
- [x] ~~Add dynamic repo path detection with folder picker~~ - Auto-detects GitEHR repo on startup; shows folder picker if none found.

## Documentation & Spec Alignment

- [x] ~~Update `spec/spec.md` journal layout and YAML schema~~ - Updated to match implementation.
- [x] ~~Add/expand spec sections for repository lifecycle~~ - Added Initialization, Adding entries, Verification.
- [x] Keep command specs aligned with actual CLI behavior (stubs vs implemented).

## Tests & QA

- [x] ~~Add tests for new commands and finalized journal schema~~ - 86 tests total (83 passing, 3 ignored for unimplemented features).
- [x] ~~Add tests for `gitehr init` artifact creation~~ - Tests cover template copy, version file, binary bundle.
- [x] ~~Add GUI E2E tests~~ - WebDriverIO + tauri-driver tests in `gui/gitehr-gui/e2e/`. Tests initial load, journal entries, sidebar. Requires `webkit2gtk-driver` on Linux.

## Packaging & Distribution

- [ ] Decide how CLI + GUI are packaged together for end users (spec implies bundling inside repo).
- [ ] Document upgrade/migration strategy and version compatibility rules.

## Documentation Site

- [x] ~~Add documentation site using Material for MkDocs~~ - Added in `docs
- [ ] Expand documentation site with usage guides, CLI reference, GUI walkthroughs, and repository structure explanations.
- [ ] Ensure documentation site is kept up to date with any changes in CLI/GUI features or repository structure.
- [ ] Consider adding a "Getting Started" guide for new users to quickly learn how to initialize a repo, add entries, and use the GUI.
- [ ] Add troubleshooting section to documentation site for common issues and how to resolve them.
- [ ] Add contribution guidelines to documentation site for developers who want to contribute to GitEHR development.
- [ ] Add changelog and release notes section on documentation site to track changes across versions.

## Journal Command TODOs (from spec/commands/journal.md)

- Front matter parsing currently assumes the YAML is delimited by `---` and that non-genesis entries include `parent_entry`. If this changes, update verification logic accordingly.
- Add an option to `gitehr journal verify` for increased verbosity to show details of any verification failures (e.g., which entry is broken, expected vs actual parent hash/entry). This will be crucial for debugging integrity issues in the journal chain.

## Repo organisation

- [ ] gitehr CLI should be contained in a folder called cli/ This might be as simple as moving the src/ folder into cli/src/ and adjusting the Cargo.toml accordingly.
- [ ]
