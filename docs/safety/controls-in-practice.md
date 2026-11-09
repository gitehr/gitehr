<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Safety controls in practice

The [safety overview](safety.md) explains the architectural argument. This page lists the controls that ship today, what each one does, and what it does not do. Planned work and known limitations are explicit so these controls are not read as stronger assurances than the software supports.

## Check the journal is append-only: `gitehr verify`

```text
gitehr verify
```

This walks Git history and reports commits that modify, delete, or rename a `journal/` file; entries whose filename timestamps are out of order; and divergence from the upstream branch, including force-updated remote-tracking refs. It exits non-zero on findings, so it is suitable for scheduled jobs or CI.

What it does not do:

- It is an advisory client-side audit. It cannot stop another writer; a server-side guardian is planned (R40).
- It does not check Git object integrity. Run `git fsck` for that.
- It does not check authorship. Signing is planned work (R41, R69).
- Merge commits are checked against their first parent, so entries merged from a concurrent writer can be reported. Review those findings rather than assuming tampering.
- It does not establish that a later entry is an adequate clinical correction of an earlier one.

See [`gitehr verify`](../cli/verify.md).

## Journal drafts require human approval

The MCP `add_journal_entry` tool writes an uncommitted, unstaged journal draft. It is not part of the record until a human approves it.

```text
gitehr journal drafts                      # list pending drafts
gitehr journal drafts --approve <filename> # commit it; you are the committer
gitehr journal drafts --reject <filename>  # delete the uncommitted draft
```

Approval commits the entry together with its pending MCP audit drafts. Rejection leaves the committed record unchanged. See [`gitehr journal`](../cli/journal.md) and [MCP usage](../cli/mcp-usage.md).

### Known exception: `update_state`

`update_state` does not yet follow the approval model. It writes a State file directly, making the change visible before human review. This is a known ADR-0007 violation tracked by R30. Keep the tool disabled in `.gitehr/mcp.json` until approval-gated State proposals replace it.

```json
{
  "tools": {
    "update_state": { "enabled": false }
  }
}
```

## MCP audit drafts

After a successful MCP mutation, the server attempts to create a separate uncommitted audit draft with author `mcp-server` and `mcp_audit` front matter. Read-only tools do not create audit drafts. Audit creation is best-effort: a failure is reported on standard error but does not turn the completed tool call into a failure. Client and session identity are not yet recorded.

## Correcting a record

The record only grows ([ADR-0002](https://github.com/gitehr/gitehr/blob/main/spec/adr/0002-record-only-grows.md)). Correct a clinical assertion by adding a new journal entry that identifies the earlier entry and explains the correction; never edit the earlier entry in place. GitEHR has no dedicated correction link, and `gitehr verify` checks immutability rather than the clinical adequacy of the correction.

## Suggested routine

| When | Action |
| --- | --- |
| After every sync or pull | `gitehr verify` |
| Before approving an MCP draft | Read it in full and check it against the source |
| Per Store, before enabling MCP | Disable `update_state` in `.gitehr/mcp.json` |
| Periodically, per Store | `git fsck` for object integrity |

## Still planned

Cryptographic signing and hardware-backed credentials, a server-side append-only guardian, approval-gated MCP State changes, encryption at rest, and external anchoring of the genesis record are tracked in the [roadmap](https://github.com/gitehr/gitehr/blob/main/spec/roadmap.md) (R30, R40, R41, R67-R70). Hazard analysis and the formal safety case are developed under [Turva](https://github.com/pacharanero/turva).
