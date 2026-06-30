# gitehr vaccinations

Manage recorded vaccinations and immunisations in `state/vaccinations.md`.

This is typed state for current clinical summaries, GUI display, automation, and
future FHIR export. Mutations update the state file and create a journal entry.
Use `gitehr immunisations` or `gitehr immunizations` as aliases if that spelling
is more natural.

## gitehr vaccinations list

```bash
gitehr vaccinations list [--json] [--all]
```

Lists completed vaccination entries by default. Use `--all` to include entries
marked `entered-in-error`. Use `--json` for GUI/automation output.

## gitehr vaccinations add

```bash
gitehr vaccinations add --vaccine <name> --date <YYYY-MM-DD> [OPTIONS]
```

Records a vaccination.

Common options:

| Option | Description |
|---|---|
| `--vaccine <name>` | Vaccine or immunisation display name |
| `--date <YYYY-MM-DD>` | Administration date |
| `--dose-sequence <n>` | Dose sequence number |
| `--target-disease <text>` | Target disease; repeatable |
| `--site <text>` | Anatomical site, e.g. `left deltoid` |
| `--route <text>` | Administration route |
| `--product <text>` | Exact product administered |
| `--manufacturer <text>` | Manufacturer |
| `--batch-number <text>` | Batch or lot number |
| `--performer <text>` | Person or organisation administering |
| `--fhir-json <path>` | FHIR R4 `Immunization` JSON resource to embed |
| `--note <text>` | Optional journal narrative |

Example:

```bash
gitehr vaccinations add \
  --vaccine MMR \
  --date 2026-06-30 \
  --dose-sequence 1 \
  --target-disease measles \
  --target-disease mumps \
  --target-disease rubella \
  --site "left deltoid" \
  --route intramuscular \
  --product Priorix \
  --manufacturer GSK \
  --batch-number ABC123
```

## gitehr vaccinations entered-in-error

```bash
gitehr vaccinations entered-in-error <id> [--reason <text>]
```

Marks a vaccination entry as `entered-in-error` without deleting it. The entry is
hidden from the default list but remains visible with `--all` and remains in Git
history and the journal audit trail.
