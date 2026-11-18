<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr completions`

Generates shell completions for `gitehr`. Implementation: [`cli/src/commands/completions.rs`](../../cli/src/commands/completions.rs).

### `gitehr completions install [--shell <shell>] [--dir <dir>]`

Writes a completion script to the default (or given) completion directory for the current user.

Behaviour:

- Shell defaults to the basename of `$SHELL` when it is `bash`, `zsh`, `fish`, or `elvish`; otherwise (e.g. `pwsh`, or `$SHELL` unset) the command fails with `could not detect shell` and `--shell` must be passed.
- Directory defaults per shell when `--dir`/`-d` is omitted, relative to `$HOME` (falling back to `%USERPROFILE%`; the command fails if neither is set):
  - `bash`: `$XDG_DATA_HOME/bash-completion/completions` (falls back to `~/.local/share/...`)
  - `zsh`: `~/.zfunc`
  - `fish`: `$XDG_CONFIG_HOME/fish/completions` (falls back to `~/.config/...`)
  - `powershell`: `~/.config/powershell/completions`
  - `elvish`: `~/.elvish/lib`
- Creates the directory if it doesn't exist, and writes the completion file (`gitehr` for bash, `_gitehr` for zsh, `gitehr.fish` for fish, `gitehr.ps1` for PowerShell, `gitehr.elv` for elvish), replacing any existing file of that name.
- Prints the path written, plus a shell-specific note: zsh needs its `fpath` updated before `compinit`, PowerShell needs the script dot-sourced from the profile; other shells just need a restart.

### `gitehr completions <shell> [--dir <dir>]`

Generates a completion script for `<shell>` (`bash`, `zsh`, `fish`, `powershell`, or `elvish`).

Behaviour:

- With `--dir`, writes the completion file into that directory (same filenames as `install`).
- Without `--dir`, prints the completion script to stdout, for manual redirection.
- No shell-specific setup note is printed in either case.
- `--dir` without a shell fails with ``missing shell; try `gitehr completions install` ``; `gitehr completions` with no arguments prints help.

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
