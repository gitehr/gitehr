<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Calculator input definitions

## The problem

A clinical score input is not a value, it is a *predicate over the patient record*: "does this patient have vascular disease?", "is this patient a current smoker?", "is there acute disease with no nutritional intake for more than 5 days?". The score is only as correct as the clinician's (or the LLM's) answer to that predicate, and the predicates are routinely under-specified.

The canonical example is CHA2DS2-VASc. The "vascular disease" (V) criterion is satisfied by *arterial* disease - prior myocardial infarction, peripheral arterial disease, or complex aortic plaque - and is explicitly **not** satisfied by venous thromboembolism (DVT or PE). A clinician or an LLM that sees "history of PE" in the record and sets V = true produces a score that is one point too high, recommends anticoagulation it should not, and gives no error: the result is plausible, well-formatted, and wrong. This is the worst failure mode in clinical software, a silent miscalculation that looks authoritative.

This is not a quirk of one score. "Diabetes" in QRISK3 means a specific set of diagnoses and excludes others; "smoking" is banded (non / ex / light / moderate / heavy), not boolean; "congestive heart failure" in CHA2DS2-VASc means recent decompensation or moderate-to-severe LV dysfunction, not any historical mention. Every score with clinician-asserted criteria carries the same trap.

## Why this is unsolved, and why SNOMED alone does not fix it

The wider clinical informatics world has not solved input definition. Scores are published with prose criteria that are ambiguous or assume clinical context; different calculators (MDCalc, UpToDate, vendor EHRs) interpret them differently; and there is no governed, machine-readable, per-score statement of what counts.

SNOMED CT is necessary but not sufficient. SNOMED gives unambiguous *concept identifiers*, but a score input is not one concept, it is a *set* of concepts - a value set. SNOMED already has the machinery to express such a set:

- a **reference set (refset)**, an enumerated, governed list of concept ids; or
- an **ECL (Expression Constraint Language)** expression, an intensional query such as `<< 49601007 |Disorder of cardiovascular system|` minus the venous-thromboembolism subhierarchy.

The concepts exist and the query language exists. What does not exist, anywhere, is the *published, governed mapping from each score input to its concept set*. That mapping - including the exclusions - is the missing artifact. Building it is the innovation.

## The proposal: every input carries a citable, machine-readable definition

Each calculator input gains a **definition**: an authoritative, versioned, source-cited specification of exactly what makes it TRUE or FALSE. It is data, carried alongside the input in the calculator's JSON Schema, with this shape:

```json
{
  "concept": "Vascular disease",
  "statement": "Established arterial vascular disease.",
  "includes": [
    "Prior myocardial infarction",
    "Peripheral arterial disease",
    "Complex aortic plaque"
  ],
  "excludes": [
    "Venous thromboembolism (DVT or PE) does NOT count",
    "Isolated coronary artery disease without prior MI is disputed; see caveats"
  ],
  "source": {
    "citation": "Lip GYH et al. Refining clinical risk stratification (CHA2DS2-VASc). Chest. 2010;137(2):263-272.",
    "url": "https://doi.org/10.1378/chest.09-1584"
  },
  "snomedEcl": "<< 49601007 |Disorder of cardiovascular system (disorder)| MINUS << 118927008 |Disorder of venous system (disorder)|",
  "refset": null,
  "caveats": "Guidelines differ on whether stable CAD without MI qualifies.",
  "status": "draft"
}
```

The sharp end is the **`excludes`** field. It promotes the silent-failure cases - "VTE does not count" - from buried prose into a first-class, machine-readable datum. Most of the harm in clinical scoring comes from false-positive inclusions, and `excludes` is the field that names them.

`status` records governance maturity: `draft`, `reviewed` (checked by a second clinician against the cited source), or `endorsed` (matches a published guideline or an official refset). `snomedEcl` and `refset` are **advisory until reviewed** and must never silently drive a result while in `draft`.

## Delivery: one definition, every surface

The definition travels inside `input_schema()`, as a non-standard `definition` keyword on each property. JSON Schema validators ignore unknown keywords, so validation is unaffected, and the definition reaches every surface for free:

- **CLI** - `gitehr calc <name> --schema` already prints the definitions; a `gitehr calc <name> --define <field>` pretty-printer is a thin convenience on top. The fillable template marks fields that carry a definition.
- **MCP / LLM** - this is the decisive win. The model already receives `input_schema()` as the tool's `inputSchema`, so the `includes`, `excludes`, and `snomedEcl` are *in the tool contract the model reasons over*. An LLM mapping record data to score inputs sees "VTE does NOT count" at the point of decision, not in documentation it never reads.
- **Docs** - definition tables are generated from the schema, never hand-maintained.
- **Web UI** - per-input info popovers, which the Result Card spec already anticipates for clinical guidance.

## The governed registry: the genuinely novel artifact

Taken together, the definitions form an **open, versioned, peer-reviewed registry of clinical-score input definitions** - something the field lacks. Every definition cites a trusted source and carries a governance status; the SNOMED ECL expressions are reviewed and, where a governed refset does not yet exist, are candidates to become one. This is standards-track work: the reviewed ECL expressions can feed SNOMED International / NHS England refset processes, and the registry is publishable in its own right. It is licensed like the rest of GitEHR's clinical content (CC-BY-SA-4.0).

## The payoff: defensible LLM auto-population

Definitions are what make automatic input extraction *defensible* rather than merely plausible. With the `snomedEcl` and `excludes` present, an agent populating a score from a GitEHR record can:

1. Evaluate each input's ECL against the patient's coded data deterministically, or reason over the narrative with the exclusions explicitly in view.
2. Emit each TRUE/FALSE **with provenance**: "V = true (SCT 22298006 |Myocardial infarction| matches includes); a recorded PE was excluded per definition."
3. Surface low-confidence or disputed predicates (those whose definition `status` is `draft`, or whose record evidence is ambiguous) for human confirmation rather than guessing.

Every value in the score becomes traceable to a concept and a governed definition. This is the concrete mechanism behind "LLMs use the calculators - correctly".

## Staging

1. **Now** - establish the `definition` shape (this document) and embed definitions in the schemas of the calculators being built, starting with the simple cases (recall period and scoring anchors for the questionnaires, the PHQ-9 item-9 safety item). The delivery mechanism (schema -> `--schema` -> MCP -> docs) is proven by these.
2. **Next** - apply the full `includes` / `excludes` / `snomedEcl` treatment when the boolean-criteria scores land (CHA2DS2-VASc is the motivating case and the natural flagship; QRISK3 and HAS-BLED follow). These are where `excludes` and ECL earn their keep.
3. **Then** - the `--define` CLI pretty-printer, generated docs tables, the web popovers, and a deterministic ECL evaluator against a local SNOMED subset (reusing the `sct` MCP server already planned in `spec/DESIGN.md`).
4. **Later** - governance workflow (review to move `draft` -> `reviewed` -> `endorsed`), and submission of mature ECL expressions as candidate refsets.

## Open questions

- A typed `InputDefinition` Rust struct for authoring ergonomics, versus the current hand-written JSON in each schema. A builder would prevent shape drift across calculators once there are many.
- Where the deterministic ECL evaluator lives - in `calc-core` (which would break its leaf discipline by needing a terminology server) or in a separate extraction crate that depends on both `calc-core` and the `sct` server. The latter keeps `calc-core` pure.
- Whether to version definitions independently of the calculator (a definition can be corrected without the scoring logic changing).
- How to represent banded / non-boolean predicates (smoking) - an ordered set of definitions, one per band, rather than a single include/exclude pair.
