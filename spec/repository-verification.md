<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Repository verification and the server-side guardian

*Status: notes for future consideration. Captures the design thinking about what `gitehr verify` should mean now that the per-entry front-matter chain has been removed (see [`commands/journal.md`](commands/journal.md) "Planned refinements"). No implementation is proposed yet. Relates to [ADR-0002](adr/0002-record-only-grows.md) (the record only grows), the hardware-backed signing and gittuf items in [`roadmap.md`](roadmap.md), and [`commands/document.md`](commands/document.md) (`document verify`, which stays).*

## The change that prompted this

GitEHR used to chain each journal entry to its parent through front matter and a dedicated journal command. That per-entry chain has been removed: each committed entry is its own Git commit, so ordering and tamper-evidence now derive from Git's own history rather than from a hand-rolled chain. This note records what, if anything, should take its place.

## Two different guarantees the old chain blurred together

### Object integrity - Git already provides this

Git content-addresses every commit and blob. Altering any committed byte changes that object's hash, and `git fsck` walks the object graph to detect corruption or a doctored object. GitEHR does not need, and should not reimplement, a command for this. The removed per-entry chain was largely duplicating what Git already does, which is why removing it was correct.

### GitEHR policy invariants - Git does NOT enforce these

This is the real gap and the only place a future `verify` earns its keep. Git will happily let a writer:

- modify or delete a past journal entry in a new commit - nothing enforces ADR-0002's "the record only grows";
- rewrite history or force-push - rebase an entry away, or backdate a fabricated genesis;
- commit as an unauthorised or unsigned author.

## What `verify` should be: a policy checker over Git history

Not a content hasher. A future `gitehr verify` walks the Git history and asserts GitEHR's invariants:

- **Append-only:** every commit that touches `journal/` only adds entries - none are modified or deleted (ADR-0002).
- **Monotonic ordering:** journal filenames are time-ordered; history only ever appends.
- **No history rewrite:** the history is a fast-forward of the last known-good ref / the remote (force-push and rebase detection).
- **Authorised authorship (once signing lands):** every journal commit is signed by an enabled contributor in `.gitehr/contributors.json`.

## Where it should run

- **Server-side `pre-receive` guardian (primary).** The strongest deployment is a receive gate on the shared server that rejects any push failing the invariants above. This is the "guardian of the GitEHR server" framing: tampered or non-append-only pushes never land. This is where the feature has real teeth, because it governs what the canonical remote will accept.
- **Client-side advisory audit (secondary).** The same logic run locally as `gitehr verify` for a contributor to check a clone before trusting or pushing it. Advisory only - a local check cannot bind other writers.

## Do we need it right now?

No, not urgently, and this note deliberately does not propose building it yet.

- Object integrity is already covered by `git fsck`.
- There is no multi-writer server yet for a guardian to protect, so the highest-value deployment point does not exist yet.
- The one honest gap today is that **append-only is enforced only by convention**: nothing currently stops a `git commit` that rewrites a past entry, even though ADR-0002 actively claims the record only grows.

If a small near-term piece were wanted, it would be a client-side check of just that one invariant - no journal entry ever modified or deleted across the whole history - which could ship now and be reused verbatim by the server guardian later. But it is a "nice to have soon", not a missing safety control blocking anything today.

## Connections to existing plans

- **Signing** ([`roadmap.md`](roadmap.md) hardware-backed contributor credentials): the "authorised authorship" invariant depends on commit/entry signing existing first.
- **gittuf** ([`roadmap.md`](roadmap.md) security review item): gittuf applies TUF concepts to a Git repo - policy-controlled, signed access to refs, rollback/rewrite protection. That is substantially the server-side guardian described here, already built and maintained upstream. Evaluating gittuf may be a better path than hand-rolling the guardian.
- **Genesis without a false-genesis claim** ([`commands/journal.md`](commands/journal.md) "Planned refinements"): the "no history rewrite / no backdated genesis" invariant pairs with the idea of anchoring the genesis entry to an external, timestamped registration.
- **`document verify`** ([`commands/document.md`](commands/document.md)): unaffected and retained - Documents keep a real SHA-256 verifiability contract (ADR-0003). Only the journal's per-entry chain was removed.

## Open questions

- Is the guardian best built natively, or by adopting gittuf?
- What is the "last known-good ref" a client audit compares against, before there is a canonical server?
- How does the append-only check treat legitimate working-tree deletions of Documents (allowed - the bytes remain in history per ADR-0002) versus deletions of journal entries (disallowed)?
- Does `verify` belong as a top-level `gitehr verify` (repository-wide policy) rather than under `journal`, given it spans journal, documents, and refs?
