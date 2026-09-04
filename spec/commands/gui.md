<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr gui`

### `gitehr gui [--allow-bundled]`

Launches the GitEHR graphical user interface (GUI) application.

Behavior:
- If launched outside a GitEHR repository or Store, the GUI can use the configured Store path (`GITEHR_STORE_PATH` or `store_path` in `gitehr config`) to find the patient index.
- The GUI selects one active Store at a time and can switch between independent Stores without changing the CLI's configured default Store. See [ADR-0006](../adr/0006-multiple-stores-are-a-gui-concern.md).
- Searches for `gitehr-gui` in PATH and launches it if found.
- A bundled GUI binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows) is only launched when `--allow-bundled` is passed, since a repository received from another party (a clone or a transport archive) can carry an untrusted executable at that path (roadmap R78).
- If no GUI binary can be launched, prints guidance on how to install or build one, or on re-running with `--allow-bundled`.
