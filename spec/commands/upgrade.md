<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr upgrade`

### `gitehr upgrade`

Upgrades an existing GitEHR repository to the latest version.

## Behaviour

1. Requires the current directory to be a GitEHR repository (presence of `.gitehr`).
2. Compares the repository's current version (from `.gitehr/GITEHR_VERSION`) with the CLI version.
3. If versions match, reports "Repository is already at the latest version" and exits.
4. Otherwise, performs the upgrade:
   - Updates `.gitehr/GITEHR_VERSION` to the new version.
   - Updates the bundled binary in `.gitehr/gitehr`.
   - Records the upgrade in the journal for auditability.
5. Prints confirmation of completion.

## Example

```bash
$ gitehr upgrade
GitEHR Repository Upgrade
=========================

Current version: 0.1.5
New version: 0.1.6

Performing upgrade...
  Updated version file.
  Updated bundled binary.
  Recorded upgrade in journal.

Upgrade complete!
```

## Related Commands

- [`gitehr upgrade-binary`](upgrade-binary.md) - Updates only the bundled binary without recording in journal.
- [`gitehr init`](init.md) - Creates a new repository with the current version.