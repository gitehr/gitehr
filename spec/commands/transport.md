<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr transport`

### `gitehr transport create [--output <path>] [--encrypt]`

Creates a compressed transport archive of the GitEHR repository using `tar.gz`.

Behavior:
- Requires the current directory to be a GitEHR repository.
- Includes `journal`, `state`, `imaging`, `documents`, and `.gitehr` directories.
- Defaults output to `gitehr-transport-<timestamp>.tar.gz` if `--output` is not provided.
- If `--encrypt` is set, prints a note that transport encryption is not yet implemented and continues unencrypted.

### `gitehr transport extract <archive> [--output <dir>]`

Extracts a transport archive into the target directory (defaults to current directory).

Behavior:
- Prints a message if a GitEHR repository is detected in the extracted files.

### `gitehr transport`

If run without a subcommand, prints usage for `create` and `extract`.
