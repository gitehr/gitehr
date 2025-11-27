---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Patient-Centricity

Informaticians and health technologists have been proposing the idea of 'Patient-Centric Health Records' for a long time. They have been unable to deliver a patient-centric record because the EHR uses a database and is thereby inherently **organisation-centric**.

!!! success
    Digesting this concept is the most crucial step to understanding why progress cannot be made in EHR without changing the paradigm.

## One patient, one record

A GitEHR *clinical record* is only ever the record of **one** patient.

This ensures a clean separation of the data in any other patient records and organisational data.

Organisations giving care keep a local record of the patient's care.

## Separation from Organisational information

A GitEHR clinical record is completely separated from the records of other patients and any organisation-level information.

This separation allows multiple organisations to contribute to the GitEHR, akin to how numerous contributors can work on the same Git repository.

It also allows database operations to be performed over multiple patients if required.

An organisation-level database is useful as a healthcare organisation may care for thousands or even millions of patients. But this must never be the canonical Source of Truth of the patient record.
