# gitehr status

```text
gitehr status
```

Shows a summary of the GitEHR repository.

Alias: `st`.

Output includes:

- Repository version from `.gitehr/GITEHR_VERSION`
- Encryption state (presence of `.gitehr/ENCRYPTED`)
- Journal entry count
- State file list (excluding `README.md`)
- Git working directory status, when the directory is also a Git repository

If the current directory is not a GitEHR repository, prints a hint to run `gitehr init`.
