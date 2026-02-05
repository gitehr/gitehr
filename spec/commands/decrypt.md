<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr decrypt`

### `gitehr decrypt [--key <source>]`

Removes the encryption marker for the repository. This is a placeholder implementation for future full-file decryption.

Options:
- `--key <source>`: Key source identifier (for example, `local` or a remote URL). Defaults to `local` when omitted.

Behavior:
- Fails if the current directory is not a GitEHR repository.
- Fails if the repository is not marked as encrypted.
- Removes `.gitehr/ENCRYPTED` and prints a note that full decryption is pending.
