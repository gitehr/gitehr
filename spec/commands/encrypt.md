<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr encrypt`

### `gitehr encrypt [--key <source>]`

Marks the repository as encrypted and records the key source. This is a placeholder implementation for future full-file encryption.

Options:
- `--key <source>`: Key source identifier (for example, `local` or a remote URL). Defaults to `local` when omitted.

Behavior:
- Fails if the current directory is not a GitEHR repository.
- Fails if the repository is already marked as encrypted.
- Writes `.gitehr/ENCRYPTED` with `encrypted_at` and `key_source` metadata.
- Prints the planned encryption scope (journal, state, imaging, documents) and notes that full encryption is pending.
