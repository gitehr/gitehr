# gitehr gui

```text
gitehr gui
```

Launches the GitEHR GUI application.

Behavior:

- Looks for a bundled GUI binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows).
- Falls back to `gitehr-gui` on `$PATH` if no bundled binary exists.
- If no GUI binary is found, prints guidance on how to install or build it.
- If launched outside a GitEHR repository, prints a warning and opens the GUI without repo context.

For installation, see [Install the GUI](../install/gui.md). For day-to-day GUI usage, see [GUI Quick Start](../gui/quick-start.md) and the [GUI overview](../gui/gui.md).
