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

## Install a released CLI

After a GitEHR release is published to crates.io, install the CLI with:

```sh
cargo install gitehr --locked
```

This downloads the released source package from crates.io and compiles it locally. It does not download a prebuilt binary, so Rust remains a prerequisite.

## Install a prebuilt binary

Prebuilt archives, Windows MSI installers, and shell/PowerShell installer scripts are available from [GitHub Releases](https://github.com/gitehr/gitehr/releases). Use the assets for your operating system and architecture.

## Build from source

From the CLI project root:

```sh
cd gitehr
cargo build
```

This compiles the `gitehr` binary in debug mode under `target/debug/gitehr`.

To build and install the local checkout into `~/.cargo/bin` so it is available globally:

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

Install completion scripts for your current shell:

```sh
gitehr completions install
```

The installer detects your shell, writes the correctly named completion file, and prints any one-time shell setup still needed. Re-run it after upgrading GitEHR if your package manager or installer did not refresh completions for you.

## What's next

- [CLI Reference](../cli/cli.md) for command syntax.
- [Install the GUI](gui.md) if you want a graphical interface on top of the CLI.
