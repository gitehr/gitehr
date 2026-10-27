# gitehr encrypt

```text
gitehr encrypt [--key <source>]
```

Encryption at rest is **not yet implemented** (roadmap R67/R68). The command always refuses and changes nothing.

!!! warning "Not implemented"
    `gitehr encrypt` no longer writes a `.gitehr/ENCRYPTED` marker, because a marker with plaintext files would falsely suggest the record is protected.

Behavior:

- Fails if the current directory is not a GitEHR repository.
- Fails with "Repository is already encrypted." if a legacy `.gitehr/ENCRYPTED` marker exists.
- Otherwise fails with a "not yet implemented" message; no files are changed. `--key` is accepted but ignored.

To clear a stale marker left by an earlier version, see [`gitehr decrypt`](decrypt.md).
