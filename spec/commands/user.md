<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr user`

Manage users in `.gitehr/contributors.json` and set the active author for journal entries.

All subcommands require the current directory to be a GitEHR repository.

### `gitehr user add <id> <name> [--role <role>] [--email <email>]`

Adds a contributor record and enables it by default.

Behavior:
- Fails if the contributor ID already exists.
- Records `added_at` timestamp.

### `gitehr user enable <id>`

Enables a contributor.

### `gitehr user disable <id>`

Disables a contributor and clears its active state.

### `gitehr user activate <id>`

Sets the contributor as the active author for future journal entries.

Behavior:
- Fails if the contributor is disabled.
- Clears any previously active contributor.

### `gitehr user deactivate`

Clears the current active contributor.

### `gitehr user list`

Lists users with their status: `[active]`, `[enabled]`, or `[disabled]`.

Alias: `gitehr contributor`

If no subcommand is provided, defaults to `list`.
