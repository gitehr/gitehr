<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr remote

Manage named GitEHR remotes stored in `.gitehr/remotes.json`.

All subcommands require the current directory to be a GitEHR repository.

## `gitehr remote add <name> <url>`

Adds a new remote GitEHR repository.

Behavior:
- Fails if the repo is not a GitEHR repository.
- Fails if the remote name already exists.
- Persists the remote in `.gitehr/remotes.json` with an `added_at` timestamp.

## `gitehr remote remove <name>`

Removes an existing remote GitEHR repository.

Aliases: `rm`

Behavior:
- Fails if the repo is not a GitEHR repository.
- Fails if the named remote does not exist.

## `gitehr remote list`

Lists configured remotes.

Behavior:
- Prints a short help message if no remotes are configured.
- Defaults to `list` if no subcommand is provided.
