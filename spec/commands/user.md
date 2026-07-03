<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr user`

Manage users in `.gitehr/contributors.json` and set the active author for journal entries.

All subcommands require the current directory to be a GitEHR repository.

### `gitehr user create`

Interactive user creation.

Prompts for:
- Name
- Email
- Public key (optional)

If no public key is provided, the CLI offers to generate an elliptic curve key pair and stores the public key in `.gitehr/contributors.json`.

### `gitehr user add <id> <name> [--role <role>] [--email <email>]`

Adds a user record and enables it by default.

Behavior:
- Fails if the user ID already exists.
- Records `added_at` timestamp.

### `gitehr user enable <id>`

Enables a user.

### `gitehr user disable <id>`

Disables a user and clears its active state.

### `gitehr user activate <id>`

Sets the user as the active author for future journal entries.

Behavior:
- Fails if the user is disabled.
- Clears any previously active user.

### `gitehr user deactivate`

Clears the current active user.

### `gitehr user list`

Lists users with their status: `[active]`, `[enabled]`, or `[disabled]`.

Alias: `gitehr contributor`

If no subcommand is provided, defaults to `list`.

## Planned configuration

Beyond the current name, email, and generated elliptic-curve public key, contributor configuration is intended to grow to cover:

- **Committer name** used for git commits, configured per repository (so journal commits carry a consistent, contributor-controlled identity).
- **GPG keys** as an alternative to the generated EC key pair, for contributors who already hold a GPG identity and want their journal commits GPG-signed.
- **Hardware-backed signing credentials** (for example YubiKey/PIV/smartcard, TPM-backed key, Secure Enclave, or equivalent) so contributors can keep private signing material off-device and present/unlock the credential when signing journal entries.
- **URI namespace link** - a stable external URI identifying the contributor (and/or the repository's identity namespace), linking the local contributor record to a resolvable identity rather than a bare local ID.
