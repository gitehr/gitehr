# `gitehr config`

Manage local machine configuration shared by the CLI and GUI.

## Config file

Default path:

- `$XDG_CONFIG_HOME/gitehr/config.toml`
- otherwise `~/.config/gitehr/config.toml`

`GITEHR_CONFIG` may point to a specific config file.

Current TOML schema:

```toml
store_path = "/home/marcus/gitehr-store"
document_whitelist = ["pdf", "jpg", "png"]
```

`GITEHR_STORE_PATH` overrides `store_path` for the current process.

`document_whitelist` is an optional list of file extensions (without the leading dot; matched case-insensitively) that [`gitehr import --mode documents`](import.md) will accept. When absent, any file format is accepted, matching the pre-whitelist behaviour.

## Commands

```text
gitehr config path
gitehr config show
gitehr config set-store <path>
```

`set-store` requires an existing Store root containing `gitehr-mpi.json`.

## Context resolution

The current directory remains authoritative when it is inside a subject repo or Store. Outside both, Store-level commands use the configured Store. Repo-level commands use the configured Store only when it has exactly one subject; multi-subject Stores still require the user to enter the subject repo they intend to work on.
