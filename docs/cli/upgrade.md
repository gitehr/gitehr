# gitehr upgrade

Upgrade an existing repository to the current CLI version.

All subcommands require a GitEHR repository.

## gitehr upgrade

```text
gitehr upgrade
```

Updates `.gitehr/GITEHR_VERSION`, re-bundles the CLI binary at `.gitehr/gitehr`, and records an upgrade entry in the journal documenting the version change.

If `.gitehr/GITEHR_VERSION` already matches the CLI version, it prints "Repository is already at the latest version." and changes nothing (no binary copy, no journal entry). A missing version file is reported as `unknown` and treated as needing an upgrade.

Both subcommands refuse to proceed if `.gitehr/GITEHR_VERSION` or `.gitehr/gitehr` is a symlink, since a received repository could use one to make the command read or write outside the repository.

## gitehr upgrade-binary

```text
gitehr upgrade-binary
```

Updates the bundled binary and `.gitehr/GITEHR_VERSION` only - does not write a journal entry. Useful when refreshing the bundled binary as an internal/maintenance step without polluting the clinical record.
