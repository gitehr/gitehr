# gitehr transport

Bundle and unbundle a GitEHR repository as a single `tar.gz` archive for transport.

All subcommands require a GitEHR repository.

## gitehr transport create

```text
gitehr transport create [-o|--output <path>] [--encrypt]
```

Creates a `.tar.gz` archive containing `journal/`, `state/`, `imaging/`, `documents/`, `openehr/`, `fhir/`, and `.gitehr/`. Directories that do not exist in the repository are skipped.

What the archive does and does not contain:

- It holds regular files only. Symlinks are skipped rather than followed, so a link planted in a received repository cannot pull in a file from outside it.
- It does **not** include the Git history (`.git/`), so it is a snapshot of the working files, not a complete backup. Commit history and removed files are not recoverable from it.
- Every file is stored with mode `0644`; executable bits are not preserved.

Options:

- `-o, --output <path>`: archive path (default: `gitehr-transport-<YYYYMMDD-HHMMSS>.tar.gz` in the current directory, timestamped in UTC)
- `--encrypt`: not yet implemented. The command exits with an error and writes no archive, rather than silently producing an unencrypted one. Re-run without `--encrypt` to create a plaintext archive.

## gitehr transport extract

```text
gitehr transport extract <archive> [-o|--output <dir>]
```

Extracts a transport archive to the target directory (default: current directory).

Does not require a GitEHR repository. The output directory is created if it does not exist, existing files with the same names are overwritten, and entry paths that would escape the output directory are not extracted. It prints a note when the extracted files contain a `.gitehr/` directory. Extraction only unpacks files and does not validate the record; run `gitehr verify` afterwards.
