# Install

GitEHR is shipped as two main components: a Rust CLI (`gitehr`) and a Tauri-based GUI (`gitehr gui`). The CLI is the foundation. The GUI wraps the CLI and is the recommended interface for clinicians and patients.

## Pick your install path

- [**Install the CLI**](cli.md) - required for everyone. Choose a prebuilt binary for your operating system, or compile from source with Cargo. This is the foundation; everything else is built on top.
- [**Install the GUI**](gui.md) - recommended for clinicians and patients. Built on top of the CLI.

## CLI releases

GitEHR publishes `gitehr` to crates.io and as prebuilt Linux, macOS, and Windows release assets. Rust users can also install the released source with `cargo install gitehr --locked`.

Use the [CLI installation guide](cli.md) to choose an operating-system installer, a standalone archive, Homebrew, or a source build.

## After install

- [CLI Reference](../cli/cli.md) - command syntax and behavior.
- [GUI Quick Start](../gui/quick-start.md) - first-time use, walkthrough of the main panels.
