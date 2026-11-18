# gitehr completions

```text
gitehr completions <shell>
gitehr completions --dir <path> <shell>
gitehr completions install [--shell <shell>] [--dir <path>]
```

Installs or generates shell completion scripts. Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

For the current user, prefer the installer:

```bash
gitehr completions install
```

It detects your shell from `$SHELL` (`bash`, `zsh`, `fish` or `elvish`), writes the correctly named completion file, and prints any one-time shell setup still needed. PowerShell is never detected automatically; use `gitehr completions install --shell powershell`.

For package managers or custom locations, generate or write a specific shell's completion script:

```bash
# bash
gitehr completions --dir ~/.local/share/bash-completion/completions bash

# zsh
gitehr completions --dir ~/.zfunc zsh

# fish
gitehr completions --dir ~/.config/fish/completions fish

# powershell
gitehr completions --dir ~/.config/powershell/completions powershell

# elvish
gitehr completions --dir ~/.elvish/lib elvish
```

Without `--dir`, the script is printed to stdout. Passing `--dir` without a shell is an error (`missing shell`); use `install` if you want the shell detected. Writing a completion file creates the directory if needed and replaces any existing file of the same name.

Restart your shell after installing or refreshing the completion script.
