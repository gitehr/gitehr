---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Redundancy

Every organisation providing care to a patient keeps a complete copy of their entire GitEHR healthcare record. Health records are small in the context of modern data management.

All organisations to which a patient is granted access keep an up-to-date record copy.

Distributing the storage of medical records makes them extremely resilient to loss, deletion or corruption.

## Tamper resistance

Each GitEHR journal entry stores the cryptographic hash of its parent entry, and is itself identified by the hash of its own content. This builds a [Merkle DAG](https://en.wikipedia.org/wiki/Merkle_tree): altering any past entry changes its hash, which breaks the chain in every descendant.

Detection is mechanical. A `gitehr journal verify` walk recomputes hashes from genesis to the latest entry and reports any link that no longer matches. There is no central server, no consensus protocol, and no trusted administrator. This is the same content-addressed-hashing property that makes Git trustworthy as the substrate of essentially all modern software development.
