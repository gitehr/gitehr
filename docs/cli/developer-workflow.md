# Developer workflow

For contributors hacking on the CLI itself. End users should use [Install the CLI](../install/cli.md) instead.

## Repository layout

Relevant paths inside the CLI repo:

- `cli/` - Rust CLI project root
- `folder-structure/` - template copied by `gitehr init`
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

   gitehr init
   gitehr journal add "Test entry"
   ```

4. Inspect generated files and adjust implementation as needed.

## Manually testing `gitehr init`

1. Choose or create a directory to act as a test EHR repo (outside the CLI repo is recommended):

   ```sh
   cd /tmp
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

GitEHR follows semantic versioning (`MAJOR.MINOR.PATCH`) and keeps the canonical version in `cli/Cargo.toml` under the `[workspace.package]` section.

### Version bump policy

- **PATCH** (`x.y.z -> x.y.(z+1)`)
    - Backwards-compatible bug fixes or internal changes that do not affect the public CLI surface or on-disk data layout.
- **MINOR** (`x.y.z -> x.(y+1).0`)
    - Backwards-compatible feature additions.
    - New commands, new flags, or new fields that older clients can safely ignore.
- **MAJOR** (`x.y.z -> (x+1).0.0`)
    - Breaking changes to the CLI interface or on-disk format.
    - Anything that may invalidate existing EHR repos or tools built on top of GitEHR.

### Bumping the version

Use the helper script from the repo root:

```sh
s/version++ patch   # or: minor, major
```

This calls `cargo set-version` to rewrite the workspace version and creates a `v<version>` git tag on the current commit if the working tree is clean.

After bumping, rebuild and install as usual:

```sh
s/install
```

### Recommended workflow

1. Make and test your changes.
2. Decide the appropriate semver level (patch / minor / major).
3. From the repo root:

   ```sh
   s/version++ patch
   ```

4. Commit your changes, including the touched `Cargo.toml` and any code.
5. Push tags if you use them:

   ```sh
   git push --tags
   ```

This keeps the Cargo version, git history, and published binaries aligned.
