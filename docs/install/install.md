# Install

GitEHR is shipped as two main components: a Rust CLI (`gitehr`) and a Tauri-based GUI (`gitehr gui`). The CLI is the foundation. The GUI wraps the CLI and is the recommended interface for clinicians and patients.

## Pick your install path

- [**Install the CLI**](cli.md) - required for everyone. Small Rust binary. This is the foundation; everything else is built on top.
- [**Install the GUI**](gui.md) - recommended for clinicians and patients. Built on top of the CLI.

## CLI releases

GitEHR will publish the `gitehr` CLI to crates.io. Once published, Rust users can install the released version with `cargo install gitehr --locked`; Cargo downloads the crate source and compiles it locally.

Prebuilt release binaries and operating-system installers are a separate distribution path. The release pipeline is configured for them, but no GitHub release assets are available yet. Until the first packaged release is published, build from source as described in the CLI and GUI guides.

## After install

- [CLI Reference](../cli/cli.md) - command syntax and behavior.
- [GUI Quick Start](../gui/quick-start.md) - first-time use, walkthrough of the main panels.
