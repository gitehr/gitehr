<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr clincalc`

Clinical calculators: scores, screeners, and risk tools. `gitehr clincalc` is a `$PATH` plugin (see [`cli/src/commands/plugin.rs`](../../cli/src/commands/plugin.rs)): GitEHR discovers the external `gitehr-clincalc` executable and forwards `list`, `<name>`, `--schema`, `--license`, and computing via `--input` straight through, unchanged. See [`docs/cli/clincalc.md`](../../docs/cli/clincalc.md) for that surface.

`gitehr clincalc record` is the one exception: a built-in GitEHR command (R25) that runs the plugin itself, rather than forwarding to it, so the result can be written to the journal.

### `gitehr clincalc record <name> --input <value>`

Runs `<name>` through `gitehr-clincalc --input <resolved> --format json` (captured, not exec'd, unlike the plugin fallthrough) and records the result as a new, immediately committed journal entry. Requires the same repository context as `gitehr journal add` (the nearest `.gitehr/` ancestor, or a Store's single auto-targeted subject).

`--input` follows the same three-way contract as the plugin's own `--input`:

| Value | Meaning |
|---|---|
| `-` | Read JSON from stdin |
| A value starting with `{` or `[` | Used inline as JSON |
| Anything else | Read as a file path (resolved against the caller's working directory, before GitEHR changes into the repository root) |

## Data model

The entry's `clincalc` front-matter block, added to the shared `JournalEntry` (see [`cli/src/commands/journal/mod.rs`](../../cli/src/commands/journal/mod.rs)) alongside `documents`:

| Field | Meaning |
|---|---|
| `calculator` | The calculator name passed to `record` |
| `version` | The `gitehr-clincalc --version` output, best-effort - omitted (never faked) when it cannot be determined |
| `inputs` | The resolved input JSON, echoed verbatim |
| `result` | The calculator's `result` field |
| `interpretation` | The calculator's `interpretation` field, when present |
| `reference` | The calculator's `reference` (citation) field, when present |

The entry body is a short Markdown narrative naming the calculator and its interpretation; the structured, auditable record lives in the front matter.

Like every journal entry, this is append-only and immutable once committed: a `gitehr clincalc record` invocation always creates a new entry rather than mutating a previous one.

## Errors

`record` fails with a clear, non-zero-exit error - and writes nothing - when: `gitehr-clincalc` is not found on `$PATH`; the resolved input is not valid JSON; the plugin exits non-zero or does not print valid JSON; or the command is run outside a GitEHR repository or Store.
