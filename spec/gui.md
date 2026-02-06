# GitEHR GUI Launch

## Launch behavior

`gitehr gui` launches a compiled GUI binary when available.

Behavior:
- Uses `.gitehr/gitehr-gui` (or `.gitehr/gitehr-gui.exe` on Windows) if present.
- Falls back to `gitehr-gui` in PATH.
- If no GUI binary is found, prints guidance on how to install or build it.
