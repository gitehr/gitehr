<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr state`

Manages the mutable clinical state files within the GitEHR repository.

All subcommands require the current directory to be a GitEHR repository.

### `gitehr state list`

Lists state files under the `state/` directory (excluding `README.md`).

Behavior:
- Prints last modified timestamps when available.
- If no files are present, prints a hint to use `gitehr state set`.

### `gitehr state get <filename>`

Prints the contents of a state file.

Behavior:
- Fails if the file does not exist in `state/`.

### `gitehr state set <filename> <content>`

Writes content to the specified state file, creating the `state/` directory if needed.

Behavior:
- Overwrites any existing file of the same name.

### `gitehr state`

If run without a subcommand, defaults to `list`.


