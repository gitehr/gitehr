# gitehr upgrade

Upgrade an existing repository to the current CLI version.

All subcommands require a GitEHR repository.

## gitehr upgrade

```text
gitehr upgrade
```

Updates `.gitehr/GITEHR_VERSION`, re-bundles the CLI binary at `.gitehr/gitehr`, and records an upgrade entry in the journal documenting the version change.

## gitehr upgrade-binary

```text
gitehr upgrade-binary
```

Updates the bundled binary and `.gitehr/GITEHR_VERSION` only - does not write a journal entry. Useful when refreshing the bundled binary as an internal/maintenance step without polluting the clinical record.
