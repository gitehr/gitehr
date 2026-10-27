<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr decrypt`

### `gitehr decrypt [--key <source>]`

Removes a stale `.gitehr/ENCRYPTED` marker written by an earlier placeholder `gitehr encrypt`. Nothing was ever encrypted, so nothing is decrypted.

Options:
- `--key <source>`: Accepted but ignored until encryption is implemented.

Behavior:
- Fails if the current directory is not a GitEHR repository.
- Fails if no marker exists.
- Removes `.gitehr/ENCRYPTED` and prints a note that no data was encrypted.
