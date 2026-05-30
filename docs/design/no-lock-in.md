---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# No Vendor Lock-In

GitEHR is engineered to be an interoperable file format for clinical records, **free from Vendor Lock-In**.

## What is Vendor Lock-In?

Vendor Lock-in occurs when the 'switching costs' of moving from one EHR to another are prohibitively high because of data incompatibility. In traditional database-backed EHRs, the patient data in an is tightly coupled with the application's database structure, making it very difficult to separate the data from the software. In addition, the physical and technical mechanisms to export and transfer data are often limited, expensive, or incomplete.

One way to reduce to vendor lock-in is interoperability. For example, in the world of document file formats, the `.odf` or `.docx` file formats allow you to easily use different editor software, if the current one becomes unaffordably expensive.

In healthcare - and governmental IT in general - vendor lock-in seems to be a particular problem: *there are no interoperable file formats*. Even basic data in simple IT systems is wholly linked to the database schema of the software used.

## Vendor lock-in leads to problems throughout the entire software lifecycle

- **Migration challenges:** When switching EHR vendors, data migration can be complex, costly, and risky due to incompatible formats and structures.
- **Leaving costs:** Exiting a vendor relationship may involve significant fees, extended contracts, or technical barriers.
- **Risk of data loss:** Even successful migrations frequently lead to lossy or degraded data, which over time means the patient record 'decays' in quality and fidelity.

