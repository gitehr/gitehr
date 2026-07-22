<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr gui`

### `gitehr gui`

Launches the GitEHR graphical user interface (GUI) application.

Behavior:
- If launched outside a GitEHR repository or Store, the GUI can use the configured Store path (`GITEHR_STORE_PATH` or `store_path` in `gitehr config`) to find the patient index.
- The GUI selects one active Store at a time and can switch between independent Stores without changing the CLI's configured default Store. See [ADR-0006](../adr/0006-multiple-stores-are-a-gui-concern.md).
- Searches for a bundled GUI binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows).
- Falls back to `gitehr-gui` in PATH if no bundled binary exists.
- If no GUI binary is found, prints guidance on how to install or build it.
