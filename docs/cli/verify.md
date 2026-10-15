# gitehr verify

```text
gitehr verify
```

Walks the repository's Git history and checks that the journal is append-only ([ADR-0002](https://github.com/gitehr/gitehr/blob/main/spec/adr/0002-record-only-grows.md)): no commit may modify, delete, or rename a file under `journal/`. Each offending commit is reported on stderr and the command exits non-zero. Adding entries is always allowed.

This is an advisory client-side audit; it cannot stop another writer. It also compares the current branch with its upstream and fails if the two have diverged or if the upstream's remote-tracking ref was ever force-updated (a likely history rewrite); a branch with no upstream skips this check. It does not check object integrity (use `git fsck`) or authorship. See [`spec/repository-verification.md`](https://github.com/gitehr/gitehr/blob/main/spec/repository-verification.md) (roadmap item R40).
