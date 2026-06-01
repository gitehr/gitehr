# gitehr transport

Bundle and unbundle a GitEHR repository as a single `tar.gz` archive for transport.

All subcommands require a GitEHR repository.

## gitehr transport create

```text
gitehr transport create [-o|--output <path>] [--encrypt]
```

Creates a `.tar.gz` archive containing `journal/`, `state/`, `imaging/`, `documents/`, and `.gitehr/`.

Options:

- `-o, --output <path>`: archive path (default: `<repo-name>.tar.gz` in the current directory)
- `--encrypt`: not yet implemented; prints a warning that transport-layer encryption is pending

## gitehr transport extract

```text
gitehr transport extract <archive> [-o|--output <dir>]
```

Extracts a transport archive to the target directory (default: current directory).
