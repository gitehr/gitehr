---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Common objections

GitEHR deliberately separates the durable clinical record from the systems used to query, edit, and exchange it. That division prompts reasonable questions.

## How do you query across patients?

A per-patient repository is not a substitute for an analytics database. A hospital, research network, or regulator can build a derived database from the repositories it is entitled to hold, then use that database for cohort discovery, reporting, operational dashboards, and research.

The important boundary is that the derived database is replaceable. It can be rebuilt with a different engine, schema, or vendor from the canonical records. Losing or changing an analytics system must not mean losing the clinical record. This is the same division of responsibility as a lakehouse: durable, portable files at the bottom; purpose-built query systems above them.

## How can several people edit one record?

Healthcare already has concurrent editing: a GP, hospital clinician, pharmacy, and patient may all create information about the same person. A central database often hides that conflict behind last-writer-wins behaviour or local reconciliation rules.

GitEHR makes divergent edits visible. Each contributor works from a complete record copy, and Git preserves the individual changes and their history. When changes affect the same clinical fact, they must be reconciled deliberately by an authorised person; the system must not silently discard a clinician's or patient's contribution. The repository format provides the evidence and the history needed for that review. A clinical interface can make the reconciliation workflow safe and comprehensible without changing the underlying record.

## What about ACID transactions and consistency?

ACID is valuable when many people update a shared operational database. GitEHR does not reject that need. It assigns it to the operational and derived-data layers, where a database is the right tool.

At the custody layer, the unit of change is a patient's record. A Git commit captures an atomic, reviewable update to that record and its journal. Cross-patient reports and workflows can use transactional databases built from record copies, while the canonical files remain independent of any one database product. This avoids making the survival of a fifty-year record depend on the survival of a particular transaction engine.

## What about the GDPR right to erasure?

This is the hardest objection, and an immutable record format does not remove the legal or operational responsibility to address it. The right to erasure is not absolute: health records may have statutory retention duties or other lawful grounds for continued processing. The applicable decision depends on the jurisdiction, purpose, and record type.

Where erasure is required, every holder of the record must participate in an agreed disposal process. Git history makes deletion a governed, auditable operation rather than a quiet overwrite, but it also means that retention, replication, encryption, backup expiry, and access control need explicit policy. GitEHR is a record format, not a substitute for clinical records governance or legal advice.

## Does a file format solve digital preservation?

No. Preservation also requires responsible storage, managed copies, access control, retention decisions, integrity checking, migration planning, and sustained organisational investment.

The [Digital Preservation Coalition's Bit List entry for Electronic Hospital and Medical Records](https://www.dpconline.org/index.php?option=com_content&view=article&id=4519:bitlist-electronic-hospital-medical-records&catid=127:bitlist-endangered&Itemid=989) identifies medical records as endangered material requiring immediate action. It observes that records may need to survive a patient's lifetime or support intergenerational study, while there is little evidence of the medical profession participating in the digital-preservation community. The DPC also identifies loss of context and integrity, poor migration planning, and ad-hoc records management as conditions that make loss more likely.

GitEHR addresses the format part of that problem. It is a file-based, archival-grade record format: human-readable narrative, self-contained files, open structured data where needed, explicit provenance, and versioned history can be inspected without a proprietary EHR or database. It makes preservation work possible, visible, and independently verifiable. It does not make that work optional. See [Longevity](longevity.md) for the design rationale.
