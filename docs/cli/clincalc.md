# gitehr clincalc

!!! note "External plugin"
    `gitehr clincalc` is provided by the external `gitehr-clincalc` plugin. Install that executable on your `$PATH`; GitEHR discovers it automatically and forwards all following arguments.

Clinical calculators: scores, screeners, and risk tools. The same scoring engine drives the command line, the MCP server (for LLM use), the GUI, and the standalone web tools, so a result is identical wherever it is produced.

!!! note "One regular surface, no per-calculator flags"
    Every calculator is driven the same way: ask for a template, fill it in, pass it back. There are no calculator-specific flags to learn, and adding a calculator makes it available here and over MCP automatically.

## The shape

```text
gitehr clincalc list                       # list available calculators
gitehr clincalc <name>                     # print a fillable input TEMPLATE (JSON)
gitehr clincalc <name> --schema            # print the JSON Schema (the full contract)
gitehr clincalc <name> --license           # print the algorithm's distribution licence + evidence URL
gitehr clincalc <name> --input -           # compute, reading JSON from stdin
gitehr clincalc <name> --input data.json   # compute, reading JSON from a file
gitehr clincalc <name> --input '{...}'     # compute, reading an inline JSON string
gitehr clincalc <name> --input ... --format json   # machine-readable result
```

`gitehr clincalc` with no name (or `gitehr clincalc list`) prints the catalogue. Computing always requires an explicit `--input`, so a bare `gitehr clincalc <name>` is pure discovery and never waits on input.

## Discover, fill, compute

A bare `gitehr clincalc <name>` prints a template whose placeholders describe each expected value (type, allowed range, meaning). Its shape is exactly the input the calculator expects:

```console
$ gitehr clincalc feverpain
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
    | gitehr clincalc feverpain --input -
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
$ gitehr clincalc feverpain --input '{"fever":"yes"}'
Error: invalid input: invalid type: string "yes", expected a boolean
```

## The input contract

`gitehr clincalc <name> --schema` prints the JSON Schema for the inputs: types, required fields, enumerated values, and ranges. This is the authoritative contract, and the template above is generated from it (so the two cannot disagree).

The schema also carries, for inputs that have one, a **definition**: an authoritative, source-cited statement of exactly what makes the input TRUE or FALSE, including explicit exclusions (for example, that venous thromboembolism does not count as "vascular disease" in CHA2DS2-VASc). This guards against the silent-miscalculation trap, where a plausible but wrong input selection produces a wrong score with no error.

## Licence and provenance

Each calculator records the licence its clinical algorithm is distributed under, with a URL evidencing it (distinct from the AGPL-3.0 code licence). `gitehr clincalc <name> --license` prints both, and `gitehr clincalc list --format json` includes `license` and `license_source` for every calculator, so the basis on which each is shipped can be re-verified at any time.

```console
$ gitehr clincalc phq9 --license
{
  "license": "Public domain - released by Pfizer (2010); no permission required to reproduce, translate, display, or distribute",
  "source_url": "https://www.pfizer.com/news/press-release/press-release-detail/pfizer_to_offer_free_public_access_to_mental_health_assessment_tools_to_improve_diagnosis_and_patient_care"
}
```

## Available calculators

Run `gitehr clincalc list` for the current set (or `gitehr clincalc list --format json` for machine-readable output with each calculator's licence). The library covers the UK-focused 50-tool roadmap across five tiers - primary-care and NHS-mandated tools (QRISK3, PHQ-9, GAD-7, AUDIT, eGFR, FIB-4, ...), acute and emergency scores (NEWS2, CURB-65, Wells DVT/PE, CHA2DS2-VASc, HAS-BLED, qSOFA, ...), chronic-disease and specialist tools (DAS28, SOFA, HEART, MELD, Child-Pugh, ...), and PROMs and decision rules (CHALICE, Gleason, NPI, ...).

A handful of tools cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, ELF, ACQ, the Oxford Hip/Knee Scores, CAT, MUST, CFS, LANSS). These are still listed: running one returns an explanation of why it is absent, who owns it, open alternatives, and how to advocate for open clinical tools - see "Proprietary tools" below.

## Validation references

Each calculator ships its own authoritative citation, returned in the `reference` field of `--input ... --format json`, and re-checked at any time with `gitehr clincalc <name> --license`. For background reading, the table below lists the primary paper behind each tool family currently in the catalogue:

