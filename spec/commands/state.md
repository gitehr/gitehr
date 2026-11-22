<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr state`

Manages the mutable clinical state files within the GitEHR repository.

All subcommands resolve the subject repository from the current directory (nearest `.gitehr/` ancestor, or the sole subject at a single-subject Store root).

`gitehr state` is the raw file-level interface. GUI-facing and automation-facing
clinical state should prefer typed commands such as
[`gitehr demographics`](demographics.md), [`gitehr allergies`](allergies.md),
[`gitehr medications`](medications.md), [`gitehr conditions`](conditions.md), and
[`gitehr observations`](observations.md), which validate their fields, emit
stable JSON, and create companion journal entries for mutations.

### `gitehr state list`

Lists state files recursively under the `state/` directory (excluding its `README.md`).

Behavior:
- Prints last modified timestamps when available.
- If no files are present, prints a hint to use `gitehr state set`.
- Non-UTF-8 files are read lossily so they are still listed (this listing is shared with `gitehr status` and the GUI).
- A symbolic link anywhere under `state/` is refused.

### `gitehr state get <filename>`

Prints the contents of a state file.

Behavior:
- Fails if the file does not exist in `state/`, is not valid UTF-8, or is reached through a symbolic link.

### `gitehr state set <filename> <content>`

`gitehr state set <filename> --file <path|->`

`gitehr state set <filename> --file <path|-> --commit`

Writes content to the specified state file, creating the `state/` directory (and any subdirectory in `filename`) if needed.

Behavior:
- Overwrites any existing file of the same name.
- `filename` may include subdirectories (for example `calculations/feverpain-latest.json`); an absolute path or any `..` component is rejected.
- `--file -` reads content from standard input, avoiding sensitive content in process arguments. A relative `--file` path is resolved against the invocation directory, before context resolution changes into the repository.
- `--commit` creates a commit containing only the updated state file and refuses to overwrite that state file when it has uncommitted changes. When the written content equals the committed content it prints that there is nothing to commit and exits successfully. A failed commit unstages the file but does not restore its previous content.

### `gitehr state`

If run without a subcommand, defaults to `list`.
