# gitehr gui

```text
gitehr gui [--allow-bundled]
```

Launches the GitEHR GUI application.

Behavior:

- Looks for `gitehr-gui` on `$PATH` and launches it if found.
- A bundled GUI binary at `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows) is only
  launched when `--allow-bundled` is passed. A GitEHR repository received from another party (a
  clone or a transport archive) can carry an untrusted executable at that path, so it is never run
  automatically - without `--allow-bundled`, its presence is reported in the error instead.
- If no GUI binary can be launched, prints guidance on how to install or build one, or how to
  re-run with `--allow-bundled` if a bundled binary was found but not trusted yet.
- If launched outside a GitEHR repository, prints a warning and opens the GUI without repo context.

For installation, see [Install the GUI](../install/gui.md). For day-to-day GUI usage, see [GUI Quick Start](../gui/quick-start.md) and the [GUI overview](../gui/gui.md).
