<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr upgrade-binary`

### `gitehr upgrade-binary`

Updates the bundled binary in `.gitehr/gitehr` to match the currently running CLI version.

## Behaviour

1. Requires the current directory to be a GitEHR repository (presence of `.gitehr`).
2. Copies the current `gitehr` executable to `.gitehr/gitehr`, overwriting any existing binary.
3. Sets executable permissions on Unix systems (mode 0755).
4. Updates `.gitehr/GITEHR_VERSION` to reflect the new version.
5. Prints confirmation of the update.

## When to Use

- After installing a new version of GitEHR system-wide, run this command in each repository to update the bundled binary.
- When sharing a repository with others who may not have GitEHR installed.
- To ensure the repository's bundled binary matches your current CLI version.

## Example

```bash
$ gitehr upgrade-binary
GitEHR Binary Upgrade
=====================

CLI version: 0.1.6
Bundled binary: exists

Updating bundled binary...
  Copied current executable to .gitehr/gitehr
  Updated version file to 0.1.6

Binary upgrade complete!
```

## Related Commands

- [`gitehr upgrade`](upgrade.md) - Upgrades the repository structure/schema and also updates the bundled binary.
- [`gitehr init`](init.md) - Creates a new repository with the binary bundled.
