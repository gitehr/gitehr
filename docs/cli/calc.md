# gitehr calc

!!! warning "Temporarily dormant"
    `gitehr calc` is temporarily disabled while the shared calculator crates in `pacharanero/calc` are prepared for crates.io. This lets GitEHR use release-plz, which runs Cargo package verification and cannot package GitEHR while it has git-only calculator dependencies. The command and MCP calculator tools should return once `calc-cli` and `calc-core` are published.

Clinical calculators: scores, screeners, and risk tools. The same scoring engine drives the command line, the MCP server (for LLM use), the GUI, and the standalone web tools, so a result is identical wherever it is produced.

!!! note "One regular surface, no per-calculator flags"
    Every calculator is driven the same way: ask for a template, fill it in, pass it back. There are no calculator-specific flags to learn, and adding a calculator makes it available here and over MCP automatically.

## The shape

```text
gitehr calc list                       # list available calculators
gitehr calc <name>                     # print a fillable input TEMPLATE (JSON)
gitehr calc <name> --schema            # print the JSON Schema (the full contract)
gitehr calc <name> --license           # print the algorithm's distribution licence + evidence URL
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

## Licence and provenance

Each calculator records the licence its clinical algorithm is distributed under, with a URL evidencing it (distinct from the AGPL-3.0 code licence). `gitehr calc <name> --license` prints both, and `gitehr calc list --format json` includes `license` and `license_source` for every calculator, so the basis on which each is shipped can be re-verified at any time.

```console
$ gitehr calc phq9 --license
{
  "license": "Public domain - released by Pfizer (2010); no permission required to reproduce, translate, display, or distribute",
  "source_url": "https://www.pfizer.com/news/press-release/press-release-detail/pfizer_to_offer_free_public_access_to_mental_health_assessment_tools_to_improve_diagnosis_and_patient_care"
}
```

## Available calculators

Run `gitehr calc list` for the current set (or `gitehr calc list --format json` for machine-readable output with each calculator's licence). The library covers the UK-focused 50-tool roadmap across five tiers - primary-care and NHS-mandated tools (QRISK3, PHQ-9, GAD-7, AUDIT, eGFR, FIB-4, ...), acute and emergency scores (NEWS2, CURB-65, Wells DVT/PE, CHA2DS2-VASc, HAS-BLED, qSOFA, ...), chronic-disease and specialist tools (DAS28, SOFA, HEART, MELD, Child-Pugh, ...), and PROMs and decision rules (CHALICE, Gleason, NPI, ...).

A handful of tools cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, ELF, ACQ, the Oxford Hip/Knee Scores, CAT, MUST, CFS, LANSS). These are still listed: running one returns an explanation of why it is absent, who owns it, open alternatives, and how to advocate for open clinical tools - see "Proprietary tools" below.

## Proprietary tools

Some clinical tools are owned and licence-controlled by their authors and cannot be distributed in open-source software. Rather than omit them silently, GitEHR registers each as a calculator that returns a structured explanation instead of a score:

```console
$ gitehr calc frax --input '{}'
frax = unavailable: proprietary

FRAX (10-year fracture risk) is not available in GitEHR because it is proprietary
or licence-locked. Owner: University of Sheffield ... Open alternatives: qfracture ...
```

The response names the owner, the reason, open alternatives (often one GitEHR already ships - e.g. QFracture for FRAX, AMTS for MMSE, FIB-4 for ELF), and advice to advocate for open clinical tools.

## Use from an LLM

The MCP server exposes each calculator as a tool named `calc_<name>` whose input schema is the calculator's own JSON Schema, so a model receives a typed input contract (including any input definitions) rather than scraping help text. See [MCP usage](mcp-usage.md). The CLI and MCP surfaces share one engine and one schema: discover the schema, supply the JSON, receive the result.

## Standalone `calc` binary

The calculators also ship as a small, dependency-light standalone binary with the same interface, for use without a GitEHR repository:

```console
$ cargo install --git https://github.com/pacharanero/calc calc-cli
$ calc phq9 --input '{"responses":[2,2,1,1,1,0,1,0,0]}' --format json
```

!!! warning "Clinical safety"
    Calculators support clinical decisions; they do not replace clinical judgement. Each cites primary literature and is tested against published vectors, and results are interpretations, not diagnoses. Confirm input definitions against the cited source before acting on a result.
