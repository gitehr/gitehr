<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Open Product Queries

This file records product decisions that block or constrain roadmap work. A resolved query should link its decision record or ADR and update the affected roadmap item and specification in the same change.

## Q1 - MCP state-change drafts

**Blocks:** [R30](roadmap.md#r30-add-mcp-tool-handlers)

ADR-0007 requires MCP-driven clinical mutations to remain proposed until a human approves them. `update_state` currently writes live `state/` files directly, so it does not meet that requirement.

Decide where a proposed state change lives and how it is reviewed:

- A dedicated, untracked draft area such as `.gitehr/mcp-drafts/`.
- Versioned draft files that every ordinary reader, search, export, and GUI path explicitly excludes.
- Another design that preserves the same boundary.

The decision must define the draft format, whether it represents a replacement or a patch, approval and rejection semantics, audit linkage, and how drafts remain invisible to ordinary clinical readers until approved. A future single-user relaxation must be explicit per-Store configuration, never repository-controlled, as required by ADR-0007.

## Q2 - Acquisition due-date basis

**Blocks:** [R87](roadmap.md#r87-align-the-acquisition-due-date-model)

The shipped command calculates `due_date` one calendar month after `date_sent`, because receipt is ordinarily unobservable and the register should chase unanswered requests. The original acquisition proposal describes the statutory clock as starting from receipt and identity verification, while the previous roadmap text incorrectly said it started from `ack_date`.

Decide whether the enduring model should retain the conservative `date_sent` default, represent receipt and identity-completion dates separately, or use another explicit rule. The decision must state the resulting field model, automatic calculation, override semantics, overdue filtering, and migration behaviour for existing acquisition records.
