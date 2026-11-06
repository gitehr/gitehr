# Safety controls in practice

The [safety overview](safety.md) explains the architectural argument. This page lists the controls that ship **today**, what each one does, and what it does not do. Planned work is called out explicitly so nothing here is read as a stronger claim than the software supports.

## 1. Check the journal is append-only: `gitehr verify`

```text
gitehr verify
```

Walks the Git history and reports any commit that modifies, deletes or renames a file under `journal/`, any backdated entry (filename timestamp earlier than one already added), and any divergence from the upstream branch, including a force-updated remote-tracking ref (a likely history rewrite). It exits non-zero on findings, so it can be run from a scheduled job or CI.

What it does **not** do:

- It is an advisory, client-side audit. It cannot stop another writer; a server-side guardian is planned (roadmap R40).
- It does not check object integrity. Run `git fsck` for that.
- It does not check authorship. Signing is planned work (R41, R69).
- Merge commits are checked against their first parent, so entries merged from a concurrent writer can be flagged. Review those findings rather than assuming tampering.

See [`gitehr verify`](../cli/verify.md).

## 2. AI-assisted writes are drafts until a human approves

Entries written through the MCP server (for example by an LLM client) are **uncommitted drafts**. They are not staged, not committed, and not part of the record until a clinician approves them ([ADR-0007](https://github.com/gitehr/gitehr/blob/main/spec/adr/0007-mcp-writes-are-drafts-until-approved.md)).

```text
gitehr journal drafts                      # list pending drafts
gitehr journal drafts --approve <filename> # commit it; you are the committer
gitehr journal drafts --reject <filename>  # delete the uncommitted draft
```

Approval commits the entry together with its pending MCP audit entries. Rejection leaves the record unchanged.

See [`gitehr journal`](../cli/journal.md) and [MCP usage](../cli/mcp-usage.md).

## 3. MCP mutations leave their own audit entry

Every successful MCP mutation tool call is also recorded as a separate audit journal entry (author `mcp-server`, front matter `mcp_audit`). Read-only tools do not create audit entries. Client and session identity (client name, token, IP) is not yet recorded.

## 4. Corrections are new entries

The record only grows ([ADR-0002](https://github.com/gitehr/gitehr/blob/main/spec/adr/0002-record-only-grows.md)). A correction is a new journal entry that references the entry it corrects, never an edit in place. This is what `gitehr verify` checks for.

## A suggested routine

| When | Action |
| --- | --- |
| After every sync or pull | `gitehr verify` |
| Before approving an MCP draft | Read it in full; check it against the source |
| Periodically, per Store | `git fsck` for object integrity |

## What is still planned

Cryptographic signing and hardware-backed credentials, a server-side append-only guardian, encryption at rest, and external anchoring of the genesis record are tracked in the [roadmap](https://github.com/gitehr/gitehr/blob/main/spec/roadmap.md) (R40, R41, R67-R70). Hazard analysis and the formal safety case are developed under [Turva](https://github.com/pacharanero/turva).
