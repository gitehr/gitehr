# Install the CLI

Install a released `gitehr` binary for your operating system. Every release includes checksums in [`sha256.sum`](https://github.com/gitehr/gitehr/releases/latest/download/sha256.sum) and the full list of assets is on [GitHub Releases](https://github.com/gitehr/gitehr/releases).

## Install a released CLI

=== ":material-apple: macOS"

    **Homebrew** (recommended)

    ```sh
    brew tap pacharanero/tap
    brew install gitehr
    ```

    **Shell installer** - detects your chip, verifies the release checksum, and installs to `~/.local/bin`:

    ```sh
    curl -LsSf https://github.com/gitehr/gitehr/releases/latest/download/gitehr-installer.sh | sh
    ```

    **Standalone archive** - download an archive, extract it, then place `gitehr` on your `PATH`:

    [:material-download: Apple Silicon](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-aarch64-apple-darwin.tar.xz){ .md-button }
    [:material-download: Intel](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-x86_64-apple-darwin.tar.xz){ .md-button }

=== ":material-linux: Linux"

    **Homebrew**

    ```sh
    brew tap pacharanero/tap
    brew install gitehr
    ```

    **Shell installer** - detects your architecture, verifies the release checksum, and installs to `~/.local/bin`:

    ```sh
    curl -LsSf https://github.com/gitehr/gitehr/releases/latest/download/gitehr-installer.sh | sh
    ```

    **Standalone archive** - download an archive, extract it, then place `gitehr` on your `PATH`:

    [:material-download: x86_64](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-x86_64-unknown-linux-gnu.tar.xz){ .md-button }
    [:material-download: aarch64](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-aarch64-unknown-linux-gnu.tar.xz){ .md-button }

=== ":material-microsoft-windows: Windows"

    **MSI installer** (recommended)

    [:material-download: Download the Windows installer](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-x86_64-pc-windows-msvc.msi){ .md-button }

    **PowerShell installer** - downloads and verifies the release, then installs `gitehr` for the current user:

    ```powershell
    irm https://github.com/gitehr/gitehr/releases/latest/download/gitehr-installer.ps1 | iex
    ```

    **Standalone executable** - download the ZIP archive, extract `gitehr.exe`, and place it in a folder on your `PATH`:

    [:material-download: Download the Windows ZIP](https://github.com/gitehr/gitehr/releases/latest/download/gitehr-x86_64-pc-windows-msvc.zip){ .md-button }

=== ":material-language-rust: Cargo"

    With a [Rust toolchain](https://rustup.rs) installed:

    ```sh
    cargo install gitehr --locked
    ```

    Cargo downloads the released source package from crates.io and compiles it locally. Choose one of the operating-system tabs when you want a prebuilt binary instead.

## Build GitEHR from source

For contributors developing GitEHR itself, build and install the local checkout. [`mise`](https://mise.jdx.dev/) is recommended for managing the Rust toolchain:

```sh
mise install rust
mise use rust
```

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
