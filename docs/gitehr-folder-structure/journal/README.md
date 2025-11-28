# Journal (Initial thoughts)

The journal is an append-only logbook of clinical interactions organised per patient.

## Layout

```
journal/
  patients/
    <patient_id>/
      <ISO8601-event-timestamp>--<event_id>.md
```

- `patient_id` should be a stable GitEHR identifier (for example `pat-1234`).
- `ISO8601-event-timestamp` must represent when the interaction happened (UTC), e.g. `2025-11-28T12-34-56Z`.
- `event_id` is a short GUID/slug (`c01`, `evt-uuid`) to guarantee uniqueness.

## File format

Each file describes one interaction and uses **YAML front matter + Markdown body**:
Need to strongly consider the cons of using markdown through out

Proposed yaml header structure:
```yaml
---
gitehr_event_id: c01
gitehr_patient_id: pat-1234
recorded_at: 2025-11-28T12:34:56Z
author:
  id: user-dr-osutuk
  name: "Akan Osutuk"
  role: "ST2 Paediatrics"
context:
  location: "Ward 3B"
  encounter_id: "enc-987"
  episode_id: "ep-ALL-block1"
links:
  related_imaging:
    - "imaging/pat-1234/studies/study-abc/meta.json"
  related_files:
    - "attachments/pat-1234/letters/2025-11-28-discharge.pdf"
codes:
  diagnoses:
    - system: "snomed"
      code: "123456"
      display: "Acute lymphoblastic leukaemia"
  procedures: []
correction_of: null
---
Markdown narrative of the clinical interaction...
```

- YAML captures machine-readable metadata and references.
- The Markdown body is the human narrative, free-form but diffable.


