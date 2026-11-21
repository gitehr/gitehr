# gitehr user

Manage contributors who can author journal entries. Contributors live in `.gitehr/contributors.json`. The single currently-active contributor is the author stamped onto new journal entries.

Alias: `gitehr contributor`.

All subcommands run against the subject repository resolved from the current directory: the nearest enclosing directory containing `.gitehr/` (so they work from any subdirectory of a record), or the only subject when run at a Store root with exactly one subject. If no subcommand is given, defaults to `list`.

These commands only rewrite `.gitehr/contributors.json`; they do not create a Git commit or a journal entry.

## gitehr user create

```text
gitehr user create
```

Interactive user creation. Prompts for name and email (both required; blank answers are re-prompted) and an optional public key. Role is not prompted for; use `gitehr user add --role` if one is needed.

The user ID is derived rather than asked for: the local part of the email (before `@`) lowercased, with each run of non-alphanumeric characters replaced by a single `-`, falling back to the name and then to `user`. If that ID is taken, a random four-hex-digit suffix is appended (for example `jsmith-3fa2`). The chosen ID is printed.

A pasted public key is stored as given and is not validated. If the key is left blank, the command offers (default yes) to generate an Ed25519 key pair by running `ssh-keygen`, which must be installed. The private key is written **without a passphrase** to `~/.gitehr/keys/<id>_ed25519` (outside the repository) and the public key is stored in `.gitehr/contributors.json`. If key generation fails, no user is added.

## gitehr user add

```text
gitehr user add <id> <name> [--role <role>] [--email <email>]
```

Adds a contributor record non-interactively and enables it by default. Fails if the ID already exists. Records the creation timestamp.

## gitehr user enable

```text
gitehr user enable <id>
```

Enables a previously-disabled contributor. Fails if the ID is not found. Enabling does not make the contributor active; use `gitehr user activate`.

## gitehr user disable

```text
gitehr user disable <id>
```

Disables a contributor. Fails if the ID is not found. If it was the active contributor, it is also deactivated, so new entries are no longer attributed to it. A disabled contributor is never used as the author of new entries, even if `.gitehr/contributors.json` still names it as current (as older versions of this command left it).

## gitehr user activate

```text
gitehr user activate <id>
```

Sets the named contributor as the active author for subsequent journal entries and typed-state records (`recorded_by`). Fails if the ID is not found or the contributor is disabled. Clears any previously active contributor first.

## gitehr user deactivate

```text
gitehr user deactivate
```

Clears the current active contributor. Journal entries written while no contributor is active have no `author` field in their front matter, and typed-state records leave `recorded_by` empty.

## gitehr user list

```text
gitehr user list
```

Lists contributors sorted by ID, one per line as `<id> - <name> <role> <status>`, where status is `[active]`, `[enabled]`, or `[disabled]`. With no contributors it prints `No users configured.` and a hint to run `gitehr user add`.
