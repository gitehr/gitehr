# gitehr user

Manage contributors who can author journal entries. Contributors live in `.gitehr/contributors.json`. The single currently-active contributor is the author stamped onto new journal entries.

Alias: `gitehr contributor`.

All subcommands require the current directory to be a GitEHR repository. If no subcommand is given, defaults to `list`.

## gitehr user create

```text
gitehr user create
```

Interactive user creation. Prompts for name, email, and an optional public key. If no public key is provided, offers to generate an elliptic-curve key pair and stores the public key in `.gitehr/contributors.json`.

## gitehr user add

```text
gitehr user add <id> <name> [--role <role>] [--email <email>]
```

Adds a contributor record non-interactively and enables it by default. Fails if the ID already exists. Records the creation timestamp.

## gitehr user enable

```text
gitehr user enable <id>
```

Enables a previously-disabled contributor.

## gitehr user disable

```text
gitehr user disable <id>
```

Disables a contributor and clears its active state if it was the active contributor.

## gitehr user activate

```text
gitehr user activate <id>
```

Sets the named contributor as the active author for subsequent journal entries. Fails if the contributor is disabled. Clears any previously active contributor first.

## gitehr user deactivate

```text
gitehr user deactivate
```

Clears the current active contributor. New journal entries written while no contributor is active will record `author: (unknown)`.

## gitehr user list

```text
gitehr user list
```

Lists contributors with their status: `[active]`, `[enabled]`, or `[disabled]`.
