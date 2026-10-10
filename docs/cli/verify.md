# gitehr verify

```text
gitehr verify
```

Walks the repository's Git history and checks that the journal is append-only ([ADR-0002](../../spec/adr/0002-record-only-grows.md)): no commit may modify, delete, or rename a file under `journal/`. Each offending commit is reported on stderr and the command exits non-zero. Adding entries is always allowed.

This is an advisory client-side audit; it cannot stop another writer. It does not check object integrity (use `git fsck`), history rewrites, or authorship. See [`spec/repository-verification.md`](../../spec/repository-verification.md) (roadmap item R40).
