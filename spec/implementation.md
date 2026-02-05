# GitEHR Implementation Specification

## Overview

### CLI

- The GitEHR CLI is implemented in Rust, which enables a compiled, cross-platform binary to be shipped inside each GitEHR repository for portability.
- The CLI uses the `clap` crate for command-line argument parsing.
- The CLI commands are implemented in the `src/commands` directory, with each command in its own module. Specifications for each command are detailed in their respective files in the `spec/commands` directory.
- the CLI includes completions for popular shells (bash, zsh, fish, powershell) to enhance the user experience.

### GUI

- The GitEHR GUI is implemented in [Tauri](https://tauri.app/) (Rust backend) and [Mantine](https://mantine.dev/) (React frontend), as a cross-platform desktop application that can be bundled with the CLI binary in each GitEHR repository, so that users have a consistent graphical interface regardless of platform.
- The design of GitEHR is such that many other interfaces can be developed, all interacting with the same underlying repository structure, however shipping a 'default' GUI with the repository ensures the 'batteries are included' - all users have an easy way to get started.

## Versioning

- At the moment GitEHR uses semantic versioning (semver) for releases, with versions in the format `MAJOR.MINOR.PATCH`.
- The primary source of truth for the current version is the `version` field in `gitehr/Cargo.toml`.
- In future we may switch to a date-based versioning scheme (CalVer) in which versions will look something like `YYYY.MM.minor`), if that proves more intuitively informative for end users and developers.

## Licensing in source files

All source files contain an SPDX license identifier at the top of the file to enable automated license detection.

For non-code files and documentation, please use CC-BY-SA-4.0:

```html
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
```

For source code files, please use AGPL-3.0:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
```
