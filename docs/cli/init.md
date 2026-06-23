# gitehr init

```text
gitehr init
```

Initializes a GitEHR repository in the current directory.

Behavior:

- Creates `.gitehr/` and writes `.gitehr/GITEHR_VERSION`.
- Copies the repository template from `folder-structure/` (`journal/`, `state/`, `imaging/`, `documents/`, and the per-directory README files).
- Bundles the CLI binary at `.gitehr/gitehr` so the repository is self-contained and version-pinned.
- Creates a genesis journal entry seeded with a random hash to anchor the tamper-evident chain.
- Runs `git init` and creates the initial commit.

Example:

```bash
mkdir patient-record
cd patient-record
gitehr init
```

After init the new repo is ready for journal entries:

```bash
gitehr journal add "First entry: initial registration."
```

!!! note "Multi-patient stores"
    For deployments managing many patient repos under a single store root with a Main Patient Index, see [`gitehr store`](../design/repository-structure.md). Single-patient `gitehr init` is the right path for development, demos, and most clinical use today.
