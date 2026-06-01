# CLI overview

!!! note "Developer tool"
    The CLI is the interoperability layer and the canonical interface for automation, testing, and integration. Clinicians and patients should use the [GUI](../gui/gui.md).

## Basics

- Running `gitehr` with no arguments prints the version followed by a list of subcommands.
- Most commands require a GitEHR repository in the current directory (presence of `.gitehr/`). Exceptions are listed on each command page.
- Add `--help` to any subcommand to see its exact syntax (`gitehr journal --help`, `gitehr journal add --help`).

## Command pages

Each command has its own page. Subcommands are sections within the parent command's page.

| Command | Purpose |
|---|---|
| [`gitehr init`](init.md) | Initialise a new GitEHR repository |
| [`gitehr journal`](journal.md) | Append-only clinical journal (`add`, `show`, `cat`, `verify`) |
| [`gitehr state`](state.md) | Mutable current state files (`list`, `get`, `set`) |
| [`gitehr user`](user.md) | Manage contributors and the active author |
| [`gitehr remote`](remote.md) | Named remote repositories for sync |
| [`gitehr status`](status.md) | Summarise the repository |
| [`gitehr encrypt`](encrypt.md) / [`gitehr decrypt`](decrypt.md) | Encryption markers (placeholder implementation) |
| [`gitehr transport`](transport.md) | Bundle and unbundle the repository as a single archive |
| [`gitehr gui`](gui.md) | Launch the bundled or system GUI |
| [`gitehr upgrade`](upgrade.md) | Upgrade the repository and bundled binary |
| [`gitehr version`](version.md) | Print the CLI and Git versions |
| [`gitehr completions`](completions.md) | Generate shell completion scripts |

## Developer-facing pages

- [Developer workflow](developer-workflow.md) - typical local dev loop, version bump policy, manual testing recipes.
- [MCP usage](mcp-usage.md) - integrating the MCP server with LLM clients.

## End-to-end demo

The shortest useful demonstration of the CLI:

```bash
mkdir patient-record && cd patient-record
gitehr init

gitehr user add me "Dr Marcus Baw" --role gp
gitehr user activate me

gitehr journal add "Initial consultation. Patient reports 3 weeks of breathlessness on exertion."
gitehr journal add "Echo arranged. Started furosemide 40mg OD."
gitehr journal add "Follow-up: symptoms improved. EF 35%. Started ramipril and bisoprolol."

gitehr journal cat        # play back the record
gitehr journal verify     # confirm the chain is intact
```
