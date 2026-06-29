# Problem / Condition list

*Status: proposal (draft). Surfaced from maintaining a real personal health record in parallel with GitEHR (see note at the end). Relates to #10 (base information model) and to the existing `state/` typed-state pattern (`allergies`, `demographics`).*

## Why this is the priority gap

GitEHR ships typed-state for allergies and demographics, an immutable journal, documents/imaging, the store/MPI model, import, MCP, and a calculator library in waiting. The single most important *clinical* primitive of any health record - "what health states does this person have, and which are a current concern" - has no structured home. A `Condition` type is sketched in `DESIGN.md`, but it is not shipped: there is no `state/conditions.md` in the folder template, no command, and no lifecycle. A problem today exists only as free text inside a journal entry.

That blocks three things at once:

- A usable summary. The problem list is the first thing any clinician reads; without it there is no current-state clinical view.
- The calculators. `pacharanero/calc` is built and validated, but many scores are condition-gated (current diagnoses, treated-hypertension status, etc.). No structured conditions, no automatic scores.
- FHIR / openEHR export. There are no `Condition` / `EVALUATION` instances to serialize.

## Condition vs Problem - a load-bearing distinction

These are not synonyms, and the difference is worth modelling explicitly:

- A **condition** is any recorded health state - symptomatic or not, concerning or not. A vasectomy, an asymptomatic incidental finding, a quiescent old disc lesion: all conditions.
- A **problem** is a condition that is, or might be, a current concern. The word "problem" deliberately carries an implication of concern.

So every problem is a condition; not every condition is a problem. The "problem list" is the concern-filtered view over the set of conditions. This mirrors FHIR, which deliberately named the resource `Condition` (broader than "problem", explicitly spanning incidental/asymptomatic findings) and carries a `category` of `problem-list-item` vs `encounter-diagnosis`; and openEHR, which separates the Problem/Diagnosis concern from the broader recorded state.

Recommendation: model **Condition** as the primitive; "problem list" is a filtered projection (`category = problem-list-item` and active), not a separate store.

## Proposed model

One record per condition, following the shipped `allergies` precedent exactly: typed-state entries, with a journal entry written on every state change (the immutable audit trail in `journal/`, the current-state view overwrite-in-place in `state/`). This honours ADR-0002 (the journal only grows) while keeping a clean "true now" surface.

Two representations are possible; pick per the project's existing pattern:

- **(A) Single `state/conditions.md`** with a YAML array, mirroring `state/allergies.md` exactly. Simplest, matches the shipped pattern. Recommended for v1.
- **(B) One file per condition** under `state/conditions/<slug>.md`, with folder location encoding lifecycle (active vs inactive). Richer per-condition narrative, cleaner git diffs, closer to one-FHIR-resource-per-file. Worth considering for v2 once conditions carry long narratives. (This is what the parallel real record uses.)

### Fields (mapped to FHIR Condition)

- `id` - stable, opaque (GitEHR already mints ids like `ALG-<ts>-<rand>` for allergies; reuse that scheme). See identity below.
- `name` - human-readable display (mutable).
- `clinical_status` - active | recurrence | relapse | inactive | remission | resolved (FHIR `Condition.clinicalStatus`).
- `verification_status` - unconfirmed | provisional | differential | confirmed | refuted | entered-in-error (FHIR `Condition.verificationStatus`). This captures the working-diagnosis-vs-confirmed need directly, and gives "entered in error" a home without deletion.
- `category` - problem-list-item | encounter-diagnosis (the problem-vs-condition switch).
- `onset` - ISO date / year / approximate / "childhood".
- `abatement` - ISO date if resolved, else null.
- `recorded_at`, `recorded_by`.
- `body_site` (+ `laterality`) - see below.
- `code` - SNOMED CT primarily; optional ICD-10. (Terminology binding is a wider gap; SNOMED is already designed for the GUI lookup.)
- `severity` - optional.
- `related` - episodes (journal-entry / document refs), medications, other conditions.
- `provenance` - see the companion proposal `record-provenance-and-acquisition.md` (source, evidence-level, acquired-via).

### Lifecycle and immutability

State change = update the current-state entry + write a journal entry. This is exactly how `allergies inactive` already works. Corrections supersede, they never erase: an `entered-in-error` is a `verification_status` flip plus a journal entry, not a deletion. A reader must always be able to reconstruct what was believed at any past date from the journal.

### Identity and slug rot (a hard-won lesson)

Conditions evolve as understanding changes. A lesion recorded as a "haemangioma" turns out histologically to be an angiokeratoma; a one-off "metallic taste" later becomes the first datapoint of a chronic symptom that belongs under a different entry. A descriptive slug is therefore a commitment to *current* understanding, and current understanding moves - so descriptive slugs rot, and renaming breaks links.

Recommendation: a stable opaque `id` as the primary handle (as allergies already do), with the human-readable `name` in a mutable field. If using representation (B), keep the *filename slug stable* even when the `name` updates - so the filename records what it was first thought to be, the `name` records what it is now - and change lifecycle by moving the file between active/inactive, never by renaming.

### Body site / laterality

Conditions, findings and procedures need an anatomical location and laterality. Rather than ship a separate half-feature, attach it here: a `body_site` (free text + optional SNOMED body-structure code) plus a `laterality` enum (left | right | bilateral | midline). Most clinical use of body-site is condition- or finding-attached, so this is its natural home.

### Relationship to Medical Markdown (ADR-0004)

Two ingestion routes that are complementary, not in conflict:

- **Direct authoring:** `gitehr conditions add ...` (the typed-state path, like `allergies add`). This is v1.
- **Derived:** a journal entry written in Medical Markdown with an `IMP/` (impression / diagnosis) marker projects into the conditions state (Medical Markdown phase 3, per the roadmap).

Both converge on the same `state/conditions.*`. The typed command does not contradict ADR-0004: ADR-0004 governs structure *derived from narrative*; directly-authored typed state (the allergies/demographics mechanism) is the established second route and already shipped.

## Proposed CLI (mirrors `gitehr allergies`)

```
gitehr conditions list [--all] [--problems] [--json]
gitehr conditions add --name <text> [--status <s>] [--verification <v>] [--category <c>]
                      [--onset <date>] [--code snomed:<id>] [--body-site <text>] [--laterality <l>]
                      [--severity <s>] [--note <text>]
gitehr conditions resolve <id> [--date <date>] [--reason <text>]   # → inactive, like `allergies inactive`
gitehr conditions show <id>
```

`gitehr conditions list --problems` is the problem-list view (active + `category = problem-list-item`).

## Respecting existing decisions

- Not a query engine (explicitly rejected): "problem list" is a simple filtered projection, not SQL.
- Complementary to #10 (base information model): this is the concrete `Condition` primitive that sits on top of that base.
- The cross-cutting "problems dashboard" view belongs with the derived-projections proposal, not here.

## Note on provenance of this proposal

Drafted from running a parallel real personal record whose condition model hit exactly these edges: the condition-vs-problem distinction, slug rot, working-vs-confirmed diagnosis, body-site/laterality, and absorbing mid-stream corrections without losing prior state. The field list above is the one that proved necessary in practice, not a speculative superset. The anonymised, battle-tested frontmatter schema can be shared if useful.
