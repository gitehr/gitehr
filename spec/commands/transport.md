<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr transport`

### `gitehr transport create [--output <path>] [--encrypt]`

Creates a compressed transport archive of the GitEHR repository using `tar.gz`.

Behavior:
- Requires the current directory to be a GitEHR repository.
- Includes `journal`, `state`, `imaging`, `documents`, `openehr`, `fhir`, and `.gitehr` directories (absent ones are skipped).
- Archives regular files only: symlinks are skipped, `.git/` is not included, and all entries are written with mode `0644`.
- Defaults output to `gitehr-transport-<timestamp>.tar.gz` if `--output` is not provided.
- If `--encrypt` is set, exits non-zero with a "not yet implemented" error (roadmap R68) and writes no archive (R81).

### `gitehr transport extract <archive> [--output <dir>]`

Extracts a transport archive into the target directory (defaults to current directory).

Behavior:
- Does not require the current directory to be a GitEHR repository; creates the output directory if needed and overwrites existing files.
- Unpacks via `tar::Archive::unpack`, which refuses entries escaping the output directory.
- Prints a message if a GitEHR repository is detected in the extracted files.

### `gitehr transport`

If run without a subcommand, prints usage for `create` and `extract`.
