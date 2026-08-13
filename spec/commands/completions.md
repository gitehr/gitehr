<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr completions`

Generates shell completions for `gitehr`. Implementation: [`cli/src/commands/completions.rs`](../../cli/src/commands/completions.rs).

### `gitehr completions install [--shell <shell>] [--dir <dir>]`

Writes a completion script to the default (or given) completion directory for the current user.

Behaviour:

- Shell defaults to whatever `$SHELL` points at (`bash`, `zsh`, `fish`, or `elvish`); pass `--shell` when it can't be detected (e.g. `powershell`, or `$SHELL` unset).
- Directory defaults per shell when `--dir`/`-d` is omitted:
  - `bash`: `$XDG_DATA_HOME/bash-completion/completions` (falls back to `~/.local/share/...`)
  - `zsh`: `~/.zfunc`
  - `fish`: `$XDG_CONFIG_HOME/fish/completions` (falls back to `~/.config/...`)
  - `powershell`: `~/.config/powershell/completions`
  - `elvish`: `~/.elvish/lib`
- Creates the directory if it doesn't exist, and writes the completion file (e.g. `_gitehr` for zsh, `gitehr.fish` for fish).
- Prints the path written, plus a shell-specific note: zsh needs its `fpath` updated before `compinit`, PowerShell needs the script dot-sourced from the profile; other shells just need a restart.

### `gitehr completions <shell> [--dir <dir>]`

Generates a completion script for `<shell>` (`bash`, `zsh`, `fish`, `powershell`, or `elvish`).

Behaviour:

- With `--dir`, writes the completion file into that directory (same filenames as `install`).
- Without `--dir`, prints the completion script to stdout, for manual redirection.

## Examples

```bash
$ gitehr completions install
Completion script written to: /home/user/.zfunc/_gitehr
Add this before `compinit` in ~/.zshrc if it is not already there:
  fpath=(/home/user/.zfunc $fpath)
Then restart zsh or run `autoload -Uz compinit && compinit`.

$ gitehr completions zsh --dir ~/.zfunc
Completion script written to: /home/user/.zfunc/_gitehr

$ gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr
```

Restart your shell after installing completions.
