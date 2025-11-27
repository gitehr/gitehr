---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Portability

Portability is a natural emergent property of complete [Patient-Centricity](patient-centricity.md). It is therefore baked into GitEHR.

When the files of a record are represented in a single directory on a disk, they can be shared easily with other caregiving organisations.

They can even be downloaded onto a portable storage medium, like a USB stick. This can be read, updated and used comprehensively in this state. A prime example includes if the patient goes off-grid because they are on an expedition or a colony ship to Mars. Later, changes made can be merged back into the record.

## Natural disasters and major incidents

In natural disasters or other major medical incidents, responders often need to capture fragments of care before identities are fully known. Recording those details in a GitEHR-compatible way ensures that, once a patient's identity is confirmed, each fragment can be reintegrated into their lifelong record. In most cases, the reconciliation process is as simple as appending a handful of new files to the end of their existing GitEHR record.

To avoid conflicts when multiple teams are working quickly, fragments can be committed using temporary identifiers and stored as standalone directories (or even zipped bundles) that follow the GitEHR folder conventions. When the individual's identity is established, those commits can be merged onto the canonical patient branch just like any other upstream contribution, preserving provenance while keeping the record append-only. This mirrors common Git workflows for reconciling offline branches after a network outage or deployment freeze.
