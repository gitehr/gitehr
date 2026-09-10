<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Primary Intent and Improvement Review

Date: 2026-09-07. This is a source-review assessment and proposed prioritisation, not an adopted ADR or an implementation commitment. Findings describe the implementation inspected during the review; they were not reproduced in the running application. Source line numbers are review-time references and may move as the code changes. Track agreed implementation work in the [roadmap](roadmap.md).

## Primary Intent

GitEHR's strongest proposition is **a patient-controlled clinical record that remains readable, portable, and recoverable independently of any particular application or supplier**.

Git is the mechanism, not the main benefit. The distinction matters: "a decentralised EHR built on Git" invites comparison with complete clinical systems; "a lifetime record you can keep, inspect, recover, and bring to consultations" gives the project a more focused purpose.

The plain-file storage, separation of chronological evidence from current State, hashed Documents, and CLI-backed GUI are good foundations. Improve the guarantees around those foundations before expanding the feature catalogue.

## Highest Priorities

### 1. Make Portability Include Demonstrable Recovery

This is the most important gap against the project's purpose.

The transport archive excludes `.git` but describes itself as containing the "complete GitEHR repository data." That omits historical State, commit history, and Documents retained only in Git history. Meanwhile, Document verification can describe a missing file as "retained in Git history" without checking that it actually is recoverable.

References: `cli/src/commands/transport/create.rs:36`, `cli/src/commands/transport/create.rs:82`, `cli/src/commands/document/verify.rs:36`.

Distinguish three explicit operations:

- **Snapshot export:** current readable files.
- **Complete backup:** record contents, history, and everything needed for offline restoration.
- **Selective sharing:** deliberately chosen information for a recipient.

Make one test the defining preservation test: create a record, change its State, remove a Document from the working tree, back it up, restore into an empty directory offline, and recover both the historical State and original Document.

**"You can recover your record" should be a tested product guarantee, not an inference from using Git.**

### 2. Enforce the Boundary Between Proposed and Accepted Clinical Information

MCP journal writes are intentionally drafts, but the common journal reader includes them, and the GUI does not retain their draft status. Unapproved machine content can therefore appear as ordinary clinical narrative. MCP `update_state` also writes directly to live State.

References: `cli/src/commands/journal/mod.rs:244`, `gui/src-tauri/src/lib.rs:272`, `cli/src/commands/mcp/server_impl/tools.rs:182`.

Introduce a consistent distinction between:

- Source evidence.
- Proposed interpretation or change.
- Human-approved clinical content.
- Current State derived from accepted information.

That distinction must survive every reader, summary, search, and export. Keeping something uncommitted is insufficient when the application reads the working tree.

The existing [draft-approval design](adr/0007-mcp-writes-are-drafts-until-approved.md) is a good starting point; it needs enforcement across the whole application. This relates to R30 and R66.

### 3. Close Wrong-Patient and Partial-Write Paths

Two concrete areas deserve attention before broader clinical use:

- **Patient switching:** the journal composer is application-wide state. Opening another patient changes the destination without clearing or binding the unfinished note to its original patient. Async loads also lack a current-patient guard.
- **Write transactions:** ordinary journal commits can include unrelated staged files, and failed multi-file operations can leave partial changes behind.

References: `gui/src/App.tsx:168`, `gui/src/App.tsx:310`, `gui/src/App.tsx:341`, `cli/src/commands/git.rs:60`, `cli/src/commands/document/add.rs:90`.

Patient-scoped drafts, stale-response protection, path-limited commits, and rollback should be shared guarantees. The medication implementation already has stronger transaction tests, so this is an opportunity to extend an existing solution rather than invent another framework. Patient-context protection relates to R63 and R64.

### 4. Make Acquisition Loss-Aware and Provenance-Rich

A lifetime record usually starts as a collection of inconsistent exports, scanned letters, PDFs, and incomplete disclosures.

The current bulk Document importer flattens filenames and skips existing names without comparing content. Two different files called `Scan0001.pdf` can collide. It also bypasses the hashed Document references used by `document add`.

Reference: `cli/src/commands/import.rs:122`.

Prioritise a trustworthy acquisition workflow:

- Preserve original bytes and source identity.
- Distinguish identical duplicates from same-name conflicts.
- Report what was imported, skipped, unreadable, or missing.
- Track where records were requested and whether the response was complete.
- Link clinical assertions back to their supporting Documents.

The [acquisition/provenance specification](record-provenance-and-acquisition.md) already points in this direction. This would contribute more to the primary purpose than simply supporting more input formats. It relates to R4 and R60.

## Product Improvements

### 5. Complete One Useful Consultation Workflow

The GUI currently requests only ten journal entries without exposing older-entry navigation, shows only the first three allergies without an overflow indication, and does not yet surface all the typed clinical State available through the CLI.

References: `gui/src/App.tsx:317`, `gui/src/App.tsx:668`, `gui/src/App.tsx:746`.

Aim for one complete workflow:

> Open a patient, understand their current situation, find the supporting evidence, add a reviewed update, and share an appropriate summary.

That needs complete-history navigation, visible medications and allergies, source links, review dates, and explicit uncertainty. "Loading," "unavailable," "not recorded," and "confirmed none" must not look interchangeable.

This builds on R61 and R63. A shareable consultation summary is a proposal, not a claim that this capability already ships.

### 6. Make the Public Promise Narrower and More Accurate

The README advertises encryption-at-rest support, while `encrypt` correctly refuses because encryption is unimplemented. A legacy marker can still make `status` report "Encrypted." The safety documentation also overstates signing guarantees.

References: `README.md:28`, `cli/src/commands/encrypt.rs:24`, `cli/src/commands/status.rs:127`, `docs/safety/safety.md:38`.

Publish a short capability matrix separating **implemented**, **environment-dependent**, and **planned** controls, plus a root safety entry point explaining intended use and limitations.

For this project, accurate limits strengthen trust more than ambitious security language. This relates to R43, R44, R67, R68, and R69; correcting current claims need not wait for encryption or signing implementation.

## Suggested Direction

The recommended near-term milestone is:

> A person can assemble a synthetic lifetime record, inspect all of it, distinguish sources from approved summaries, safely add information, and restore it completely on another machine.

Suggested sequence:

1. Fix patient-context, draft-isolation, transaction, and misleading security-status issues.
2. Prove complete backup and restoration.
3. Unify imports with verified Document storage.
4. Finish the longitudinal viewer and consultation summary.
5. Expand interoperability and AI assistance against those guarantees.

The central opportunity is not more functionality. It is making **custody, provenance, completeness, and recovery** reliably true from ingestion through everyday use to eventual restoration.
