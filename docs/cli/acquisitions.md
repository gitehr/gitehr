# gitehr acquisitions

Track the process of acquiring parts of your record from the organisations that
hold it - Subject Access Requests (SARs), portal pulls, and paper requests -
in `state/acquisitions.md`.

This is typed state for the unglamorous but essential companion to record
extraction: most of a record is not extracted by a clever agent, it is
requested from a data controller and waited for. This is the on-ramp that
populates the record; the repository stays empty until you acquire. See
[`spec/record-provenance-and-acquisition.md`](https://github.com/gitehr/gitehr/blob/main/spec/record-provenance-and-acquisition.md)
for the background (Part 2 - record acquisition workflow). Mutations update
the state file and create a journal entry in the same commit, same as
`gitehr allergies` and `gitehr conditions`.

## gitehr acquisitions list

```bash
gitehr acquisitions list [--json] [--all] [--overdue]
```

Lists unresolved acquisitions by default (not yet `received`, `nil-destroyed`,
or `refused`). Use `--all` to include resolved ones too. Use `--overdue` to
show only unresolved acquisitions whose `due_date` has passed. Use `--json`
for GUI/automation output.

## gitehr acquisitions add

```bash
gitehr acquisitions add --controller <name> --date-sent <YYYY-MM-DD> [OPTIONS]
```

Records a request as sent, and sets `due_date` to one calendar month after `--date-sent`, so a request is chaseable from the moment it is recorded rather than only once a controller replies.

Under UK GDPR Article 12(3) the deadline runs from the controller *receiving* the request. A patient cannot observe that, so the date sent is used as the earliest date the clock can have started. A controller may extend by up to two further months for complex or numerous requests, or state a different date when acknowledging - record that with `update --due-date`.

| Option | Description |
|---|---|
| `--controller <name>` | Legal data controller the request was sent to (often not the treating site) |
| `--date-sent <date>` | Date the request was sent, `YYYY-MM-DD` |
| `--site <name>` | Treating site, if different from the controller |
| `--contact <address>` | Email/portal/postal address actually used |
| `--context <text>` | What this request is chasing |
| `--right <right>` | Right invoked (default `UK-GDPR-Art-15`) |
| `--identifier <value>` | Identifier provided, e.g. `NHS:1234567890`; repeatable |
| `--id-provided <text>` | What ID was attached to the request |
| `--notes <text>` | Optional note |

New acquisitions start with status `sent`.

## gitehr acquisitions update

```bash
gitehr acquisitions update <id> [OPTIONS]
```

Updates the status and details of an existing acquisition. At least one of the
following must be given:

| Option | Description |
|---|---|
| `--status <status>` | `drafted`, `sent`, `acknowledged`, `received`, `partial`, `nil-destroyed`, or `refused` |
| `--ack-date <date>` | Date the controller acknowledged the request, `YYYY-MM-DD`. Recorded only; it does not move the due date |
| `--due-date <date>` | Response due date, `YYYY-MM-DD`. Overrides the date computed when the request was recorded |
| `--outcome <text>` | What came back |
| `--filed-to <ref>` | Journal entry or document reference the result was filed to; repeatable, appends |
| `--notes <text>` | Optional note |

## gitehr acquisitions letter

```bash
gitehr acquisitions letter <id> [--requester-name <name>] [--requester-address <address>] [--out <path>]
```

Renders a UK GDPR Article 15 (Data Protection Act 2018 section 45) subject
access request letter from an already-recorded acquisition, filling in the
controller, site, contact, requested context, identifiers, and identity
evidence the register already holds. It reads state only - no journal entry
is written and nothing is committed.

The requester's name and address are not modelled anywhere else in GitEHR
(that is patient-identity data, not acquisition-tracking data), so they are
left as bracketed placeholders - `[your name]` / `[your return address]` -
unless supplied with `--requester-name`/`--requester-address`. This is a
template to review and complete, not a letter to send unedited.

| Option | Description |
|---|---|
| `--requester-name <name>` | Name to sign the letter with |
| `--requester-address <address>` | Return address for the letterhead |
| `--out <path>` | Also write the rendered letter to this file |

The letter always prints to stdout; `--out` additionally saves a copy.

## Example

```bash
gitehr acquisitions add --controller "York Teaching Hospitals NHS Trust" \
  --site "York Hospital" --contact dpo@york.nhs.uk \
  --context "2019 discharge summary" --identifier NHS:1234567890 \
  --date-sent 2026-07-10
gitehr acquisitions letter ACQ-20260710T090000Z-4f2a9c1b \
  --requester-name "Jo Patient" --requester-address "1 Example Street, Leeds" \
  --out sar-letter.txt
gitehr acquisitions update ACQ-20260710T090000Z-4f2a9c1b \
  --status acknowledged --ack-date 2026-07-12
gitehr acquisitions update ACQ-20260710T090000Z-4f2a9c1b \
  --status received --outcome "14 documents received" \
  --filed-to documents/discharge-2019.pdf
gitehr acquisitions list --overdue
```
