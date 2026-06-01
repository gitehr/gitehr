# gitehr remote

Manage named remote GitEHR repositories for synchronisation. Remotes are stored in `.gitehr/remotes.json`.

All subcommands require the current directory to be a GitEHR repository. If no subcommand is given, defaults to `list`.

## gitehr remote add

```text
gitehr remote add <name> <url>
```

Adds a named remote. The URL can be any location the underlying Git transport understands (`https://`, `git@host:path`, a local path, etc.).

## gitehr remote remove

```text
gitehr remote remove <name>
```

Removes the named remote. Alias: `rm`.

## gitehr remote list

```text
gitehr remote list
```

Lists configured remotes. This is the default when no subcommand is given.
