# gitehr calc

Clinical calculators: scores, screeners, and risk tools. The same scoring engine drives the command line, the MCP server (for LLM use), the GUI, and the standalone web tools, so a result is identical wherever it is produced.

!!! note "One regular surface, no per-calculator flags"
    Every calculator is driven the same way: ask for a template, fill it in, pass it back. There are no calculator-specific flags to learn, and adding a calculator makes it available here and over MCP automatically.

## The shape

```text
gitehr calc list                       # list available calculators
gitehr calc <name>                     # print a fillable input TEMPLATE (JSON)
gitehr calc <name> --schema            # print the JSON Schema (the full contract)
gitehr calc <name> --input -           # compute, reading JSON from stdin
gitehr calc <name> --input data.json   # compute, reading JSON from a file
gitehr calc <name> --input '{...}'     # compute, reading an inline JSON string
gitehr calc <name> --input ... --format json   # machine-readable result
```

`gitehr calc` with no name (or `gitehr calc list`) prints the catalogue. Computing always requires an explicit `--input`, so a bare `gitehr calc <name>` is pure discovery and never waits on input.

## Discover, fill, compute

A bare `gitehr calc <name>` prints a template whose placeholders describe each expected value (type, allowed range, meaning). Its shape is exactly the input the calculator expects:

```console
$ gitehr calc feverpain
{
  "fever": "<boolean> Fever in the last 24 hours",
  "purulence": "<boolean> Purulence (pus on the tonsils)",
  "attend_rapidly": "<boolean> Symptom onset within 3 days (<= 3 days)",
  "inflamed_tonsils": "<boolean> Severely inflamed tonsils",
  "absence_of_cough": "<boolean> No cough or coryza"
}
```

Replace each placeholder with a real value and pass it back:

```console
$ echo '{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}' \
    | gitehr calc feverpain --input -
feverpain = 3

A score of 3 is associated with 34-40% isolation of streptococcus. A delayed
prescribing strategy is appropriate after discussion with the patient.
...
```

The template, schema, and computed result are printed as JSON on **stdout**; hints and usage go to **stderr**, so output stays clean when piped.

## Output

`--format json` prints the canonical result object, identical to what the MCP server and the web tools return:

```json
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "A score of 3 is associated with 34-40% isolation of streptococcus ...",
  "working": { "score": 3, "level": "delayed", "...": "..." },
  "reference": "Little P, Stuart B, Hobbs FDR, et al. Lancet Infect Dis. 2014. ..."
}
```

| Field | Meaning |
|---|---|
| `calculator` | Machine name of the calculator |
| `result` | The primary computed value (a number or short string) |
| `interpretation` | Human-readable clinical interpretation |
| `working` | Step-by-step breakdown of how the result was reached |
| `reference` | Primary citation / guideline |

Invalid input is rejected by the calculator's own typed validation, with a clear message and a non-zero exit code:

```console
$ gitehr calc feverpain --input '{"fever":"yes"}'
Error: invalid input: invalid type: string "yes", expected a boolean
```

## The input contract

`gitehr calc <name> --schema` prints the JSON Schema for the inputs: types, required fields, enumerated values, and ranges. This is the authoritative contract, and the template above is generated from it (so the two cannot disagree).

The schema also carries, for inputs that have one, a **definition**: an authoritative, source-cited statement of exactly what makes the input TRUE or FALSE, including explicit exclusions (for example, that venous thromboembolism does not count as "vascular disease" in CHA2DS2-VASc). This guards against the silent-miscalculation trap, where a plausible but wrong input selection produces a wrong score with no error.

## Available calculators

Run `gitehr calc list` for the current set. The library is being built out in clinical-priority order; today it includes the reference implementations:

| Name | Title |
|---|---|
| `feverpain` | FeverPAIN Score (acute sore throat, antibiotic stewardship) |
| `asrs` | ASRS-v1.1 Adult ADHD Screener |
| `phq9` | PHQ-9 Depression Severity |
| `gad7` | GAD-7 Anxiety Severity |

## Use from an LLM

The MCP server exposes each calculator as a tool named `calc_<name>` whose input schema is the calculator's own JSON Schema, so a model receives a typed input contract (including any input definitions) rather than scraping help text. See [MCP usage](mcp-usage.md). The CLI and MCP surfaces share one engine and one schema: discover the schema, supply the JSON, receive the result.

## Standalone `calc` binary

The calculators also ship as a small, dependency-light standalone binary with the same interface, for use without a GitEHR repository:

```console
$ cargo install --git https://github.com/gitehr/gitehr -p calc-cli
$ calc phq9 --input '{"responses":[2,2,1,1,1,0,1,0,0]}' --format json
```

!!! warning "Clinical safety"
    Calculators support clinical decisions; they do not replace clinical judgement. Each cites primary literature and is tested against published vectors, and results are interpretations, not diagnoses. Confirm input definitions against the cited source before acting on a result.
