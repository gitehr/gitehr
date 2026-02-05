<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr status`

**Aliases**: `st`

### `gitehr status`

Displays the current status of the GitEHR repository.

Behavior:
- If the current directory is not a GitEHR repository, prints a message and suggests running `gitehr init`.
- Prints repository version (from `.gitehr/GITEHR_VERSION`) when present.
- Shows encryption status based on `.gitehr/ENCRYPTED`.
- Shows the number of journal entries.
- Lists state files (excluding `README.md`) and count.
- Shows git working directory status if the directory is a git repo, otherwise reports clean.
