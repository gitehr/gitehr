# Install

GitEHR is shipped as two main components: a Rust CLI (`gitehr`) and a Tauri-based GUI (`gitehr gui`). The CLI is the foundation. The GUI wraps the CLI and is the recommended interface for clinicians and patients.

## Pick your install path

- [**Install the CLI**](cli.md) - required for everyone. Small Rust binary. This is the foundation; everything else is built on top.
- [**Install the GUI**](gui.md) - recommended for clinicians and patients. Built on top of the CLI.

## What you will not find here

A pre-built distribution. GitEHR is in developer preview; for now you build the CLI and the GUI from source. Packaged binaries, system installers, and macOS/Windows bundles will follow once the on-disk format is stable.

## After install

- [CLI Reference](../cli/cli.md) - command syntax and behavior.
- [GUI Quick Start](../gui/quick-start.md) - first-time use, walkthrough of the main panels.
