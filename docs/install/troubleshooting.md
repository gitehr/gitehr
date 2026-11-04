# Troubleshooting

Common messages from the `gitehr` CLI and what to do about them. Every message below is quoted from the CLI source; if you see something not listed here, please [open an issue](https://github.com/gitehr/gitehr/issues).

## Repository and Store errors

### No repository or Store found

```text
Not a GitEHR repository or Store. Run `gitehr store init` to create one, or set one with `gitehr config set-store <path>`.
```

Repo-level commands such as `journal`, `status`, `encrypt` and `decrypt` look for a `.gitehr/` directory in the current directory or an ancestor, so they work from subdirectories too. They also auto-target the only subject when run from a Store with one subject, or use a configured default Store. With a Store containing multiple subjects, change into the subject's repository before running a repo-level command.

To create a Store and its first subject repository, run `gitehr store init [name]` in an empty directory. `gitehr status` prints a setup hint instead of failing when no repository or Store can be found.

### `This directory is already a GitEHR Store (...)`

`gitehr store init` found an existing Store in the current directory. Nothing was changed.

### `This directory is a GitEHR repository, not a Store root.`

`gitehr store init` was run inside a single-subject repository. Create a Store in a separate, empty directory.

### `Not inside a GitEHR Store (no gitehr-mpi.json found).`

A `gitehr store` subcommand could not find a Store in the current directory or an ancestor, and no valid default Store is configured. Run it from inside a Store, create one with `gitehr store init`, or configure one with `gitehr config set-store <path>`.

### `Not a GitEHR Store root (... not found)`

`GITEHR_MPI_PATH` names an MPI file that does not exist. Unset it to use the Store's default `gitehr-mpi.json`, or set it to an existing MPI file.

### `GitEHR repository already exists in this directory`

The directory already contains a repository. Use a different, empty directory for a new record.

## Journal errors

### `Aborting: the journal entry is empty.`

No text was supplied. Pass the entry text as an argument, use `--file`, pipe it on stdin, or write text in your editor before saving.

### `Editor exited with a non-zero status; the entry was not recorded.`

When `gitehr journal add` is run on a terminal without text, it opens `$EDITOR` (falling back to `$VISUAL`). If the editor exits with an error, no entry is written. Check that `EDITOR` names a working command, or supply the text directly.

### `Journal entry not found` / `Draft not found`

The filename does not exist under `journal/`. List entries with `gitehr journal list` and copy the filename exactly.

## Git errors

### `Git binary not found. Please install git to use this feature.`

GitEHR records history with Git and needs `git` on your `PATH`. Install Git and open a new terminal.

### `Git command failed: ...`

The text after the colon is Git's own message. Run the same Git command by hand in the repository to investigate; common causes are a missing `user.name`/`user.email`, a locked repository, or a lack of write permission.

## `gitehr verify` findings

`gitehr verify` exits non-zero and reports `N append-only violation(s), N history-rewrite finding(s) and N ordering violation(s)` when the history is not a clean, time-ordered, append-only journal.

- **Append-only violation:** a commit modified, deleted or renamed a file under `journal/`. Review the named commit.
- **Ordering violation:** an entry was added with a filename timestamp earlier than one already present (a backdated entry).
- **History-rewrite finding:** the branch has diverged from its upstream, or the upstream's remote-tracking ref was force-updated.
- Merge commits are checked against their first parent, so entries merged in from a concurrent writer can be reported. Review such findings rather than assuming tampering.

`verify` is an advisory, client-side audit. It does not check object integrity; run `git fsck` for that. See [`gitehr verify`](../cli/verify.md).

## Encryption

### `Encryption at rest is not yet implemented`

`gitehr encrypt` deliberately refuses and changes no files. See [`gitehr encrypt`](../cli/encrypt.md).

### `Repository is not encrypted.`

`gitehr decrypt` was run on a repository with no encryption marker. Nothing to do.

## Still stuck?

Run `gitehr version` and `gitehr status`, then include their redacted output and the exact error text in an [issue](https://github.com/gitehr/gitehr/issues). Do not paste patient-identifiable data or file paths that could identify a person.
