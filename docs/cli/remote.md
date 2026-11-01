# gitehr remote

Manage named remote GitEHR repositories for synchronisation. Remotes are stored in `.gitehr/remotes.json`.

All subcommands must be run from the root of a GitEHR repository (the directory containing `.gitehr/`); elsewhere they fail with `Not a GitEHR repository (or not in the repository root).` If no subcommand is given, defaults to `list`.

These remotes are GitEHR's own registry in `.gitehr/remotes.json`. They are separate from Git's remotes (`git remote`), and the URL is stored as given, without being validated or contacted.

## gitehr remote add

```text
gitehr remote add <name> <url>
```

Adds a named remote. The URL can be any location the underlying Git transport understands (`https://`, `git@host:path`, a local path, etc.). The entry records an `added_at` UTC timestamp. If a remote with that name already exists the command fails; run `gitehr remote remove <name>` first.

## gitehr remote remove

```text
gitehr remote remove <name>
```

Removes the named remote. Alias: `rm`. Fails if no remote has that name.

## gitehr remote list

```text
gitehr remote list
```

Lists configured remotes as `name -> url`, in no guaranteed order. If none are configured it prints a hint to use `gitehr remote add`. This is the default when no subcommand is given.
