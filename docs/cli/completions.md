# gitehr completions

```text
gitehr completions <shell>
gitehr completions --dir <path> <shell>
gitehr completions install [--shell <shell>] [--dir <path>]
```

Installs or generates shell completion scripts. Supported shells: `bash`, `zsh`, `fish`, `powershell`.

For the current user, prefer the installer:

```bash
gitehr completions install
```

It detects your shell, writes the correctly named completion file, and prints any one-time shell setup still needed.

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
```

Restart your shell after installing or refreshing the completion script.
