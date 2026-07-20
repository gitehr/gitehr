<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Encryption at Rest

**Status:** Research and design discussion. No decision made yet. This document surveys how the wider healthcare-software industry achieves encryption at rest, explains why GitEHR's file-and-git model needs a different approach from the dominant database-backed one, and lays out the candidate approaches and open questions. It is intended to feed a future grilling session and an ADR once a direction is chosen.

Related: `gitehr encrypt` / `gitehr decrypt` are currently placeholders (they write a `.gitehr/ENCRYPTED` marker and state an intent to encrypt `journal/`, `state/`, `imaging/`, and `documents/` with AES-256-GCM, leaving `.gitehr/` in the clear). The immutability and verifiability model that encryption must not break is recorded in the Documents ADRs ([ADR-0002](adr/0002-record-only-grows.md): the record only grows; [ADR-0003](adr/0003-directory-documents-with-manifest.md): SHA-256 references anchor integrity).

## Why encryption at rest matters for GitEHR

Two facts about HIPAA drive the architecture, and both are easy to miss:

1. **It is "addressable," not "required."** The HIPAA Security Rule (45 CFR 164.312(a)(2)(iv)) lists encryption of data at rest as an *addressable* implementation specification. A covered entity must either implement it or document a written justification for an equivalent alternative. In practice everyone implements it, because of the second fact.

2. **The breach safe harbor is the real driver.** Under the Breach Notification Rule, if PHI has been rendered "unsecured" - i.e. encrypted to the HHS/NIST standard - and the encryption keys were *not* also compromised, then loss or theft of that data is **not a reportable breach**. The governing guidance points to **NIST SP 800-111** for data at rest, and the crypto must be performed by a **FIPS 140-2 / 140-3 validated module**. This is why vendors care so much about *which* crypto implementation and *where the keys live*: "we encrypted it" only earns the safe harbor if it is done to spec with proper key separation.

The design goal is therefore not "encrypt the bytes." It is "encrypt the bytes with a validated module, such that compromising the storage does not compromise the keys."

Encryption at rest is also only one control among many (access control, audit logging, transmission security, integrity, plus administrative and physical safeguards). There is no official "HIPAA certified" stamp; compliance is an ongoing organisational program, not a build artifact.

## How database-backed healthcare systems do it

DB-backed systems almost always stack several layers, because each defends against a different threat. This is the menu GitEHR is implicitly being compared against.

**1. Full-disk / volume encryption (FDE).** LUKS/dm-crypt, BitLocker, or cloud volume encryption (EBS, Azure Disk, GCP Persistent Disk). Defends against physical theft of a disk or decommissioned hardware. Blind spot: once the OS is running and the volume is mounted, a compromised application or an authorised database session sees plaintext. Table stakes, not sufficient alone.

**2. Transparent Data Encryption (TDE).** The database engine encrypts its data files, logs, and backups on write and decrypts on read, invisibly to the application. SQL Server / Azure SQL TDE, Oracle TDE (used by Oracle Health / Cerner), PostgreSQL via cloud-managed RDS/Aurora/Cloud SQL, MySQL/InnoDB. Defends against stolen data files, stolen backups, and leaked snapshots. Blind spot: anyone with a valid DB session still gets plaintext, so SQL injection or a rogue DBA bypasses it entirely. It is transparent precisely because it is invisible to authenticated access.

**3. Application-level / column (field) encryption.** The application encrypts specific sensitive fields *before* they reach the database (or uses an engine feature such as SQL Server "Always Encrypted," where the server never sees the plaintext keys). Defends against a compromised database, malicious DBAs, and the injection case TDE misses. Cost: server-side indexing, search, range queries, and joins on those columns become impractical, and the application owns key rotation. Used surgically on the most sensitive fields, not on everything.

**4. Backup encryption.** Called out explicitly because forgotten or unencrypted backups and snapshots are one of the most common real-world breach sources. TDE usually covers this; a hand-rolled scheme must not forget it.

