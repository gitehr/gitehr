# Install the CLI

!!! note "Linux first"
    These instructions are written for Linux because that is the primary development platform. macOS should work with minor adjustments. Windows support is best-effort for now.

## Prerequisites

- `cargo` available on your PATH.

[`mise`](https://mise.jdx.dev/) is recommended as a general development toolchain manager. It handles the installation and versioning of Rust and many other languages. Once you have `mise` installed, install Rust with:

```sh
mise install rust
```

Then activate it in your shell:

```sh
mise use rust
```

## Build and install

From the CLI project root:

```sh
cd gitehr
cargo build
```

This compiles the `gitehr` binary in debug mode under `target/debug/gitehr`.

To build and install the release binary into `~/.cargo/bin` so it is available globally:

```sh
cd gitehr
cargo install --path .
```

Or use the helper script from the repo root:

```sh
s/install
```

## Verify

```sh
gitehr --help
gitehr version
```

You should see a short usage summary and the current GitEHR version.

## Shell completions

Generate a completion script and save it to your shell's completion directory.

```sh
gitehr completions bash > ~/.local/share/bash-completion/completions/gitehr
gitehr completions zsh > "${fpath[1]}/_gitehr"
gitehr completions fish > ~/.config/fish/completions/gitehr.fish
gitehr completions powershell | Out-File -Append $PROFILE
```

Restart your shell after installation.

## What's next

- [CLI Reference](../cli/cli.md) for command syntax.
- [Install the GUI](gui.md) if you want a graphical interface on top of the CLI.
