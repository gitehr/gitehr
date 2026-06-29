# Record provenance and acquisition

*Status: proposal (draft). Two linked gaps the current model under-serves: (1) where each fact came from and how strongly it is asserted (provenance / evidence-level), and (2) the workflow of acquiring your record from the organisations that hold it (SARs / portal extraction / paper), and tracking that acquisition. Relates to the patient-activated-extraction synopsis - which covers extraction agents, but not acquisition tracking or per-fact provenance - and to #10. Drafted from operating a real record and running a real Subject Access Request in parallel; the field lists below are the ones that proved necessary.*

## Part 1 - Provenance and evidence-level

### The gap

Journal entries carry `author` and `timestamp`, and git gives tamper-evidence. That answers "who typed this into the repo, and when." It does not answer:

- **Source of the assertion** - self-reported by the patient, asserted by a treating clinician, extracted from a portal (e.g. NHS App), received via a Subject Access Request from a named controller, transcribed from a paper Lloyd-George envelope, captured from a device, or inferred from population priors.
- **Evidence level / strength** - documented-with-source vs inferred vs assumed. Concrete real example: immunity status *evidenced by a serology certificate* must not render identically to immunity *inferred from a childhood-schedule population prior*. Both are useful; conflating them is dangerous, because an inference must never masquerade as a record.
- **Verification status** of a clinical claim (working vs confirmed) - overlaps with `Condition.verificationStatus` (see `problem-condition-list.md`).

### Proposed: a small, reusable provenance block

A common optional `provenance` structure usable on journal entries, documents, and typed-state entries (conditions, observations, immunisations, ...):

```yaml
provenance:
  source_type: self-reported | clinician-asserted | portal-extracted | sar | paper-transcribed | device | inferred
  source_detail: <free text - controller name, portal name, device, citation>
  acquired_via: <acquisition-id>          # links to Part 2, when applicable
  evidence_level: documented | inferred | assumed
  document_ref:                            # the artifact that substantiates it, if any
    path: documents/<...>
    sha256: <hex>
  confidence: high | medium | low          # optional
```

This is metadata *about* an assertion. It does not conflict with ADR-0004: it annotates provenance, it is not derived clinical structure. For Medical-Markdown-derived data, `source_type` flows from the originating entry.

### Why it matters

- Lets the summary distinguish "confirmed and sourced" from "patient-reported" from "inferred" - the difference between a record and a guess.
- Underpins safe LLM use over the record: an agent must know a value's strength before acting on it.
- Gives the inferred-vs-documented display that the real record found essential a structured basis, rather than relying on prose hedging.

## Part 2 - Record acquisition workflow (SAR / portal / paper)

### The gap

The patient-activated-extraction synopsis covers *portal extraction agents* - the clever, novel part. But most of a person's record is acquired by boring, stateful, multi-week processes: Subject Access Requests to each data controller, portal pulls, and scanning paper. Acquisition has state that must be tracked, and GitEHR models none of it:

- which organisations (data controllers) hold parts of your record - and the controller is often not the site you were treated at;
- what was requested, when, and under what right (UK GDPR Article 15);
- identity provided, acknowledgement / clock-start, and the statutory due date;
- status: drafted -> sent -> acknowledged -> received -> partial -> nil/destroyed -> refused;
- what came back, where it was filed, and - importantly - what was confirmed destroyed under a retention schedule. A documented gap is itself a record; silence is not the same as "nothing existed."

### Why GitEHR specifically should hold this

1. It is the on-ramp that *populates* the record - the activation-energy problem the project explicitly cares about. The repo is empty until you acquire; acquisition is the cold-start workflow.
2. It is longitudinal and provenance-bearing. An acquisition record is the *source* that newly-imported entries point at (Part 1's `acquired_via`). "These 14 documents arrived from York Teaching Hospitals via SAR on 2026-07-10" is exactly the provenance that an imported blob otherwise lacks.
3. It generalises. Every activated patient - and any clinician or relative assembling a record - does this. Battle-tested this week: a real SAR went to a private hospital, and the tracking needs were concrete and non-obvious. The legal data controller differed from the hospital site; the request went to a different mailbox than the published DPO address (so "which address did it actually go to" matters); the one-month clock interacts with ID verification; and "records confirmed destroyed" needs a first-class status, not an absence.

### Proposed model

An acquisition register as typed-state (or a dedicated `acquisitions/` area), one record per request:

```yaml
- id: <acq-id>
  controller: <legal data controller>      # often != the treating site
  site: <hospital/practice treated at>
  contact_used: <email/portal/postal address actually used>
  care_context: <what this request is chasing>
  right_invoked: UK-GDPR-Art-15
  identifiers_provided: [NHS: <...>, MRN: <...>]
  date_sent: <date>
  id_provided: <what ID was attached>
  ack_date: <date>          # statutory clock start
  due_date: <date>          # one month from receipt/ID; note weekends/bank holidays
  status: drafted | sent | acknowledged | received | partial | nil-destroyed | refused
  outcome: <free text>
  filed_to: [<refs to journal entries / documents>]
  notes: <free text>
```

On "received", imported documents and journal entries carry `provenance.acquired_via = <acq-id>`, linking Parts 1 and 2.

Optional helpers:

- `gitehr acquisitions add | list | update` (same shape as `allergies`/`conditions`: current-state register, journal entry per status change).
- A UK-GDPR-Article-15 SAR letter template generator. The project already owns the patient-activated thesis; this is its unglamorous but essential companion - the thing that turns the thesis into a populated repo.

### Respecting existing decisions

- Same shape as the other typed-state primitives: each status change writes a journal entry (append-only audit), the register row is the overwrite-in-place current view.
- Not a query engine: a simple register plus filtered views (e.g. "everything overdue").

## Summary

Part 1 makes every fact carry where it came from and how strongly it is believed. Part 2 tracks the multi-week, multi-controller process of getting the facts in the first place, and wires its output back into Part 1 as provenance. Together they turn "a pile of imported files" into "a sourced, longitudinal record" - and they are drawn from doing exactly this by hand, not from speculation.
