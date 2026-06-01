# gitehr decrypt

```text
gitehr decrypt [--key <source>]
```

Removes the encryption marker from the repository.

!!! warning "Placeholder implementation"
    The current implementation deletes the `.gitehr/ENCRYPTED` marker file only. End-to-end repository decryption is on the roadmap but not yet implemented.

See [`gitehr encrypt`](encrypt.md) for the marker format.
