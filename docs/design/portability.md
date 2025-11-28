---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Portability

Portability is a natural emergent property of complete [Patient-Centricity](patient-centricity.md). It is therefore baked into GitEHR.

When the files of a record are represented in a single directory on a disk, they can be shared easily with other caregiving organisations.

They can even be downloaded onto a portable storage medium, like a USB stick. The GitEHR record can still be readfrom ans written to in this state. For example, if the patient goes 'off-grid' because they are on an expedition - or for that matter a colony ship to Mars - they can take their entire medical record with them in a fully interactive way. 

Later, any changes made can be synchronised with their GitEHR records back home.

## Natural disasters and major incidents

In natural disasters or other major medical incidents, responders often need to capture information about care before patient identities can be established. Recording those details in a GitEHR repo with a temporary identifier ensures that, once a patient's identity is confirmed, disaster records can be **losslessly** reintegrated into their lifelong record. In most cases, the reconciliation process is as simple as appending a handful of new files to the end of their existing GitEHR record.

## Military and remote settings

In a military or deployable context, medical care of employed staff may occur in multiple locations. Currently this is a complex interoperability and information management challenge for armed forces and 'deployable' organisations such as disaster relief organisations. GitEHR solves much of this technical difficulty by allowing each deployed location to maintain a local or even fully offline copy of the patient's record, which can then be synchronised with the main record when connectivity allows.

With a GitEHR record, authorised contributors to the record anywhere in the world can be given appropriate levels of read or write access, according to their role and the sensitivity of the data. Each of these multiple contributors can 'pull' from and 'push' to GitEHR 'remotes' - the server chosen as the main 'source of truth', which can be located **anywhere**. No bespoke interoperability work is needed between systems, they simply need to read and write the GitEHR record.
 
