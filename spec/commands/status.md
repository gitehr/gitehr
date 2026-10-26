<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr status`

Aliases: `st`

### `gitehr status`

Displays the current status of the GitEHR repository.

Behavior:

- If the current directory is not a GitEHR repository, prints a message and suggests running `gitehr store init`.
- Prints repository version (from `.gitehr/GITEHR_VERSION`) when present.
- Shows encryption as "Not implemented" (R67/R68). A stale legacy `.gitehr/ENCRYPTED` marker changes the wording to say so, but is never reported as the repository being encrypted (R79/R82).
- Shows the number of journal entries.
- Lists state files (excluding `README.md`) and count.
- Shows git working directory status if the directory is a git repo, otherwise reports clean.
