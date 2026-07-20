---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Redundancy

Every organisation providing care to a patient keeps a complete copy of their entire GitEHR healthcare record. Health records are small in the context of modern data management.

All organisations to which a patient is granted access keep an up-to-date record copy.

Distributing the storage of medical records makes them extremely resilient to loss, deletion or corruption.

## Tamper resistance

Each GitEHR journal entry is committed to Git as it is added. Git content-addresses every commit and file blob, so corruption or a doctored object is detectable in a known history - the same [Merkle DAG](https://en.wikipedia.org/wiki/Merkle_tree) property that makes Git trustworthy as the substrate of essentially all modern software development.

`git fsck` checks object integrity, but Git alone cannot distinguish a valid rewritten history from the expected one. Enforcing GitEHR's higher-level invariants - that the journal only ever grows, and that entries come from authorised contributors - is the role of a planned policy checker and server-side guardian.
