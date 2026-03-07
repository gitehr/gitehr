# CLI Reference

!!! note "Developer tool"
    The CLI is intended for developers, automation, and testing. Clinicians and patients should use the GUI.

## Basics

- Running `gitehr` with no arguments prints the version and help.
- Most commands require a GitEHR repository in the current directory (presence of `.gitehr`).

## Commands

### `gitehr init`

Initializes a GitEHR repository in the current directory.

Behavior:
- Creates `.gitehr/` and writes `.gitehr/GITEHR_VERSION`.
- Copies the repository template from `gitehr-folder-structure/`.
- Bundles the CLI binary at `.gitehr/gitehr`.
- Creates a genesis journal entry seeded with a random hash.

### `gitehr journal add [content] [--file <path>]`

Adds a journal entry to `journal/`.

Behavior:
- Requires a GitEHR repository.
- Uses the latest journal entry hash as `parent_hash`.
- Accepts inline content, a file path, or stdin with `--file -`.
- Rejects using both inline content and `--file` at the same time.

Example:

```sh
gitehr journal add "Patient reviewed and plan updated."
gitehr journal add --file note.md
cat note.md | gitehr journal add --file -
```

### `gitehr journal show [options]`

Lists journal entries with metadata and preview.

Options:
- `-n, --limit <N>`: maximum entries (default 10)
- `-o, --offset <N>`: skip entries (default 0)
- `-r, --reverse`: newest first
- `-a, --all`: show all (ignores limit)

### `gitehr journal verify`

Verifies the journal hash chain using YAML front matter and SHA-256.

### `gitehr state list`

Lists state files under `state/` (excluding `README.md`).

### `gitehr state get <filename>`

Prints the contents of a state file.

### `gitehr state set <filename> <content>`

Writes content to a state file, creating `state/` if needed.

### `gitehr user`

Manage users that can author journal entries. Alias: `gitehr contributor`.

Subcommands:
- `gitehr user create` (interactive)
- `gitehr user add <id> <name> [--role <role>] [--email <email>]`
- `gitehr user enable <id>`
- `gitehr user disable <id>`
- `gitehr user activate <id>`
- `gitehr user deactivate`
- `gitehr user list`

### `gitehr remote add <name> <url>`

Adds a named remote and stores it in `.gitehr/remotes.json`.

### `gitehr remote remove <name>`

Removes a named remote. Alias: `rm`.

### `gitehr remote list`

Lists configured remotes. This is the default when no subcommand is provided.

### `gitehr encrypt [--key <source>]`

Marks the repo as encrypted (placeholder implementation).

Behavior:
- Writes `.gitehr/ENCRYPTED` with `encrypted_at` and `key_source`.
- Prints a note that full encryption is pending.

### `gitehr decrypt [--key <source>]`

Removes `.gitehr/ENCRYPTED` (placeholder implementation).

### `gitehr status`

Shows repository status:
- Repository version from `.gitehr/GITEHR_VERSION`
- Encryption state
- Journal entry count
- State file list
- Git working directory changes (if the repo is a git repo)

Alias: `st`

### `gitehr transport create [--output <path>] [--encrypt]`

Creates a `tar.gz` archive containing `journal/`, `state/`, `imaging/`, `documents/`, and `.gitehr/`.

Notes:
- `--encrypt` prints a warning that transport encryption is not implemented.

### `gitehr transport extract <archive> [--output <dir>]`

Extracts a transport archive to the target directory (default: current directory).

### `gitehr gui`

Launches the GUI if a binary is available.

Behavior:
- Uses `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows) if present.
- Falls back to `gitehr-gui` in PATH.
- Prints guidance if the GUI is not found.

### `gitehr upgrade`

Updates `.gitehr/GITEHR_VERSION`, re-bundles the CLI binary, and records an upgrade journal entry.

### `gitehr upgrade-binary`

Updates the bundled binary and writes `.gitehr/GITEHR_VERSION` without recording a journal entry.

### `gitehr version`

Prints the CLI version string as `GitEHR <version>`.

### `gitehr completions <shell>`

Generates shell completion scripts for bash, zsh, fish, or powershell.

Example installation:

```sh
gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr
gitehr completions zsh > "${fpath[1]}/_gitehr"
gitehr completions fish > ~/.config/fish/completions/gitehr.fish
gitehr completions powershell | Out-File -Append $PROFILE
```

Restart your shell after installation.
