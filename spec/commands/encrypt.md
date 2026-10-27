<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr encrypt`

### `gitehr encrypt [--key <source>]`

Not yet implemented (R67/R68). Always refuses, so the repository is never falsely marked as encrypted (R79).

Options:
- `--key <source>`: Accepted but ignored until encryption is implemented.

Behavior:
- Fails if the current directory is not a GitEHR repository.
- Fails if a legacy `.gitehr/ENCRYPTED` marker exists ("already encrypted").
- Otherwise fails with a "not yet implemented" message and changes no files.
