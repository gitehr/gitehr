# Install the GUI

The GUI is built with [Tauri](https://tauri.app/) (Rust backend) and React + Mantine (frontend). It wraps the CLI and is the recommended interface for clinicians and patients.

!!! note "CLI first"
    [Install the CLI](cli.md) before installing the GUI. The GUI shells out to the CLI for all data operations.

## Prerequisites

- `cargo` available on your PATH.
- `npm` (Node.js) available on your PATH for the GUI tooling.

### Linux system dependencies

The Tauri GUI on Linux needs WebKit2GTK and a few other system libraries. The list below is for Debian and Ubuntu; adjust for your distribution.

```sh
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

See the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/) for the current package names for your distribution and any additional requirements.

## Run from source

From the repo root:

```sh
s/gui-dev
```

This starts the GUI in development mode against the local source. It is equivalent to running `npm run tauri dev` inside `gui/`.

## What's next

- [GUI Quick Start](../gui/quick-start.md) - first-time use.
- [GUI overview](../gui/gui.md) - walkthrough of the main panels.
