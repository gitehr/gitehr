# gitehr encrypt

```text
gitehr encrypt [--key <source>]
```

Marks the repository as encrypted.

!!! warning "Placeholder implementation"
    The current implementation writes only a `.gitehr/ENCRYPTED` marker. End-to-end repository encryption is on the roadmap but not yet implemented.

Behavior:

- Writes `.gitehr/ENCRYPTED` containing `encrypted_at` (current timestamp) and `key_source` (the value of `--key`, or `local` if omitted).
- Prints a notice that full encryption is pending.

To remove the marker, see [`gitehr decrypt`](decrypt.md).
