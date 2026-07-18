---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Common objections

Answers to the questions GitEHR gets asked most often, honestly - including the ones without a fully comfortable answer.

## "How do you run population health queries across patients if every record is its own repo?"

You do not query the canonical files directly for this. An organisation that needs cross-patient answers - a hospital, a research network, a regulator - builds a derived database from its own clone of the patients it cares for, in whatever engine and schema suit the workload, and rebuilds it whenever it likes. This is the same shape as a data lakehouse: canonical files at the bottom, a derived query layer on top. See [Files, not databases](files-not-databases.md#what-about-cross-patient-queries) for the full argument.

## "What happens when two organisations edit the same patient's record at the same time?"

Git already has an answer to this, because it is the same problem distributed software teams solve constantly: two people changing the same file. Each organisation's clone advances its own history; merging two clones is a normal `git merge`.

Two properties keep this workable for clinical data:

- **The journal only grows.** Journal entries are individually append-only, timestamped, and attributed - a new consultation, result, or note is always a new file, never an edit to an existing one. Two organisations adding entries at the same time simply produces two new files with no possibility of a merge conflict, because there is no shared line of text to fight over.
- **State files can genuinely conflict, and that is surfaced, not hidden.** A structured record such as the allergy or demographics state file can be legitimately changed by two organisations in the same window - for example, two clinics both recording a new allergy at the same visit. Git's merge machinery detects this as a conflict rather than silently picking a winner, exactly as it does for source code. Resolution is a clinical decision, not a technical one: a clinician (or, later, a defined reconciliation workflow) looks at both versions and decides what the merged state should say, and that resolution is itself committed and attributed. Nothing is lost in the process - both proposed versions remain visible in the git history even after the conflict is resolved.

This is a genuine difference from the traditional model, where the database simply serialises writes to a row and the last write silently wins. GitEHR trades that silent behaviour for a visible one: a conflict that must be looked at, with full provenance of who proposed what.

## "Where's your ACID? How is this consistent without a database engine?"

There is no single-engine, cross-record transaction manager, because there is no single engine - each patient's record is its own repository. Consistency is provided at two different levels instead of one:

- **Per-file atomicity.** A git commit is atomic: a journal entry or a state-file update either lands completely, with a valid commit object, or it does not exist at all. There is no equivalent of a half-written row.
- **Cryptographic chain of custody**, not row-level locking. Each commit is signed and each commit's parent is fixed, so the *history itself* - not a lock manager - is what proves nothing has been silently reordered, backdated, or dropped after the fact. See [Provenance](provenance.md).

What GitEHR does not give you is cross-patient, multi-row ACID transactions ("update these ten patients' records atomically together"), because that was never a requirement of a per-patient custody layer. See the [lakehouse framing](files-not-databases.md#the-lakehouse-is-the-industry-quietly-conceding-the-argument) in *Files, not databases* for why the query-layer/custody-layer split is deliberate, and why Stonebraker and Pavlo's "the relational model always wins" argument is answered there rather than here.

## "What about GDPR's right to erasure? You can't delete anything from an immutable history."

This is the hardest objection, and the honest answer has two parts, not one comfortable one.

!!! warning "There is no free lunch here"
    Any system that provides strong tamper-evidence through history - GitEHR, a blockchain, a WORM archive, an append-only audit log - has exactly this same tension with a literal reading of "the right to erasure." A design that can be trivially rewritten to delete an entry is a design that can be trivially rewritten to delete evidence of tampering. GitEHR does not get to have both properties for free, and neither does anyone else.

**For the common case - correcting a mistake - GitEHR already has the right answer, and it is not deletion.** As described in [Provenance](provenance.md), a factual correction is a new commit, not an edit to history: the original and the corrected version both remain visible, with the person who made the change identifiable. This matches how clinical records have always worked - you do not tear a page out of a paper notes folder, you countersign an amendment - and it is arguably a *better* GDPR posture than a database's silent `UPDATE`, because the data subject can see exactly what was wrong and what was corrected, rather than trusting an invisible edit.

**For genuine erasure requests - the rare case where data must actually become unrecoverable - the honest mechanism is cryptographic, not historical.** Once [encryption at rest](https://github.com/gitehr/gitehr/blob/main/spec/encryption-at-rest.md) lands, a patient's data is encrypted under a key the patient or their organisation controls. Destroying that key ("crypto-shredding") renders the ciphertext permanently unrecoverable without rewriting a single git object. This is the same technique used by other systems that face the immutable-history-versus-erasure tension, and it works because it targets confidentiality (can anyone still read this?) rather than trying to falsify history (can we pretend this was never written?), which is the property GitEHR deliberately will not compromise.

**What GitEHR does not attempt** is making committed git history itself mutable on request. Rewriting a signed, hash-chained history to remove a commit is indistinguishable, from a tooling perspective, from an attacker doing the same thing to hide tampering - so a "delete this entry" feature would quietly undermine the entire trust model in [Provenance](provenance.md). Cryptographic erasure of the *plaintext*, once encryption at rest ships, is the honest way to satisfy the regulation without pretending that problem away.

## Further reading

- [Files, not databases](files-not-databases.md) - the full cross-patient-query and lakehouse argument.
- [Provenance](provenance.md) - how corrections and audit trailing work today.
- [Encryption at Rest](https://github.com/gitehr/gitehr/blob/main/spec/encryption-at-rest.md) - the design discussion that crypto-shredding depends on (not yet built).
