# Safety

Clinical software is a regulated category in most healthcare jurisdictions. GitEHR is designed to be safer to deploy than the typical EHR by treating safety as a property that falls out of the architecture, not a feature that has to be bolted on.

This page sets out the principles. Specific evidence, hazard logs, and conformance work live under the [Turva project](https://github.com/pacharanero/turva), GitEHR's clinical safety case process.

!!! warning "Status: early"
    This page is a starting point. The Turva safety case is a work in progress.

## The safety argument in one line

A health record whose canonical form is plain files in a Git repository is, by construction, more trustworthy and more recoverable than one whose canonical form is rows in a vendor's database.

## Why this matters

The most consequential safety failures of computerised record-keeping in the last twenty years have been failures of trust in opaque, vendor-controlled systems whose audit trails could not be independently verified.

The most famous example is the [UK Post Office scandal](https://en.wikipedia.org/wiki/British_Post_Office_scandal). It was not a healthcare case, but the structural failure is the same one that can happen in any EHR: when the system, the system's vendor, and the organisation's leadership form a closed loop with the audit trail, the people the system is supposed to serve (clinicians and patients in our case, subpostmasters in theirs) have no way to mount an effective challenge when the system is wrong.

GitEHR's architecture makes that closed loop much harder to form.

## How the architecture supports clinical safety

### Tamper-evident provenance

Every journal entry is content-addressed via SHA-256 and chained to its parent (see [Tamper resistance](../design/redundancy.md#tamper-resistance) and [Provenance](../design/provenance.md)). Altering any past entry changes its hash and breaks the chain in every descendant. Detection is mechanical: `gitehr journal verify` walks the chain and reports any link that no longer matches.

This is the same content-addressed-hashing property that makes Git trustworthy as the substrate of essentially all modern software development. It does not require trusting any central server, vendor, or administrator.

### Separation of record from app

The canonical clinical record is a folder of plain-text files. The application that views or edits it is replaceable. If the vendor of a GUI dies, is acquired, or simply ships a bad release, the record itself is unaffected. See [No Lock-In](../design/no-lock-in.md) and [Longevity](../design/longevity.md).

This separation also makes safety review tractable: an external assessor can read the canonical files directly, without needing access to the vendor's database internals.

### Append-mostly journal

Clinical errors that overwrite information without recording who, when, and why are responsible for an outsized share of harm in current EHRs. GitEHR's journal is append-mostly: every change is a new entry, every entry is signed by an identified contributor, every entry is timestamped, and nothing in the history is silently overwritten. Corrections are themselves journal entries that reference what they correct.

### Redundancy across organisations

Every organisation caring for a patient holds a full clone of the record. Loss or corruption at any single organisation does not lose the record (see [Redundancy](../design/redundancy.md)).

### Offline-safe operation

The record is fully usable without connectivity. A clinician with an offline clone can read and write the record; sync happens when connectivity is restored. This avoids a common failure mode where a system "fails safe" by refusing to operate when the network is down, leaving clinicians without a record at the moment they need it most.

## Turva

[Turva](https://github.com/pacharanero/turva) is the in-progress clinical safety case for GitEHR. It will cover hazard analysis, conformance against the relevant safety standards (DCB0129 / DCB0160 in England, ISO 14971), and the safety case argument structure.

The TODO list, hazard log, and current state of the safety case live in the Turva repository rather than here. This page will link to specific Turva sections as they stabilise.

## Out of scope

Some things this page deliberately does not claim:

- That GitEHR is a medical device. It is a file format and a set of tools; whether a specific deployment is a medical device depends on the intended purpose of the deployment, not on GitEHR itself.
- That cryptographic provenance is a substitute for clinical governance. It supports governance by making the audit trail trustworthy; it does not replace the governance process.
- That the architecture eliminates the possibility of unsafe entries. A correctly recorded but clinically wrong note is still a clinically wrong note. The architecture makes such a note traceable and correctable, not impossible.
