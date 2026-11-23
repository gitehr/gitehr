# gitehr family-history

Manage recorded family history in `state/family-history.md`: conditions in blood
relatives, with the relative's age at onset where known.

This is typed state for clinical summaries, risk calculators such as QRISK3 and
FRAX, GUI display, automation, and future FHIR export. Each entry is one
condition in one relative; record a relative with two conditions as two entries.
Mutations update the state file and create a journal entry in the same isolated
commit, leaving unrelated staged work untouched. The command refuses to
overwrite a state file with uncommitted changes, and restores the previous state
if the commit fails.

The relative's name is not recorded.

## gitehr family-history list

```bash
gitehr family-history list [--json] [--all]
```

Lists family-history entries by default. Use `--all` to include entries marked
`entered-in-error`. Use `--json` for GUI/automation output.

## gitehr family-history add

```bash
gitehr family-history add --relationship <text> --condition <text> [OPTIONS]
```

Records a condition in a relative.

| Option | Description |
|---|---|
| `--relationship <text>` | Relationship to the patient, e.g. `mother`, `brother`, `maternal grandmother` |
| `--condition <text>` | Condition display name |
| `--onset-age <years>` | Relative's age in whole years when the condition began (`0`-`130`) |
| `--contributed-to-death` | The condition contributed to the relative's death |
| `--fhir-json <path>` | FHIR R4 `FamilyMemberHistory` JSON resource to embed |
| `--note <text>` | Optional journal narrative |
| `--source-type <type>` | Provenance: `self-reported`, `clinician-asserted`, `portal-extracted`, `sar`, `paper-transcribed`, `device`, or `inferred` |
| `--source-detail <text>` | Provenance: controller, portal, device or citation |
| `--acquired-via <id>` | Provenance: acquisition id (see `gitehr acquisitions`) |
| `--document-ref <path>` | Provenance: regular-file or manifest-backed directory Document under `documents/` or `imaging/`; its SHA-256 is recorded |
| `--evidence-level <level>` | Provenance: `documented`, `inferred`, or `assumed` |
| `--confidence <level>` | Provenance: `high`, `medium`, or `low` |

`--relationship` and `--condition` must not be blank. `--source-type` is required
when any other provenance option is given. The `provenance` block is omitted
when none is supplied.

Example:

```bash
gitehr family-history add \
  --relationship father \
  --condition "Myocardial infarction" \
  --onset-age 54 \
  --source-type self-reported
```

## gitehr family-history entered-in-error

```bash
gitehr family-history entered-in-error <id> [--reason <text>]
```

Marks an entry as `entered-in-error` without deleting it. The entry is hidden
from the default list but remains visible with `--all` and remains in Git
history and the journal audit trail.