### Key management is the hard part, not the cipher

The algorithms are commodities (AES-256-GCM is the usual choice, and is what GitEHR already names). What separates a compliant system from a checkbox is **key management**, and the dominant pattern is **envelope encryption**:

- Data is encrypted with a **DEK** (data encryption key).
- The DEK is itself encrypted with a **KEK** (key encryption key) that lives in a dedicated key manager and never touches the data store.
- The key manager is a **KMS** (AWS KMS, Azure Key Vault, GCP KMS, or HashiCorp Vault), often backed by an **HSM** (FIPS 140-2/3 Level 3 hardware).

This buys the properties auditors actually look for: **key rotation** without re-encrypting all data (rotate the KEK and re-wrap the DEKs), **separation of duties** (people with DB access cannot read the keys), **per-use access logging**, and **revocation** (destroy the KEK and the data is cryptographically gone). A related lever for enterprise health customers is **BYOK / CMK** (bring-your-own-key / customer-managed keys): the covered entity controls the KEK in its own KMS tenancy, so even the SaaS vendor cannot decrypt without it, which also cleanly assigns key custody in the Business Associate Agreement.

### The typical modern stack

For a cloud-hosted, DB-backed health application today the common baseline is: a cloud-managed database with **TDE on, using a customer-managed KMS key**; **volume encryption** underneath; **TLS** for data in transit; **field-level encryption** for a small set of the most sensitive identifiers; encrypted backups and snapshots; a documented key-rotation policy; and a **BAA** with the cloud provider. Epic is the partial outlier worth knowing: it runs on InterSystems IRIS/Caché rather than a SQL RDBMS, and large deployments are on-prem or Epic-hosted, but the same layering logic applies.

## Why GitEHR is different

GitEHR's PHI is **files in a git working tree and git's object store**, not rows in an engine. That removes some options and sharpens others:

