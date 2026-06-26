# Developer workflow

For contributors hacking on the CLI itself. End users should use [Install the CLI](../install/cli.md) instead.

## Repository layout

Relevant paths inside the CLI repo:

- `cli/` - Rust CLI project root
- `folder-structure/` - template copied by `gitehr store init`
  - `README.md` - root repository README template
  - `journal/` - journal directory template
  - `state/` - clinical state directory template
  - `imaging/` - imaging directory template
- `cli/src/commands/` - Rust implementations of CLI commands
  - `init.rs` - repository initialization logic (creates folders + genesis entry)
  - `journal.rs` - journal entry creation and chaining

## Typical local dev workflow

From the repo root:

1. Edit code under `cli/src/`.

2. Build and run tests:

   ```sh
   cargo build
   cargo test
   ```

3. Install and run against a throwaway test repo:

   ```sh
   s/install

   cd /tmp
   rm -rf test-ehr
   mkdir test-ehr
   cd test-ehr

   gitehr store init
   gitehr journal add "Test entry"
   ```

4. Inspect generated files and adjust implementation as needed.

## Manually testing `gitehr store init`

1. Choose or create a directory to act as a test EHR repo (outside the CLI repo is recommended):

   ```sh
   cd /tmp
   rm -rf test-ehr
   mkdir test-ehr
   cd test-ehr
   ```

2. Run init:

   ```sh
   gitehr store init
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

## Manually testing `gitehr journal add`

From an initialized test repo (see above):

```sh
cd /tmp/test-ehr
gitehr journal add "First clinical entry"
ls journal
cat journal/*.md
```

Confirm for the newest entry:

- `parent_hash` is different from the genesis seed hash
- `parent_hash` matches the SHA-256 hash of the full contents of the previous entry file
- The body contains `First clinical entry`

Optional - verify the hash manually:

```sh
# Replace GENESIS_FILE with the actual filename
GENESIS_FILE=$(ls journal | sort | head -n1)
NEWEST_FILE=$(ls journal | sort | tail -n1)

HASH=$(sha256sum "journal/$GENESIS_FILE" | awk '{print $1}')
echo "Genesis hash: $HASH"

grep parent_hash "journal/$NEWEST_FILE"
```

The `parent_hash` in the newest file should match `HASH`.

## Versioning

GitEHR follows semantic versioning (`MAJOR.MINOR.PATCH`) and keeps the canonical Rust workspace version in the root `Cargo.toml` under `[workspace.package]`.

### Version bump policy

- **PATCH** (`x.y.z -> x.y.(z+1)`)
    - Backwards-compatible bug fixes or internal changes that do not affect the public CLI surface or on-disk data layout.
- **MINOR** (`x.y.z -> x.(y+1).0`)
    - Backwards-compatible feature additions.
    - New commands, new flags, or new fields that older clients can safely ignore.
- **MAJOR** (`x.y.z -> (x+1).0.0`)
    - Breaking changes to the CLI interface or on-disk format.
    - Anything that may invalidate existing EHR repos or tools built on top of GitEHR.

### Release automation

GitEHR uses `release-plz` for normal releases. Because GitEHR is currently released as binaries rather than published to crates.io, `release-plz.toml` sets `publish = false` and `git_only = true`; the existing `vX.Y.Z` tags are the release source of truth.

On every push to `main`, `.github/workflows/release-plz.yml` opens or refreshes a Release PR. That PR bumps the Cargo workspace version, rewrites `CHANGELOG.md` from conventional commits, and is the only version-bump PR maintainers should normally merge. When the Release PR is merged, release-plz creates the `vX.Y.Z` tag and GitHub Release.

The workflow uses the `RELEASE_PLZ_TOKEN` repository secret rather than the default `GITHUB_TOKEN`, so Release PRs trigger the normal CI checks. Until that secret exists, the workflow skips cleanly.

### Recommended workflow

1. Make and test changes locally.
2. Commit using conventional commits (`fix:`, `feat:`, `feat!:` / `BREAKING CHANGE:`, `docs:`, `ci:`, etc.).
3. Merge to `main`.
4. Review and merge the release-plz Release PR.
5. Let release-plz create the tag and GitHub Release. Do not create release tags locally.

### Manual fallback

`s/version++` is kept only as an explicit local fallback for bypassing release-plz. It no longer creates tags.

```sh
s/version++ --manual patch   # or: minor, major
```

The fallback bumps the root Cargo workspace version and the GUI version files (`gui/src-tauri/Cargo.toml`, `gui/src-tauri/tauri.conf.json`, `gui/package.json`, `gui/package-lock.json`). Commit those changes manually if you use it.