| Calculator | Primary reference |
|---|---|
| FeverPAIN | Little P, Stuart B, Thompson M, et al. Predictors of suppurative complications for acute sore throat in primary care: prospective clinical cohort study. *BMJ*. 2013;347:f6867. |
| QRISK3 | Hippisley-Cox J, Coupland C, Brindle P. Development and validation of QRISK3 risk prediction algorithms to estimate future risk of cardiovascular disease: prospective cohort study. *BMJ*. 2017;357:j2099. |
| PHQ-9 | Kroenke K, Spitzer RL, Williams JBW. The PHQ-9: validity of a brief depression severity measure. *J Gen Intern Med*. 2001;16(9):606-613. |
| GAD-7 | Spitzer RL, Kroenke K, Williams JBW, Löwe B. A brief measure for assessing generalized anxiety disorder: the GAD-7. *Arch Intern Med*. 2006;166(10):1092-1097. |
| AUDIT | Saunders JB, Aasland OG, Babor TF, de la Fuente JR, Grant M. Development of the Alcohol Use Disorders Identification Test (AUDIT). *Addiction*. 1993;88(6):791-804. |
| eGFR (CKD-EPI) | Levey AS, Stevens LA, Schmid CH, et al. A new equation to estimate glomerular filtration rate. *Ann Intern Med*. 2009;150(9):604-612. |
| FIB-4 | Sterling RK, Lissen E, Clumeck N, et al. Development of a simple noninvasive index to predict significant liver fibrosis in HIV/HCV-coinfected patients. *Hepatology*. 2006;43(6):1317-1325. |
| NEWS2 | Royal College of Physicians. National Early Warning Score (NEWS) 2. RCP, London, 2017. |
| CURB-65 | Lim WS, van der Eerden MM, Laing R, et al. Defining community acquired pneumonia severity on presentation to hospital. *Thorax*. 2003;58(5):377-382. |
| Wells (DVT/PE) | Wells PS, Anderson DR, Rodger M, et al. Evaluation of D-dimer in the diagnosis of suspected deep-vein thrombosis. *N Engl J Med*. 2003;349(13):1227-1235. |
| CHA2DS2-VASc | Lip GYH, Nieuwlaat R, Pisters R, Lane DA, Crijns HJGM. Refining clinical risk stratification for predicting stroke and thromboembolism in atrial fibrillation. *Chest*. 2010;137(2):263-272. |
| HAS-BLED | Pisters R, Lane DA, Nieuwlaat R, de Vos CB, Crijns HJGM, Lip GYH. A novel user-friendly score (HAS-BLED) to assess 1-year risk of major bleeding. *Chest*. 2010;138(5):1093-1100. |
| qSOFA | Seymour CW, Liu VX, Iwashyna TJ, et al. Assessment of clinical criteria for sepsis (Sepsis-3). *JAMA*. 2016;315(8):762-774. |
| DAS28 | Prevoo MLL, van 't Hof MA, Kuper HH, van Leeuwen MA, van de Putte LBA, van Riel PLCM. Modified disease activity scores that include twenty-eight-joint counts. *Arthritis Rheum*. 1995;38(1):44-48. |
| SOFA | Vincent JL, Moreno R, Takala J, et al. The SOFA score to describe organ dysfunction/failure. *Intensive Care Med*. 1996;22(7):707-710. |
| HEART score | Six AJ, Backus BE, Kelder JC. Chest pain in the emergency room: value of the HEART score. *Neth Heart J*. 2008;16(6):191-196. |
| MELD | Kamath PS, Wiesner RH, Malinchoc M, et al. A model to predict survival in patients with end-stage liver disease. *Hepatology*. 2001;33(2):464-470. |
| Child-Pugh | Pugh RNH, Murray-Lyon IM, Dawson JL, Pietroni MC, Williams R. Transection of the oesophagus for bleeding oesophageal varices. *Br J Surg*. 1973;60(8):646-649. |
| CHALICE | Dunning J, Daly JP, Lomas JP, Lecky F, Batchelor J, Mackway-Jones K. Derivation of the CHALICE decision rule for head injury in children. *Arch Dis Child*. 2006;91(11):885-891. |
| Gleason | Gleason DF, Mellinger GT. Prediction of prognosis for prostatic adenocarcinoma by combined histological grading and clinical staging. *J Urol*. 1974;111(1):58-64. |
| Nottingham Prognostic Index | Galea MH, Blamey RW, Elston CE, Ellis IO. The Nottingham Prognostic Index in primary breast cancer. *Breast Cancer Res Treat*. 1992;22(3):207-219. |

These are starting points for clinical review, not a substitute for the in-tool `reference` field or a calculator's own `--schema` definitions, which are the exact versions GitEHR ships and tests against.

## Proprietary tools

Some clinical tools are owned and licence-controlled by their authors and cannot be distributed in open-source software. Rather than omit them silently, GitEHR registers each as a calculator that returns a structured explanation instead of a score:

```console
$ gitehr clincalc frax --input '{}'
frax = unavailable: proprietary

FRAX (10-year fracture risk) is not available in GitEHR because it is proprietary
or licence-locked. Owner: University of Sheffield ... Open alternatives: qfracture ...
```

The response names the owner, the reason, open alternatives (often one GitEHR already ships - e.g. QFracture for FRAX, AMTS for MMSE, FIB-4 for ELF), and advice to advocate for open clinical tools.

## Use from an LLM

The MCP server exposes each calculator as a tool named `clincalc_<name>` whose input schema is the calculator's own JSON Schema, so a model receives a typed input contract (including any input definitions) rather than scraping help text. See [MCP usage](mcp-usage.md). The CLI and MCP surfaces share one engine and one schema: discover the schema, supply the JSON, receive the result.

## Standalone `clincalc` binary

The calculators also ship as a small, dependency-light standalone binary with the same interface, for use without a GitEHR repository:

```console
$ cargo install --git https://github.com/pacharanero/clincalc clincalc
$ clincalc phq9 --input '{"responses":[2,2,1,1,1,0,1,0,0]}' --format json
```

!!! warning "Clinical safety"
    Calculators support clinical decisions; they do not replace clinical judgement. Each cites primary literature and is tested against published vectors, and results are interpretations, not diagnoses. Confirm input definitions against the cited source before acting on a result.
