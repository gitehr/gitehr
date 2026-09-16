<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `gitehr acquisitions`

Status: implemented v1 (Part 2 of
[`record-provenance-and-acquisition.md`](../record-provenance-and-acquisition.md)).

Manages the record-acquisition register in `state/acquisitions.md`: the
multi-week, multi-controller process of getting parts of a record from the
organisations that hold it (Subject Access Requests, portal pulls, paper).
This is typed state, following the same pattern as `gitehr allergies` and
`gitehr conditions`: the state file is the current queryable register; each
mutation also writes a journal entry as the audit narrative.

Not yet implemented: the reusable per-fact `provenance` block from Part 1 of
the spec (linking imported journal entries/documents back to an acquisition
via `acquired_via`), and a UK-GDPR-Article-15 SAR letter template generator.

All subcommands require the current directory to be a GitEHR repository.

## Storage

`state/acquisitions.md` is YAML front matter with a top-level `acquisitions`
array. Each entry is one acquisition request.

```yaml
---
acquisitions:
  - id: ACQ-20260710T090000Z-4f2a9c1b
    controller: York Teaching Hospitals NHS Trust
    site: York Hospital
    contact_used: dpo@york.nhs.uk
    care_context: 2019 discharge summary
    right_invoked: UK-GDPR-Art-15
    identifiers_provided:
      - "NHS:1234567890"
    date_sent: 2026-07-10
    id_provided: passport copy
    ack_date: 2026-07-12
    due_date: 2026-08-10
    status: received
    outcome: 14 documents received
    filed_to:
      - documents/discharge-2019.pdf
    notes: null
    recorded_at: 2026-07-10T09:00:00Z
    recorded_by: patient
---
```

## Fields

- `id` - stable opaque GitEHR identifier, `ACQ-<timestamp>-<random>`.
- `controller` - the legal data controller the request was sent to. Often
  different from the treating site.
- `site` - the hospital/practice treated at, if different from the controller.
- `contact_used` - the email/portal/postal address the request actually went
  to (which may differ from the published DPO address).
- `care_context` - free text describing what this request is chasing.
- `right_invoked` - the legal right invoked, for example `UK-GDPR-Art-15`.
- `identifiers_provided` - repeatable free-text identifiers supplied with the
  request, for example `"NHS:1234567890"`.
- `date_sent` - date the request was sent, `YYYY-MM-DD`.
- `id_provided` - what identity evidence was attached to the request.
- `ack_date` - the date the controller acknowledged the request. Recorded for the audit trail; it does not affect `due_date`.
- `due_date` - the date a response is due, set by `gitehr acquisitions add`
  as one calendar month after `date_sent` and clamped to the length of the
  month (31 January yields 28 or 29 February). Under UK GDPR Article 12(3)
  the clock runs from the controller *receiving* the request, which a patient
  cannot observe, so `date_sent` is used as the earliest date it can have
  started - the right way to err for a register whose purpose is chasing.
  Acknowledgement does not restart the clock. A controller may extend by up
  to two further months for complex or numerous requests, or state a
  different date; `--due-date` records that instead.

- `status` - one of `drafted`, `sent`, `acknowledged`, `received`, `partial`,
  `nil-destroyed`, or `refused`. `nil-destroyed` and `refused` are first-class
  outcomes: a documented gap is itself a record, and silence is not the same
  as "nothing existed."
- `outcome` - free text describing what came back.
- `filed_to` - repeatable references (journal entry filenames or Document
  paths) recording where the result was filed. `update --filed-to` appends
  rather than replacing.
- `notes` - optional free text.
- `recorded_at` / `recorded_by` - GitEHR recording metadata.

## Commands

```bash
gitehr acquisitions list [--json] [--all] [--overdue]
gitehr acquisitions add --controller <name> --date-sent <YYYY-MM-DD> [OPTIONS]
gitehr acquisitions update <id> [OPTIONS]
```

`list` hides resolved acquisitions (`received`, `nil-destroyed`, `refused`) by
default. `--all` includes them. `--overdue` narrows to unresolved acquisitions
whose `due_date` has passed - the "everything overdue" view called for in the
provenance-and-acquisition proposal.

`add` writes `state/acquisitions.md`, stages it, writes a journal entry, and
commits both together. New acquisitions start with status `sent`.

`update` requires at least one of `--status`, `--ack-date`, `--due-date`,
`--outcome`, `--filed-to`, or `--notes`. It never deletes an acquisition entry;
it changes the current-state row and writes a journal entry, preserving the
prior status in Git history.

## Examples

```bash
gitehr acquisitions add --controller "York Teaching Hospitals NHS Trust" \
  --site "York Hospital" --contact dpo@york.nhs.uk \
  --context "2019 discharge summary" --identifier NHS:1234567890 \
  --date-sent 2026-07-10
gitehr acquisitions update ACQ-20260710T090000Z-4f2a9c1b \
  --status acknowledged --ack-date 2026-07-12
gitehr acquisitions list --overdue
gitehr acquisitions update ACQ-20260710T090000Z-4f2a9c1b \
  --status received --outcome "14 documents received" \
  --filed-to documents/discharge-2019.pdf
```
