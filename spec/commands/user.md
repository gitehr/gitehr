<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr user`

Manage users in `.gitehr/contributors.json` and set the active author for journal entries.

All subcommands resolve the subject repository from the current directory (nearest `.gitehr/` ancestor, or the sole subject at a single-subject Store root). They only rewrite `.gitehr/contributors.json` and create no Git commit or journal entry.

### `gitehr user create`

Interactive user creation.

Prompts for:
- Name
- Email
- Public key (optional)

The user ID is derived from the email local part (slugified), falling back to the name and then `user`, with a random 4-hex-digit suffix on collision. A pasted public key is stored unvalidated. If no public key is provided, the CLI offers to generate an Ed25519 key pair via `ssh-keygen`; the private key is written without a passphrase to `~/.gitehr/keys/<id>_ed25519` and the public key is stored in `.gitehr/contributors.json`.

### `gitehr user add <id> <name> [--role <role>] [--email <email>]`

Adds a user record and enables it by default.

Behavior:
- Fails if the user ID already exists.
- Records `added_at` timestamp.

### `gitehr user enable <id>`

Enables a user. Fails if the ID is not found. Does not activate it.

### `gitehr user disable <id>`

Disables a user. Fails if the ID is not found. If it is the current contributor, `current_contributor` is cleared as well. The current-contributor lookup used to stamp `author`/`recorded_by` ignores a `current_contributor` that names a disabled user.

### `gitehr user activate <id>`

Sets the user as the active author for future journal entries.

Behavior:
- Fails if the user is not found or is disabled.
- Clears any previously active user.

### `gitehr user deactivate`

Clears the current active user. Entries written with no active user omit `author`; typed-state records leave `recorded_by` empty.

### `gitehr user list`

Lists users sorted by ID with their status: `[active]`, `[enabled]`, or `[disabled]`.

Alias: `gitehr contributor`

If no subcommand is provided, defaults to `list`.

## Planned configuration

Beyond the current name, email, and generated elliptic-curve public key, contributor configuration is intended to grow to cover:

- **Committer name** used for git commits, configured per repository (so journal commits carry a consistent, contributor-controlled identity).
- **GPG keys** as an alternative to the generated EC key pair, for contributors who already hold a GPG identity and want their journal commits GPG-signed.
- **Hardware-backed signing credentials** (for example YubiKey/PIV/smartcard, TPM-backed key, Secure Enclave, or equivalent) so contributors can keep private signing material off-device and present/unlock the credential when signing journal entries.
- **URI namespace link** - a stable external URI identifying the contributor (and/or the repository's identity namespace), linking the local contributor record to a resolvable identity rather than a bare local ID.
