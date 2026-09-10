# gitehr config

`gitehr config` manages local machine settings used by the CLI and GUI.

GitEHR still prefers the current working directory when it is already inside a subject repo or Store. The configured Store is a fallback for launching GitEHR from elsewhere.

```text
gitehr config path              Print the config file path
gitehr config show              Show resolved config values
gitehr config set-store <path>  Set the default Store root
```

By default, the config file is:

```text
$XDG_CONFIG_HOME/gitehr/config.toml
```

If `XDG_CONFIG_HOME` is not set, GitEHR uses:

```text
~/.config/gitehr/config.toml
```

The file is TOML:

```toml
store_path = "/home/marcus/gitehr-store"
document_whitelist = ["pdf", "jpg", "png"]
```

`document_whitelist` is optional and restricts [`gitehr import --mode documents`](import.md) to the listed file extensions (case-insensitive, no leading dot). Leave it unset to accept any format.

Environment overrides:

| Variable | Purpose |
|---|---|
| `GITEHR_CONFIG` | Use a specific config file |
| `GITEHR_STORE_PATH` | Override the Store path for this process |

## Default Store

Set the default Store once:

```bash
gitehr config set-store ~/gitehr-store
```

After that, Store-level commands such as `gitehr store list` can run from outside the Store. Repo-level commands such as `gitehr journal add` also use the configured Store when it contains exactly one subject; with multiple subjects, `cd` into the subject repo you want.
