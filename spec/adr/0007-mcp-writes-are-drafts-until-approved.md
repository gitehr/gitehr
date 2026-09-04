# MCP tool calls write drafts only: human approval is required before anything enters the record

The MCP `add_journal_entry` tool writes a **draft** file that no git commit
references; a human must explicitly approve it (CLI, TUI, or GUI) before it is
staged and committed into the journal. The same approval gate applies to any
future MCP-driven clinical mutation. MCP's primary value is **reading** the
record; writing is deliberately mediated.

## Context

`gitehr mcp serve` gives MCP clients (Claude Desktop, other LLM agents)
direct access to a GitEHR repository. As of R30, `add_journal_entry` writes
real journal entries - there is no placeholder. That raised an authorship and
provenance question flagged in review on 2026-09-03:

- An MCP-authored clinical entry defaults its YAML `author` to the
  repository's active contributor (or takes a caller-passed ID), and can be
  absent. The git commit identity comes from the user's global git config.
  So an LLM-authored entry can be practically indistinguishable from one the
  clinician typed - the only trace being the separate `mcp_audit` entry
  (R33), which records the tool call but is not part of the entry itself.
- Whether an LLM may present itself as (or silently stand in for) the
  clinician is a clinical-safety and provenance question, not a code question.

The deployment spectrum makes a blanket rule wrong:

- **Single-user personal archive**: the patient and their MCP assistant are
  effectively one actor; forcing a preview/approval loop for every entry is
  ceremony without value.
- **Multi-user organisational deployment**: unprovenanced clinical entries are
  unacceptable; every machine-authored candidate must be seen and approved by
  a responsible human before it becomes part of the record.

## Decision

1. **MCP writes produce drafts, never commits.** `add_journal_entry` writes
   the entry file to disk but does not stage or commit it. The draft is
   clearly marked (see below) so the record's custody layer (git) contains
   nothing machine-authored until a human acts.
2. **A human approves each draft** through an interactive surface - `gitehr
   journal` review/approve in the CLI today; TUI and GUI later. Approval
   stages and commits the draft with the human as committer, converting it
   into an ordinary immutable journal entry. Rejection deletes the draft file;
   the record only ever grows.
3. **MCP is read-first.** Read resources (`journal`, `state`, `documents`,
   `imaging`, `status`) and `search_repository` are unimpeded - that is where
   MCP's value is. Future MCP write tools (`update_state`, state projections,
   summarisation) follow the same draft-then-approve pattern until an
   explicit decision says otherwise.
4. **A single-user mode may relax this later** by explicit opt-in (per-Store
   configuration, not per-repository committed state, so an untrusted repo
   cannot self-declare trust), but none exists today.
5. **Attribution stays honest at the draft stage**: MCP drafts record
   `mcp_draft: true` in front matter so the pending provenance is visible in
   any draft listing; approval clears the marker and stamps the human
   committer.

## Considered options

- **Status quo (MCP commits directly)**: rejected - indistinguishable
  authorship is unsafe in any multi-actor deployment and violates the
  provenance honesty the architecture promises.
- **Per-repo trust marker** ("this repo allows MCP commits"): rejected - a
  marker inside the repository can be supplied by the same untrusted party it
  is meant to gate (the same reasoning as R78's `--allow-bundled` flag).
- **Drafts + human approval (chosen)**: keeps the custody layer
  human-committed; makes the review step an ordinary workflow in the CLI/TUI/
  GUI; costs one extra command per accepted draft, which is the correct price
  for provenance.

## Consequences

- MCP clients that call `add_journal_entry` get a response that says the
  draft is pending human approval - the tool is honest that the record has
  not changed.
- `update_state` (verbatim state writes) receives the same treatment in a
  follow-up: drafts pending approval, no direct commit.
- The R60 provenance model gains a natural field: approved-by-human is part
  of an entry's provenance, and the approving identity is on record in the
  approval commit.
- Future LLM-driven updates to summaries (e.g. the problem list) will present
  diffs and ask for approval in the CLI/TUI/GUI rather than applying them -
  the pattern generalises.
- Draft files live under `journal/` but are untracked; the existing
  `journal list` ignores nothing, so draft listing/approval needs the
  dedicated review command (shipped with this ADR).