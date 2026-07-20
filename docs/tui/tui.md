# TUI

!!! info "Planned"
    The terminal user interface (TUI) is on the roadmap but not yet implemented. This page is a stub describing the intended shape.

## Why a TUI

The GitEHR GUI is a Tauri desktop application: rich, but heavy by design. It assumes a graphical session, a webview, and at least a few hundred megabytes of resident memory. There are clinical settings where none of those assumptions hold:

- Field clinics in humanitarian or military contexts running over slow SSH on a satellite link.
- Refurbished or low-spec hardware in resource-limited settings.
- Sysadmins and clinical informaticians operating over SSH into a repo-hosting server.
- Emergency fallback when the GUI is unavailable but a record still needs to be added.

A TUI is the right interface for these cases. It runs over SSH, fits in a small terminal, and behaves predictably without a graphics stack.

## Intended scope

The TUI will wrap the same CLI primitives the GUI uses. It will not implement any operation that does not exist in the CLI.

Likely first capabilities:

- Browse a patient record (journal timeline, state files).
- Read journal entries by entry ID or by search.
- Add a journal entry from a multi-line editor.
- View and edit state files.
- Show repository status and contributors.

## Technology

Probably [Ratatui](https://ratatui.rs/), the dominant Rust TUI framework. Ratatui is already used by tools that overlap GitEHR's audience (`gitui`, `lazygit`-via-language-bindings, various `cargo` UIs).

## Status

Not started. Tracked under the "GUI and UX" workstream of the [roadmap](https://github.com/gitehr/gitehr/blob/main/spec/roadmap.md). If you want to contribute, the right starting point is an issue describing the smallest useful TUI.
