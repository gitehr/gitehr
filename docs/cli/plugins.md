# gitehr plugins

GitEHR is extensible the way Git is. Run `gitehr <name> ...` and, if `<name>` is not a built-in command, gitehr runs the executable `gitehr-<name>` from your `$PATH`, passing the rest of the arguments through. Drop an executable named `gitehr-<command>` onto your `PATH` and you have a new subcommand - no rebuild.

```console
$ gitehr hello world
Hello from a gitehr plugin. You passed: world
```

## `gitehr plugins`

Lists the plugins gitehr can see - every `gitehr-*` executable on your `PATH`, by the name you invoke it as.

```console
$ gitehr plugins
Installed plugins (run as `gitehr <name>`):

  hello            /usr/local/bin/gitehr-hello
```

Discovered plugins also appear in a "Plugins" section at the bottom of `gitehr --help`.

!!! note "Built-in commands always win"
    A plugin can never shadow a built-in command. `gitehr journal` always runs the built-in `journal`, even if a `gitehr-journal` is on your `PATH`. Plugin names are also restricted to simple tokens, so a crafted name cannot be turned into a file path. This keeps the plugin mechanism from becoming an attack vector.

## Writing a plugin

A plugin is any executable named `gitehr-<command>` on the `PATH`. For example, `gitehr-hello`:

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "Hello from a gitehr plugin. You passed: $*"
```

Make it executable (`chmod +x gitehr-hello`) and put it somewhere on your `PATH`. Good plugins:

- write results to **stdout** and diagnostics to **stderr**, so output pipes cleanly;
- exit `0` on success and non-zero on failure (gitehr propagates the exit code);
- handle their own `--help` (gitehr forwards `gitehr <command> --help` to the plugin);
- find the repository themselves by looking for `.gitehr/` in the working directory or an ancestor - gitehr does not pass repository context implicitly.

Choose a name that is not a built-in command (built-ins win, so a same-named plugin would be unreachable). Run `gitehr --help` to see the built-in commands.
