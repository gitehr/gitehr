---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Provenance

Clinicians need absolute trust in the integrity of information in the clinical record. GitEHR has been designed to ensure that its containing information is completely trustworthy whilst remaining a distributed and decentralised system.

This is achieved using existing, reliable and time-tested technology used to establish trust in other parts of tech, including:

* Cryptographic signatures
* Cryptographic hashes

Using these technologies to sign and audit-trail commits to the clinical record already represents a significant advance on the current 'state of the art'.

## Existing audit trailing is broken

In the current typical Electronic Health Record (EHR) program, the clinical entries are saved into a database, often with (but not always!) an audit trail built into the system. This highlights any potential tampering of records by a nefarious user attempting to doctor the details of an incident to the system admins.

This places the locus of trust **entirely** on the system admins, the organisation's leadership, and the EHR design. There has never been a safe, successful, or effective example of placing this level of trust on such few people.

### [The UK Post Office Scandal](https://en.wikipedia.org/wiki/British_Post_Office_scandal)

This was an example of why clinicians cannot trust their careers and lives to organisational database-level audit trailing.

A series of errors occurred in the system used to manage the Post Office's banking service. This resulted in large discrepancies, including huge cash shortfalls in the cash drawers at the end of the day.

Each of these discrepancies was the *fault of the system and its design*.

However, its admins believed the system, *not the users*. The IT company which built the system believed the system, not the users. The Post Office's senior leadership backed the system for which they'd paid millions, not their staff.

Many people were jailed, some committed suicide, and many thousands of lives were ruined. An audit trail was built into this system, but it didn't work.

Similar discrepancies are equally likely in clinical records. A simple example is if an EHR failed to show a severe allergy to a clinician. The clinician administers a drug which kills the patient. The **system is at fault** but the clinician is scapegoated. The system admins and organisational leadership will not accept responsibility, jeopardising their multi-million-pound investment.

Another danger is that a user with admin access can be induced to falsify records in the audit trail, which would be essentially undetectable under the current paradigm.

### Why GitEHR is different

Entries in a GitEHR are each cryptographically signed. This means the entry content is followed by a cryptographic signature of the content, which can be mathematically proved to have been created only using that specific clinician's *private key*. Assuming private keys are kept secure, there isn't enough computation power on the planet to overcome this proof.

Any changes to the clinical entry content result in an unpredictably different cryptographic signature, making it invalid and immediately exposing tampering.

GitEHR entries are sequential: each commit has a direct parent commit. This way, entries cannot be reordered without changing the content, thus invalidating the signature.

Legitimate factual changes are supported using this same mechanism. The change is made transparent through the commit tree. Both the original and updated versions can be viewed, with the person who made the change becoming easily identifiable.

GitEHR is completely open source, meaning that 'with enough eyes looking at a problem, all bugs are shallow'. We expect these fundamental GitEHR trust features will be extremely safe and reliable.
