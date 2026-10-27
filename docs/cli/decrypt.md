# gitehr decrypt

```text
gitehr decrypt [--key <source>]
```

Removes a stale `.gitehr/ENCRYPTED` marker left by an earlier placeholder version of `gitehr encrypt`.

!!! warning "Not implemented"
    Encryption at rest was never implemented (roadmap R67/R68), so no data was encrypted and nothing is decrypted. `--key` is accepted but ignored.

Behavior:

- Fails if the current directory is not a GitEHR repository.
- Fails with "Repository is not encrypted." if no marker exists.
- Otherwise deletes `.gitehr/ENCRYPTED` and prints a note that nothing was actually encrypted.

See [`gitehr encrypt`](encrypt.md).
