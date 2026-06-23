<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Glossary

## GitEHR-specific concepts

- **Document** - A clinical source artifact stored as a file in the patient record: a PDF, scanned letter, photograph, Word document, or imaging study. Documents are immutable and write-once, and are referenced by one or more journal entries.
- **Organisation-centric** - Records structured around the healthcare organisation rather than the patient.
- **Patient-centric** - Records structured around one patient, independent of any single organisation.
- **Main Patient Index (MPI)** - A single index file (`gitehr-mpi.json`) at a store root that resolves identifiers (such as an NHS number or MRN) to a canonical subject and repository path, with create/link/merge semantics. Each repository in a store represents one *subject* - usually a patient, but the index generalises to any entity (for example buildings or devices), so in a non-clinical deployment it can be read as a Main *Subject* Index. *Previously called the **Master Patient Index**; GitEHR dropped "Master" for the same reasons GitHub renamed the default branch from "master" to "main". The **MPI** acronym is unchanged.*
- **Hacktitioner** - A light-hearted term combining “hacker” and “general practitioner”.
- **Lossy** - A transformation where output may omit information present in the input.

## External references

- **Git** - https://git-scm.com/
- **Distributed Version Control System (DVCS)** - https://en.wikipedia.org/wiki/Distributed_version_control
- **Centralised Version Control System (CVCS)** - https://en.wikipedia.org/wiki/Version_control#Centralized_systems
- **Electronic Health Record (EHR)** - https://en.wikipedia.org/wiki/Electronic_health_record
- **FHIR** - https://www.hl7.org/fhir/
- **OpenEHR** - https://www.openehr.org/
- **NHS** - https://www.nhs.uk/
- **NPfIT** - https://en.wikipedia.org/wiki/National_Programme_for_Information_Technology
- **RCGP** - https://www.rcgp.org.uk/
- **W3C** - https://www.w3.org/

## Abbreviations

*[CVCS]: Centralised Version Control System
*[decentralised]: Decentralised systems have replaced a central locus of control with a distributed mechanism.
*[distributed]: Distributed systems can achieve useful work without needing a single central point of coordination. Similar to Decentralised.
*[DVCS]: Distributed Version Control System
*[EHR]: Electronic Health Record
*[FHIR]: Fast Healthcare Interoperability Resource, a standard for the structure of healthcare records and
*[Git]: A Distributed Version Control System (DVCS)
*[GP]: General Practice (the UK term for Family Medicine)
*[Hacktitioner]: A bad joke I made up - it's a play on 'Hacker' and 'General Practitioner'
*[HTML]: Hyper Text Markup Language
*[IT]: Information Technology
*[lossy]: A lossy transformation of data is one in which the output may have lost data compared to the input. The opposite is 'lossless'.
*[MPI]: Main Patient Index (formerly Master Patient Index) - the index that resolves identifiers to a canonical subject and repository in a GitEHR store.
*[NHS]: National Health Service - the taxpayer-funded universal health care system in the UK, free at the point of use, and much loved by the British.
*[NPfIT]: The UK NHS National Project for IT was a plan to build a single health record across all of the NHS. It collapsed under its own weight after 12.5 billion GBP of taxpayer money had been ploughed into it.
*[OpenEHR]: OpenEHR is a standard for the structure of healthcare records.
*[organisation-centric]: Records which are structured so as to have the healthcare organisation as the primary focus, as opposed to being 'patient-centric'.
*[RCGP]: The Royal College of General Practitioners
*[W3C]: World Wide Web Consortium
