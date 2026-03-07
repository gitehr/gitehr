---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

# Overview

GitEHR has been designed to solve the problems inherent to medical records and to be extensible in the future.

It aims to use low-tech solutions and be completely reliable.

Natively, GitEHR is a technical standard that no clinician would use directly. It is a foundational technology upon which other systems can be built. These higher-level systems will focus on improving clinical workflows.

# Design Principles

## Audit Trail



## Longevity

Health records need to last a long time. Sometimes, they exist before a person is born and last well after death.
However, healthcare software is a different matter.
Like all software, there is a lifecycle to its existence. Some companies fail, some grow, and some get acquired by bigger companies. Software tends to last less time than humans.
In the current closed-source EHR world, users and managers of patient data do not have access to the source code. Due to the inseparable tight coupling of patient data record structure to the database used inside the software platform, it becomes challenging to salvage the data in a usable form once the software dies.
GitEHR seeks to separate the clinical record from the software used to view and manage it. It will ensure patient records stand alone, separate from the database structure of any single proprietary software supplier.
Using time-tested, simple technologies - such as flat files, directories, and disks - helps reduce GitEHR's dependence on the latest **new and shiny** trend. It ensures that once viewing and editing software reaches its inevitable end of life, new software can seamlessly replace the old without affecting clinical care.

## No Lock-In

GitEHR is engineered to be an interoperable file format for clinical records, free from **Vendor Lock-In**.
Vendor Lock-in is a well-known term in the technology industry. It describes the result of data tied inherently to a specific product.
One mitigation is interoperable file formats. For example, the `.odf` or `.docx` file formats allow one to use a different editor software, if the current becomes unaffordably expensive, without providing sufficient value.
In healthcare - and governmental IT in general - vendor lock-in seems to be a particular problem: *there are no interoperable file formats*. Even basic data in simple IT systems is wholly linked to the database schema of the software used.
**Vendor lock-in leads to problems throughout the entire software lifecycle.**
<!-- NEED TO ADD MORE STUFF HERE
When new software is procured, the price for the 
Migration
Leaving
Loss of data -->

## Patient-Centricity

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

## Portability

Portability is a natural emergent property of complete [Patient-Centricity](patient-centricity.md). It is therefore baked into GitEHR.
When the files of a record are represented in a single directory on a disk, they can be shared easily with other caregiving organisations.
They can even be downloaded onto a portable storage medium, like a USB stick. This can be read, updated and used comprehensively in this state. A prime example includes if the patient goes off-grid because they are on an expedition or a colony ship to Mars. Later, changes made can be merged back into the record.

## Provenance

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

## Redundancy

Every organisation providing care to a patient keeps a complete copy of their entire GitEHR healthcare record. Health records are small in the context of modern data management.
All organisations to which a patient is granted access keep an up-to-date record copy.
Distributing the storage of medical records makes them extremely resilient to loss, deletion or corruption.
## Tamper resistance
Using the same consensus principle from blockchain technology to ensure there is only one version of the truth, GitEHR records which have been altered in any way are instantly identified.

## Security

...content from security.md...

## Simplicity

**The content of the health record should be in plain text, in files on disk**.
**In plain text**: content is readable even in low-tech environments. Ideally, a viewer will render this plain text into a more usable and feature-rich application view. But the text-on-disk view exists as a fallback.
**In files**: files can easily work interoperably across all operating systems.
**On disk**: as opposed to being in a database*.
!!! "Of course, databases are also on disk."
    Though databases also exist on disk, there is a difference between being buried in a database versus having easy access to a file.
## New entries are saved in a new file
Medical practice can usually be distilled to an 'atomic unit of care'. For a clinic visit, a single entry or a few linked entries could summarise the visit.
## GitEHR entries are linkable
As modern clinical records become more complex, clinicians need help finding specific data items.
The Internet has solved this problem by using the URL or Internet 'link'. Somehow, the most basic of technologies has not yet found its way into the current paradigm.

## Structure

...content from structure.md...
