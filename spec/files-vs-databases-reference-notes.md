<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Files vs databases: reference notes

Reading notes on two reference documents - the source PDFs are in [`docs/assets/pdf/`](../docs/assets/pdf/), their machine-extracted markdown in [`spec/research/`](research/). They bear directly on GitEHR's "files, not databases" thesis - one supports it, one is the strongest mainstream argument against it. This note records what is useful from each and how GitEHR positions relative to them. It is internal design context; the public-facing version of the argument lives in `docs/design/files-not-databases.md`.

## 1. "Traditional Databases vs Flat Text Files for Long-Term Medical Records" (the supportive essay)

A researched essay comparing DBMS storage and flat-text storage for EHRs, focused on **longevity and interoperability rather than raw performance**. Its strongest, citable points - useful for the site, talks, and the FAQ:

- **Format obsolescence is the real long-term risk.** Proprietary/binary database formats can become unreadable when the vendor, OS, or hardware moves on; plain text (ASCII/UTF-8, CSV, JSON, XML) has been readable for decades and almost certainly will be in 30-50 years. Digital archivists (Digital Preservation Coalition; Library of Congress) explicitly recommend exporting to **vendor-neutral text or SQLite** for preservation, precisely to avoid opaque formats.
- **Legacy migration is a chronic, expensive failure mode.** ~two-thirds of healthcare organisations (HIMSS) struggle to move legacy data into new systems because of proprietary/non-standard formats; the common band-aid is keeping the **old EHR running read-only** just to retain access. GitEHR's portability is a direct answer to this.
- **Integrity without a live DBMS is a solved problem when records are immutable.** Once a flat record is written and made read-only, append-only logs plus **checksums / cryptographic signatures and periodic re-signing** make tampering detectable - this is exactly GitEHR's journal (hash-verifiable, append-only, git-backed). The essay frames DB ACID guarantees as strongest *during active use*, while flat files win on *long-term custody*.
- **Tooling longevity.** Plaintext is workable with universal tools (`grep`, `diff`) decades later - the same property that makes a Git history of markdown legible to an LLM (GitEHR's agentic angle).
- **VistA as the honest counter-example.** The VA's MUMPS/Caché database held veterans' records for ~40 years *because it was open and well-documented* - but still hit an "archiving problem" when migrating. Lesson: a database *can* endure if actively maintained and open, but eventually faces the migrate-or-die question that a file-based archive sidesteps.
- **Hybrid is the pragmatic norm.** Active data in a DB, periodic snapshots/exports to flat files for immutable archive. This is essentially the lakehouse pattern (below) and is compatible with - not contrary to - GitEHR.

The essay corroborates GitEHR's design choices (append-only journal, hashes/signatures, plain-text + open standards, portability, openEHR's vendor-independence goal). It is reference support, not novel to us.

## 2. Stonebraker & Pavlo, "What Goes Around Comes Around... And Around..." (the counter-argument)

A retrospective survey of ~60 years of data models (SIGMOD Record, 2024). Thesis: the **relational model (RM) and SQL keep winning**; alternative data models (hierarchical, network, object, XML, document/JSON, NoSQL, graph, vector, ...) come around repeatedly, fade, and have their good ideas **absorbed into SQL/RDBMSs**. Engine-level progress (column stores, vectorisation) was driven by changing hardware, not by abandoning the RM. This is the most credible mainstream view that would be sceptical of "store the record as flat files."

### Taking the challenge seriously

A naive reading says GitEHR contradicts a 60-year empirical pattern. Three of the paper's points sting if mis-applied to GitEHR:

- Document/JSON stores were re-litigated and judged to offer no durable advantage over RM+JSON columns for querying.
- "Schemaless" storage pushes schema-on-read cost and inconsistency onto every consumer.
- Hand-rolled stores reinvent - usually worse - what mature DBMSs already do (indexing, query optimisation, concurrency).

If GitEHR were proposing flat files as a **query/analytics engine**, Stonebraker would be right and we would lose.

### Why GitEHR is not actually in that fight

The reconciliation is that Stonebraker is arguing about the **active data-management / query layer**, while GitEHR is making a claim about the **custody / source-of-truth layer**. They are different layers, and GitEHR concedes the query layer to exactly the technology Stonebraker champions:

- **Different unit and workload.** GitEHR's unit is *one patient's record* (megabytes, mostly read, append-only, longevity-critical) - not a billion-row multi-tenant analytical table. The "DBMS wins at scale" arguments are about the latter. A single patient record does not need a DBMS to be correct or fast.
- **Custody vs query.** GitEHR optimises for properties RDBMSs do *not* primarily target: 50-year readability, cryptographic tamper-evidence, distributed patient ownership, offline portability, and a full audit DAG. SQL optimises for ad-hoc query over mutable shared state. Choosing files here is not "rejecting the RM", it is choosing the right tool for custody.
- **GitEHR agrees you build a database for querying - a *derived* one.** Population health, research, and cross-patient queries are served by derived databases built *from* the canonical files (the **lakehouse / Apache Iceberg over Parquet** pattern, and Pat Helland's *Immutability Changes Everything*, both already cited in our materials). Canonical immutable files at the bottom; query engines - very possibly relational - on top. This is the same hybrid the supportive essay endorses, and it means Stonebraker's "RM keeps winning" can be *true at the query layer* without undermining files at the custody layer.
- **The good idea RDBMSs did not absorb.** Stonebraker's own through-line is that SQL absorbs good ideas. Content-addressed, immutable, distributed version control (Git / Merkle DAGs) is a genuinely good idea that mainstream RDBMSs have *not* absorbed. GitEHR brings it to the custody layer; if it is a good idea, the paper's own logic predicts it gets absorbed, not that it is wrong.

### What we should take from it (honest lessons)

- **Do not try to be a query database over files.** Resist the temptation to bolt a bespoke query/indexing engine onto the repository. Delegate querying to derived databases (or let an LLM read the files directly for per-record questions). This keeps us out of the fight we would lose.
- **Schema-on-read is a real cost.** Our structured layers (FHIR/openEHR resources, calculator input definitions, state files) matter precisely because "just files" pushes interpretation onto consumers. Keep investing in explicit, validated schemas over the files rather than leaning on convention.
- **Frame the pitch as custody/longevity, not as "databases are bad."** Against this audience, "the canonical record is files; build whatever databases you like on top" is defensible; "files beat databases" is not.

## Cross-references

- `docs/design/files-not-databases.md` - the public-facing argument.
- Roadmap site-content notes already cite the lakehouse/Iceberg framing, Pat Helland's *Immutability Changes Everything* (2015), Kleppmann et al.'s local-first paper, and the N(N-1)/2 integration argument - the FAQ item on cross-patient queries for research/population health is precisely the "derived database on top" answer to Stonebraker.
- `spec/gitehr-patient-activated-extraction-synopsis.md` - related research synopsis.