- **TDE has no analogue.** There is no engine performing transparent encrypt/decrypt, so the menu collapses to **file/blob-level encryption** (the application-level layer above) plus **FDE** of the host.
- **AES-256-GCM over the four data directories** (GitEHR's stated plan) is a reasonable primitive, but the consequential decision is *where the encryption boundary sits relative to git*, and how it interacts with the SHA-256 integrity chain.
- **Key management is harder, not easier**, because the record is meant to be offline-capable, patient-held, and shareable across organisations. There is no central KMS to lean on.

## Candidate approaches for GitEHR

### A. Encrypt-then-commit (ciphertext lives in the repo)

Clinical files are stored as ciphertext; `.gitehr/` stays in the clear for repository detection and key metadata (as the current placeholder describes).

- **Pros:** self-contained and portable - the repo is encrypted wherever it travels, satisfying "at rest" in the strongest sense. No reliance on host FDE for the safe harbor.
- **Cons:** git cannot delta or diff ciphertext, so every change rewrites a whole blob and history bloats. Journal reads, `document verify`, and the GUI must decrypt to function. **Critical interaction:** Git's content-addressed history and Document references must define whether they address *plaintext* or *ciphertext*. Addressing plaintext preserves clinical-content integrity semantics but requires the key; addressing ciphertext lets object integrity be checked without the key but ties the proof to a specific encryption.

### B. Transparent git filters (plaintext working tree, ciphertext blobs)

Use `.gitattributes` clean/smudge filters (the `git-crypt` approach, or `age` as the cipher) so the working tree is plaintext but committed blobs are ciphertext.

- **Pros:** preserves developer and tooling ergonomics - everything operating on the working tree sees plaintext.
- **Cons:** the filter model is fiddly and easy to misconfigure (a missing attribute silently commits plaintext). It leaks metadata - filenames, file sizes, and which files changed in which commit are all visible. `git-crypt` specifically uses a deterministic AES mode so identical plaintext yields identical ciphertext, which leaks equality.

### C. Encrypt at the transport / at-rest boundary only

Rely on host **FDE** for the live working copy and encrypt at the packaging boundary (`gitehr transport create --encrypt` already gestures at this).

- **Pros:** simplest; no change to how git or the integrity chain work.
- **Cons:** "at rest" then means "the disk," which only earns the breach safe harbor against physical theft, not against a compromised host or a copied repo directory. Weakest of the three for HIPAA purposes.

## Key management for an offline, patient-held, multi-party record

This is the genuinely hard design problem and the reason this needs an ADR rather than a quick choice.

- With no central KMS, the KEK has to be derived or held locally: a **passphrase-derived key** (Argon2id KEK), optionally wrapped to hardware (YubiKey/PIV, a TPM, or Apple Secure Enclave).
- The record is meant to be **shared across organisations while remaining patient-controlled**, which is the multi-recipient case: the patient *and* each treating organisation must be able to unwrap the data key. That points at envelope encryption with **per-recipient wrapped DEKs** (the model `age` uses with multiple X25519 recipients, or any "encrypt the DEK once per public key" scheme). Adding or revoking an organisation then means re-wrapping the DEK, not re-encrypting the record - but revocation of *past* access is impossible once a party has held the plaintext, which must be stated honestly.
- This interacts with the contributor model already in the record (`.gitehr/contributors.json`, active author): recipients and contributors are related but not identical concepts and the relationship needs to be defined.

## Crypto primitive and library considerations

- **AES-256-GCM nonce management.** GCM uses a 96-bit nonce that must never be reused under the same key; nonce reuse is catastrophic (it leaks the authentication key). For file-at-a-time encryption with many files and possible key reuse, a misuse-resistant choice is safer: **AES-GCM-SIV**, or **XChaCha20-Poly1305** (192-bit random nonce, which `age` uses). Worth deciding deliberately rather than defaulting to GCM.
- **FIPS validation is a real gotcha in Rust.** The breach safe harbor wants a FIPS 140-2/3 validated module. The popular Rust options - `ring` and the `RustCrypto` crates (including `aes-gcm`, `sha2`) - are **not** FIPS validated. The realistic path to a validated module in Rust today is **`aws-lc-rs`** in its FIPS mode (backed by the FIPS 140-3 validated AWS-LC). If the safe harbor matters, the crypto crate choice is constrained from the start, and it is cheaper to design around `aws-lc-rs` than to retrofit it.
- Note that GitEHR already depends on `sha2` (RustCrypto) for the integrity chain; if FIPS validation becomes a requirement, the hashing path is in scope too, not just the cipher.

## Open questions to resolve in the ADR

1. Which approach: A (encrypt-then-commit), B (git filters), or C (transport/FDE only) - or a hybrid (e.g. A for the data dirs, FDE underneath)?
2. Does the SHA-256 integrity chain hash plaintext or ciphertext under encryption?
3. What is the key-management model for offline, patient-held records, and how does multi-organisation access map onto wrapped-DEK recipients?
4. How do encryption recipients relate to the existing contributor roster?
5. Is the FIPS 140-3 safe harbor an explicit requirement? If so, standardise on `aws-lc-rs` for both cipher and hash.
6. Which AEAD: AES-256-GCM, AES-GCM-SIV, or XChaCha20-Poly1305?
7. What metadata leakage (filenames, sizes, change timestamps) is acceptable, and does that rule out approach B?

## References

- HHS, "Guidance to Render Unsecured Protected Health Information Unusable, Unreadable, or Indecipherable to Unauthorized Individuals" (the source of the breach safe harbor, citing NIST SP 800-111 for data at rest).
- NIST SP 800-111, "Guide to Storage Encryption Technologies for End User Devices."
- HIPAA Security Rule, 45 CFR 164.312 (technical safeguards).
- FIPS 140-3, "Security Requirements for Cryptographic Modules."
- `age` file encryption (X25519 recipients, ChaCha20-Poly1305); `git-crypt` (gitattributes filters); `aws-lc-rs` (FIPS mode).
