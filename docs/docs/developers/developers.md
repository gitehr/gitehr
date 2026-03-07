# GitEHR Developer Guide

This document explains how to build and manually test GitEHR. Initially it focuses on the Linux platform, because that is what I'm using, but I will expand this to add other platforms as I can.

## CLI reference

See [CLI Reference](cli.md) for command syntax and behavior. The CLI is intended for developers and automation, not clinical end users.

## Prerequisites (CLI)

- `cargo` available on your PATH

I can recommend the use of [`mise`](https://mise.jdx.dev/) as general development toolchain manager, it can handle the installation and versioning of Rust and many other languages. once you have `mise` installed, you can install Rust with:

```sh
mise install rust
```

Then activate it in your shell:

```sh
mise use rust
```

## Prerequisites (GUI)

- `cargo` available on your PATH

- `npm` (Node.js) available on your PATH for the GUI tooling


To build/run the Tauri GUI on Linux, install the system dependencies listed in the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/). The following is an example for Debian-derived distros:

```sh
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

See https://tauri.app/guides/prerequisites/#linux for the current, distro-specific package names and any additional requirements.

## Building the CLI

From the CLI project root:

```sh
cd gitehr
cargo build
```

This compiles the `gitehr` binary in debug mode under `target/debug/gitehr`.

To build and install the release binary into `~/.cargo/bin` so it is available globally:

```sh
cd gitehr
cargo install --path .
```

After install, you should be able to run:

```sh
gitehr --help
```

## Shell completions

Generate a completion script and save it to your shell's completion directory.

```sh
gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr
gitehr completions zsh > "${fpath[1]}/_gitehr"
gitehr completions fish > ~/.config/fish/completions/gitehr.fish
gitehr completions powershell | Out-File -Append $PROFILE
```

Restart your shell after installation.

## Repository Layout (CLI)

Relevant paths inside the CLI repo:

- `gitehr/` – Rust CLI project root
- `gitehr/gitehr-folder-structure/` – template copied by `gitehr init`
  - `README.md` – root repository README template
  - `journal/` – journal directory template
  - `state/` – clinical state directory template
  - `imaging/` – imaging directory template
- `gitehr/src/commands/` – Rust implementations of CLI commands
  - `init.rs` – repository initialization logic (creates folders + genesis entry)
  - `journal.rs` – journal entry creation and chaining

From the CLI crate root (where `Cargo.toml` lives):

```sh
cd gitehr
cargo set-version --bump patch   # or: minor, major
```

```sh
cargo build      # or: cargo install --path .
```

### Recommended workflow

1. Make and test your changes (see sections above).
2. Bump the version:

   ```sh
   s/version++
   ```
   Or for a different bump level:

   ```sh
   s/version++ minor   # or: major
   ```
3. Commit your changes, including `Cargo.toml` and any code.
4. If using git tags, push them as appropriate, for example:

   ```sh
   git push --tags
   ```

This keeps the Cargo version, git history, and published binaries aligned.

- Creates - `.gitehr/` - `journal/` - `state/`
  Use the built-in Cargo command instead: - `imaging/` - `README.md` (root)
- Creates a _genesis_ journal entry in `journal/` with:
  - A filename of the form `YYYYMMDDTHHMMSSZ-<UUID>.md`
  - YAML front matter including:
    - `parent_hash`: a random 32‑byte seed hashed with SHA‑256
    - `timestamp`: current UTC time
  - Body text: `Genesis entry for GitEHR repository`

**How to manually test:**

1. Choose or create a directory to act as a test EHR repo (outside the CLI repo is recommended):

   ```sh
   cd /home/marcus/code/gitehr
   rm -rf test-ehr
   mkdir test-ehr
   cd test-ehr
   ```

2. Run init:

   ```sh
   gitehr init
   ```

3. Verify structure:

   ```sh
   ls -la
   ls -la journal state imaging .gitehr
   ```

   You should see:
   - `.gitehr/` directory
   - `journal/`, `state/`, `imaging/` directories
   - `README.md` in the root copied from the template

4. Inspect the genesis journal entry:

   ```sh
   ls journal
   cat journal/*.md
   ```

   Confirm:
   - Exactly one `.md` file exists (if you only ran `init` once)
   - YAML front matter contains `parent_hash` (not `null`) and `timestamp`
   - The body text matches the genesis description

### 2. `gitehr journal add`

Adds a new clinical entry to the `journal/` directory of the current GitEHR repository.

**What it does:**

- Requires that a `journal/` directory already exists (normally created by `gitehr init`)
- Reads the latest journal entry (by filename sort order) and computes its SHA‑256 hash
- Creates a new journal entry file with:
  - A filename of the form `YYYYMMDDTHHMMSSZ-<UUID>.md`
  - YAML front matter including:
    - `parent_hash`: the hash of the latest existing entry
    - `timestamp`: current UTC time
  - Body text: the string you passed on the command line

**How to manually test:**

1. From an initialized test repo (see `gitehr init` steps above):

   ```sh
   cd /home/marcus/code/gitehr/test-ehr
   gitehr journal add "First clinical entry"
   ```

2. List the journal files:

   ```sh
   ls journal
   ```

   You should see at least two `.md` files now: one genesis entry and one for the new entry.

3. Inspect the files:

   ```sh
   cat journal/*.md
   ```

   Confirm for the newest entry:
   - `parent_hash` is **different** from the genesis seed hash
   - `parent_hash` matches the SHA‑256 hash of the full contents of the previous entry file
   - The body contains `First clinical entry` (or your provided text)

4. (Optional) Manually verify the hash:

   ```sh
   # Replace GENESIS_FILE with the actual filename
   GENESIS_FILE=$(ls journal | sort | head -n1)
   NEWEST_FILE=$(ls journal | sort | tail -n1)

   # Compute hash of the genesis file
   HASH=$(sha256sum "journal/$GENESIS_FILE" | awk '{print $1}')
   echo "Genesis hash: $HASH"

   # Show parent_hash from newest file
   grep parent_hash "journal/$NEWEST_FILE"
   ```

   The `parent_hash` in the newest file should match `HASH`.

## Typical Local Dev Workflow

From the CLI repo root (`gitehr/`):

1. Edit code:

   ```sh
   cd gitehr
   # edit src/*
   ```

2. Build + run tests (none yet, but build verifies compilation):

   ```sh
   cargo build
   ```

3. Install and run against a throwaway test repo:

   ```sh
   cargo install --path .

   cd ..
   rm -rf test-ehr
   mkdir test-ehr
   cd test-ehr

   gitehr init
   gitehr journal add "Test entry"
   ```

4. Inspect generated files and adjust implementation as needed.

## Versioning

GitEHR follows semantic versioning (`MAJOR.MINOR.PATCH`) and keeps the canonical version in `gitehr/Cargo.toml` under the `[package]` section.

### Version bump policy

- **PATCH** (`x.y.z -> x.y.(z+1)`)
  - Backwards-compatible bug fixes or internal changes that don’t affect the public CLI surface or on‑disk data layout.
- **MINOR** (`x.y.z -> x.(y+1).0`)
  - Backwards-compatible feature additions.
  - New commands, new flags, or new fields that older clients can safely ignore.
- **MAJOR** (`x.y.z -> (x+1).0.0`)
  - Breaking changes to the CLI interface or on-disk format.
  - Anything that may invalidate existing EHR repos or tools built on top of GitEHR.

### Bumping the version

Use the helper script in the repo root (parent of `gitehr/`):

```sh
cd /home/marcus/code/gitehr
./scripts/bump_version.sh patch   # or: minor, major
```

What the script does:

1. Reads the current version from `gitehr/Cargo.toml`.
2. Computes the new version based on the argument (`major` / `minor` / `patch`).
3. Rewrites the `version = "…"` field under `[package]` in `gitehr/Cargo.toml`.
4. If the directory is a git repo, creates a tag `vNEW_VERSION` on the current commit.

After bumping, rebuild/install as usual:

```sh
cd gitehr
cargo build      # or: cargo install --path .
```

### Recommended workflow

1. Make and test your changes (see sections above).
2. Decide the appropriate semver level (patch/minor/major).
3. From the repo root:

   ```sh
   ./scripts/bump_version.sh patch   # or minor/major
   ```

4. Commit your changes, including `gitehr/Cargo.toml` and any code.
5. If using git tags, push them:

   ```sh
   git push --tags
   ```

This keeps the Cargo version, git history, and published binaries aligned.
