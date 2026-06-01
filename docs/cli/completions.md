# gitehr completions

```text
gitehr completions <shell>
```

Generates shell completion scripts. Supported shells: `bash`, `zsh`, `fish`, `powershell`.

The script is printed to stdout; redirect it to your shell's completion directory.

```bash
# bash
gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr

# zsh
gitehr completions zsh > "${fpath[1]}/_gitehr"

# fish
gitehr completions fish > ~/.config/fish/completions/gitehr.fish

# powershell
gitehr completions powershell | Out-File -Append $PROFILE
```

Restart your shell after installing the completion script.
