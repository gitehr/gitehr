---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Portability

Portability is a natural emergent property of complete [Patient-Centricity](patient-centricity.md). It is therefore baked into GitEHR.

When the files of a record are represented in a single directory on a disk, they can be shared easily with other caregiving organisations.

They can even be downloaded onto a portable storage medium, like a USB stick. This can be read, updated and used comprehensively in this state. A prime example includes if the patient goes off-grid because they are on an expedition or a colony ship to Mars. Later, changes made can be merged back into the record.

This portability also suits a military or battlefield context, where a single service member may be posted across multiple locations and treated in different hospitals, each running its own systems. With a GitEHR record, the individual can arrive with an offline but up-to-date copy of their complete medical history on a USB stick, import it into the local record for the duration of the posting, and then export the updated record losslessly to carry to the next assignment. No bespoke interoperability work is needed between those disparate systems; they simply need to read and write the GitEHR record.
