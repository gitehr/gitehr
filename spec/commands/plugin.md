<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr` plugins

GitEHR is extensible the way Git is: if you run `gitehr <name> ...` and `<name>` is not a built-in command, gitehr looks for an executable `gitehr-<name>` on your `$PATH` and runs it, forwarding the remaining arguments. Installing a binary or script onto your `PATH` adds a subcommand, with no need to recompile gitehr. The implementation is `cli/src/commands/plugin.rs`.

## How it resolves

1. **Built-in commands always win.** clap matches every defined subcommand and its aliases first (via `#[command(external_subcommand)]`); only a genuinely unknown subcommand falls through to plugin dispatch. A `gitehr-journal` on your `PATH` can never intercept `gitehr journal`. This is a security property, not just a convenience.
2. For an unknown `<name>`, gitehr resolves `gitehr-<name>` against `$PATH` (first match wins) and runs it with the remaining arguments.
3. On Unix the plugin **replaces** the gitehr process (`exec`), so its standard streams, signals, and exit code pass through transparently.
4. If `<name>` is not built in and no `gitehr-<name>` is on `PATH`, gitehr prints an `unrecognized subcommand` error that points at `gitehr plugins`, and exits non-zero.

Plugin names must be simple tokens (letters, digits, `-`, `_`); anything that could be coerced into a path (`/`, `..`) is rejected, so a crafted subcommand cannot escape the `gitehr-<name>` lookup.

## `gitehr plugins`

Lists installed plugins: every `gitehr-*` executable on `$PATH`, by the name you invoke (`gitehr-export` is listed as `export`). Names shadowed by a built-in are omitted, since they are unreachable as plugins. First match per name wins, in `PATH` order.

## `--help`

`gitehr --help` (and a bare `gitehr`) appends a "Plugins" section listing the discovered plugin names alongside the built-in commands, so installed extensions are discoverable. The section is omitted when no plugins are installed.

## Authoring a plugin

A plugin is any executable named `gitehr-<command>` on the user's `PATH`. To behave well:

- **Name** it `gitehr-<command>`, where `<command>` is a simple token, and not the name of a built-in command (built-ins win, so a same-named plugin is unreachable).
- **Arguments**: you receive everything after `<command>` as your own argv. Parse them yourself.
- **Streams**: write results to stdout and diagnostics to stderr, so callers can pipe your output cleanly - the same convention gitehr's own commands follow.
- **Exit codes**: exit `0` on success and non-zero on failure; gitehr propagates your exit code.
- **`--help`**: handle `gitehr <command> --help` yourself (gitehr forwards it), printing your own usage.
- **Repository context** is not passed implicitly. Find the repository the usual way (look for `.gitehr/` in the working directory or an ancestor), or read whatever the user passes you.

## Example

A minimal plugin, `gitehr-hello`, placed anywhere on `PATH` and made executable (`chmod +x`):

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "Hello from a gitehr plugin. You passed: $*"
```

Then:

```console
$ gitehr hello world
Hello from a gitehr plugin. You passed: world

$ gitehr plugins
Installed plugins (run as `gitehr <name>`):

  hello            /usr/local/bin/gitehr-hello
```

This is how features can ship independently of the core. A future `gitehr export` (FHIR/EHRxF bundles, per the roadmap) could be a plugin rather than a built-in, keeping the core small and letting extensions be installed and updated on their own.
