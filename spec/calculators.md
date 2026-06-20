<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Clinical Calculators in GitEHR

## Goal

Provide a comprehensive, open-source library of clinical calculators with one canonical scoring engine driving every surface: the command line, an MCP server, the GitEHR desktop GUI, standalone single-file web tools, and a separately-distributable desktop/mobile app. Calculations are evidence-based, auditable, and - when run inside GitEHR - recorded automatically into the patient's version-controlled record with both inputs and results.

An EHR that ships a native, offline, auditable calculator suite driven by a single engine is something the "Big EHR" platforms structurally cannot produce, because it depends on a small-sharp-core architecture rather than a monolith. That is the advantage this spec is built to capture.

## Philosophy

### Open and free

- **Open source** - anyone can view, use, modify, and share the code (AGPL-3.0-or-later, matching GitEHR; clinical content under CC-BY-SA-4.0).
- **Free to use** - no paywalls, no licences, no restrictions.
- **Auditable** - scoring logic is pure and trivially readable; every calculator cites primary literature and is tested against known vectors.

### Soft interoperability

'Soft' interoperability is copy-and-paste interop. It empowers clinicians to use the tools they want without being constrained by their EHR, and lets them exercise their own judgement about whether to reach for a given calculator. Copy-and-paste is a common clinician workaround for the deficiencies of EHRs and is often derided as a kludge, but until real interoperability arrives we should embrace and optimise for the tools clinicians actually use. Every calculator therefore produces a clean, editable text summary for the clipboard as a first-class output, in addition to structured dispatch when embedded.

---

## Architecture: one core, many surfaces

The defining decision is a single scoring engine reused everywhere, so a result produced at the command line, in the browser, in the GUI, or via MCP is identical by construction. The dependency arrows all point **into** the core; the core never depends on anything above it.

```
                         ┌───────────────────────────┐
                         │   calc-core (leaf crate)   │
                         │  scoring logic + schema    │
                         │  deps: serde, serde_json   │
                         │  NO gitehr, NO async        │
                         └─────────────┬──────────────┘
                                       │ (every arrow points in)
        ┌───────────────┬─────────────┼──────────────┬──────────────────┐
        │               │             │              │                  │
   ┌─────────┐   ┌────────────┐  ┌──────────┐  ┌────────────┐   ┌───────────────┐
   │ calc-cli│   │ gitehr-mcp │  │ gitehr   │  │ standalone │   │  calc-web      │
   │ (lib+bin)│  │ (MCP tools)│  │ gui      │  │ calc app   │   │  single-file   │
   │  `calc`  │  │            │  │ (Tauri)  │  │ (Tauri 2,  │   │  HTML + bridge │
   │          │  │            │  │          │  │  desktop/  │   │  (GitHub Pages)│
   │          │  │            │  │          │  │  mobile)   │   │                │
   └────┬─────┘  └────────────┘  └──────────┘  └────────────┘   └───────────────┘
        │ reused verbatim
   ┌────┴───────────┐
   │ gitehr calc    │
   │ (subcommand)   │
   └────────────────┘
```

### Workspace layout (as built)

```
gitehr/                              # repo root = Cargo workspace root
├── Cargo.toml                       # members: cli, mcp, calc-core, calc-cli
├── cli/                             # the `gitehr` binary (will gain `gitehr calc`)
├── mcp/                             # gitehr-mcp server
├── calc-core/                       # the engine — leaf crate
│   ├── Cargo.toml                   #   deps: serde, serde_json only
│   └── src/
│       ├── lib.rs                   #   registry: all() / get(name)
│       ├── response.rs              #   CalculationResponse schema
│       ├── calculator.rs            #   Calculator trait + CalcError
│       └── calculators/
│           ├── mod.rs
│           ├── feverpain.rs
│           └── asrs.rs
├── calc-cli/                        # CLI surface — lib + bin
│   ├── Cargo.toml                   #   [[bin]] name = "calc"; [lib] name = "calc_cli"
│   └── src/
│       ├── lib.rs                   #   CalcCommand + run() reused by `gitehr calc`
│       └── main.rs                  #   thin standalone wrapper
├── calc-web/                        # single-file HTML calculators (frontend)
│   ├── index.html                   #   gallery / landing page
│   ├── calculators/<name>.html      #   one self-contained file per calculator
│   ├── shared/
│   │   ├── gitehr-bridge.js         #   context detection + result dispatch
│   │   └── styles.css               #   shared brand styles
│   └── clinical-source-references/  #   authoritative source material per calculator
├── gui/                             # Tauri desktop app (excluded from lib workspace)
└── skills/build-calculator/         # authoring skill for new calculators
```

### `calc-core` - the leaf engine

The single source of truth. Pure, deterministic scoring with no clock, no I/O, and no global state; a host that needs a timestamp stamps it when recording. It depends only on `serde` and `serde_json` - never on the rest of GitEHR and never on an async runtime. That leaf discipline is what makes the calculators detachable (see Distribution below).

Every calculator implements the `Calculator` trait and also exposes a strongly-typed `Input`/`compute` pair plus a `build_response` adapter. The crate-level registry (`all()` / `get(name)`) is the one list the CLI, MCP server, and GUI all enumerate, so adding a calculator surfaces it everywhere.

### `calc-cli` - the CLI surface (lib + bin)

All CLI behaviour lives in the library (`CalcCommand` + `run()`), so there is nothing to re-implement when embedding it. It ships two ways:

1. The standalone `calc` binary - `cargo install --git <repo> -p calc-cli` installs a small, dependency-light tool (tree: `anyhow`, `serde`/`serde_json`, `clap` - no tokio, no EHR).
2. The `gitehr calc` subcommand - the gitehr CLI depends on `calc-cli` and forwards to `calc_cli::run`, repeating nothing:

```rust
// cli/src/main.rs (planned)
#[derive(clap::Subcommand)]
enum Commands {
    // ...existing commands
    /// Clinical calculators
    Calc {
        #[command(subcommand)]
        command: calc_cli::CalcCommand,
        #[arg(long, value_enum, default_value_t = calc_cli::OutputFormat::Text)]
        format: calc_cli::OutputFormat,
    },
}
// ...
Commands::Calc { command, format } => calc_cli::run(command, format)?,
```

### MCP, GUI, and the standalone app

- **MCP** - `gitehr-mcp` exposes each calculator as a tool. The tool's input schema is `Calculator::input_schema()` and the tool body calls `Calculator::calculate(value)`. This is the most LLM-native surface: typed schemas handed directly to the model rather than scraped from help text.
- **GUI** - the Tauri app calls `calc_core` natively over a Tauri command (`calculate_clinical`) and writes the result to the journal, rather than reimplementing logic in the webview.
- **Standalone calc app** - a separate Tauri 2 app (own `productName`, bundle identifier, and icons; no gitehr branding required) for desktop and mobile. Because `calc-core` is pure Rust it cross-compiles to iOS/Android. The app's frontend is the `calc-web` HTML, backed by the Rust core over `invoke`, so standalone and embedded calculators give byte-identical results.

### Distribution and decoupling

The leaf discipline (nothing in `calc-core` depends on gitehr or tokio) is what enables both of these without trade-off:

- **Install just the calculators**: `cargo install --git <repo> -p calc-cli` (or publish `calc-core` + `calc-cli` to crates.io for `cargo install calc-cli`). Cargo builds only `calc-core` + `clap` + `serde`, never the EHR. The installed binary name is `calc` (set by `[[bin]] name`), independent of the package name.
- **Ship a non-gitehr app**: the standalone Tauri app path-depends on `calc-core` and is branded independently. It can live in this workspace (excluded, like `gui/src-tauri`) or its own repo depending on `calc-core` via git/crates.io.

The one rule that keeps this true: `calc-core` must stay a leaf. `gitehr -> calc-core`, never the reverse.

### Binary-size note

Adding the calculators to the `gitehr` binary costs almost nothing, because `gitehr` already links `clap`, `serde`, and `serde_json` - so the simple score-based calculators add no new dependencies, only a few KB of code and string data each. The only thing that moves the needle is calculators embedding large reference datasets (growth charts, risk-equation coefficient tables); for those, prefer loading tables from an embedded asset rather than baking everything into the binary's read-only data.

---

## Result schema: `CalculationResponse`

The Rust struct and the JSON object dispatched by the web bridge are the same shape, so results cross surfaces unchanged.

```rust
pub struct CalculationResponse {
    pub calculator: String,         // machine name, e.g. "feverpain"
    pub result: serde_json::Value,  // primary computed value (number or short string)
    pub interpretation: String,     // human-readable clinical interpretation
    pub working: serde_json::Map<String, serde_json::Value>, // step-by-step breakdown
    pub reference: String,          // primary citation / guideline
}
```

```json
{
  "calculator": "asrs",
  "result": 4,
  "interpretation": "Positive screen: 4/6 Part A items meet the frequency threshold...",
  "working": {
    "part_a_screen_result": "POSITIVE",
    "part_a_positive_item_count": 4,
    "part_a_total_score": 9,
    "part_b_total_score": 12,
    "total_score": 21
  },
  "reference": "Kessler RC et al. (2005). Psychol Med. 35(2):245-56."
}
```

When dispatched from the browser, an optional `patient_context` object is appended (echoed from the host's URL parameters). `CalculationResponse::to_summary_text()` produces a deterministic, timestamp-free clipboard summary; the recording host adds the timestamp.

---

## The `Calculator` trait

```rust
pub trait Calculator {
    fn name(&self) -> &'static str;          // stable machine name / subcommand / MCP tool name
    fn title(&self) -> &'static str;         // human title
    fn description(&self) -> &'static str;   // one-line description
    fn reference(&self) -> &'static str;     // primary citation
    fn input_schema(&self) -> serde_json::Value;  // JSON Schema for inputs
    fn calculate(&self, input: &serde_json::Value) -> Result<CalculationResponse, CalcError>;
}
```

`input_schema()` is the key LLM affordance: it powers `--print-schema`, MCP tool definitions, and any agent that wants to discover the required inputs without parsing prose. Each calculator additionally exposes a typed `compute()` for ergonomic, compile-time-checked use from Rust.

---

## CLI design (LLM-friendly)

The CLI is built to be discoverable by an LLM agent from `--help` and machine-readable schemas:

```bash
calc --help                     # top-level help; --format text|json is global
calc list                       # list calculators (text or JSON)
calc list --format json         # [{name,title,description}, ...]
calc <name> --print-schema      # JSON Schema for that calculator's inputs
calc <name> [flags]             # compute; text by default
calc <name> [flags] --format json   # CalculationResponse as JSON on stdout

# Examples
calc feverpain --fever --purulence --attend-rapidly
calc feverpain --fever --purulence --attend-rapidly --inflamed-tonsils --absence-of-cough --format json
calc asrs --responses 2,2,2,3,0,0,1,1,1,1,1,1,1,1,1,1,1,1 --format json
```

Conventions: predictable exit codes, JSON on stdout under `--format json`, every input documented in `--help`, and `--print-schema` as the authoritative input contract. Man pages and shell completions are generated from the clap definitions (clap_mangen / clap_complete; the gitehr CLI already uses clap_complete) - this is the immediate next step after the core CLI lands.

---

## Web frontend (`calc-web`)

The browser tools are single, self-contained HTML files: no build step, no framework, openable as a static file, embeddable, or hosted on GitHub Pages. The only permitted dependencies are the shared bridge module, the shared stylesheet, and optionally CDN-hosted CSS (which must degrade gracefully offline).

### Design principles

1. **Single file, no build step** - all markup, CSS, and JS inline or via CDN. (ES module imports of the shared bridge require serving over HTTP, which any static server satisfies.)
2. **Bespoke UI per calculator** - the layout suits the clinical purpose. A questionnaire looks like a questionnaire; a converter looks like a converter. Guided by clinical context, patient-facing vs clinician-facing use, cognitive load, and accessibility (keyboard navigable, screen-reader compatible, sufficient contrast).
3. **Logic** - today the web logic is plain inline JavaScript. The canonical logic now lives in `calc-core`; the web layer obtains it one of two ways: by calling the Rust core (in the Tauri app via `invoke`, or by loading `calc-core` compiled to WASM), or, for the pure no-build single-file case, by a JavaScript mirror that is validated against the shared `calc-core` test vectors. New calculators should treat `calc-core` as the source of truth and keep any JS mirror provably equivalent. For heavyweight statistical logic, Pyodide may run authoritative Python in the browser.

### The bridge (`shared/gitehr-bridge.js`)

A small ES module that makes each calculator context-aware without the author knowing the host environment.

| Context | Detection |
|---|---|
| Tauri (GitEHR desktop / standalone app) | `window.__TAURI__` present |
| iframe embed | `window.parent !== window` |
| Standalone | neither of the above |

Exported API: `sendResult(data)` (Tauri event `calculator-complete`; iframe `postMessage`; standalone no-op), `getPatientContext()` (reads URL query params injected by the host), `getContext()`, `saveButtonLabel()`, `formatClipboardText(data)`, `copyToClipboard(data)`.

### Result Card UI conventions (mandatory)

Every calculator renders a result card, in this order:

1. **Score summary and interpretation** - score tiles, a result badge (positive/negative/severity), and the interpretation string.
2. **Breakdown** (recommended) - a collapsible `<details>` block with per-item or per-criterion scores.
3. **Clipboard preview textarea** - an editable `<textarea>` pre-filled with the plain-text summary, shown before the action buttons. The copy button reads `previewTA.value` directly so clinician edits are preserved. Use `formatClipboardText()` for simple results or a bespoke `buildSummaryText()` for richer narratives (e.g. FeverPAIN).
4. **Action buttons** - rendered by JS after `getContext()`:

| Context | Primary | Always present |
|---|---|---|
| `tauri` | "Save to patient record" (`sendResult`) | "Copy result", "Start over" |
| `iframe` | "Send result" (`sendResult`) | "Copy result", "Start over" |
| `standalone` | - | "Copy result", "Start over" |

**Dynamic refresh**: any post-result selection that changes the clinical recommendation (prescribing strategy, dosing, follow-up) must update the textarea in real time. Store the last score/interpretation at module level and call `refreshPreview()` from every relevant change listener; clear them on "Start over".

---

## GitEHR integration

When a calculator runs inside GitEHR, dispatch stops being a bridge round-trip and becomes a direct call into the journal/state code.

### Journal entries

Results are recorded as immutable, timestamped journal entries with structured YAML frontmatter (calculator type, version, inputs, result, citation) followed by a human-readable Markdown body. This makes every calculation a permanent, auditable part of the record.

### State files

Latest results may also be written to `state/calculations/<name>-latest.json` for quick reference, including the inputs, result, who calculated it and when, and a pointer to the journal entry.

### Patient context

The host may append patient identifiers as URL parameters before opening a web calculator (`?patient_id=...&given_name=...`); `getPatientContext()` exposes them for labelling, pre-fill, and inclusion in the dispatched payload. The GUI path passes context directly.

---

## Authoring a new calculator

1. Implement it in `calc-core`: a typed `Input`, a pure `compute()`, a `build_response()` adapter, a `Calculator` impl with `input_schema()`, and unit tests against known vectors. Register it in `all()`.
2. Add a `calc-cli` subcommand (flags + `--print-schema`) - mechanical, following `feverpain`/`asrs`.
3. (If a web tool is wanted) create `calc-web/calculators/<name>.html` following the Result Card conventions, with its JS logic validated against the `calc-core` vectors. Add a card to `calc-web/index.html`.
4. Add authoritative source material to `calc-web/clinical-source-references/`.

See `skills/build-calculator/` for the detailed authoring workflow.

---

## Calculator library roadmap

UK-focused build priority (50 tools), ordered by clinical volume and patient-safety impact. The first two (FeverPAIN, ASRS-v1.1) are implemented in `calc-core` as the reference pattern. The full table with per-tool descriptions lives in `spec/calculator-roadmap.md`.

### Tier 1 - High-volume primary care / NHS-mandated

QRISK3 (NICE NG238), PHQ-9 (NG222), GAD-7 (CG113), AUDIT / AUDIT-C (CG115), eGFR CKD-EPI (NG203), MUST (CG32), FRAX / QFracture (CG146), FIB-4 (NG49).

### Tier 2 - Acute / emergency

NEWS2 (NG51; RCP/NHSE mandated), CURB-65 / CRB-65 (NG250; BTS), Wells DVT / Wells PE (NG158), GRACE (NG185, CG94), CHA2DS2-VASc (NG196), HAS-BLED (NG196, NG158), ABCD2 (NG128), 4AT (CG103), qSOFA (NG51).

### Tier 3 - Common chronic disease management

MRC Dyspnoea (NG115), CAT (NG115), ACQ, IPSS (CG97), DAS28 (NG100), uACR (NG203, NG28), eGFR+uACR heatmap (NG203), EPDS (CG192; SIGN 169), Clinical Frailty Scale (NG56), MMSE (NG97).

### Tier 4 - Secondary care / specialist

SOFA (NG51), EuroSCORE II (TA163, TA245), HEART (NG185), TIMI (CG94), Padua (NG89), ELF (NG49), Child-Pugh / MELD / UKELD (NG50), Nottingham Hip Fracture Score (CG124; NHFD).

### Tier 5 - Functional / PROMs / niche but guideline-endorsed

AMTS (CG124), Waterlow (CG179), Oxford Hip / Knee Score (NHSE PROMs), BODE (NG115), LANSS (CG173), ABPI (NG19, CG168), Gleason Grade Groups (NG131), Nottingham Prognostic Index (NG101), CHALICE (CG176).

### RCPCH Digital Growth Charts (special case)

UK-WHO (0-4y, WHO 2006) and UK90 (4-20y) reference data, gestational-age correction for prematurity, z-score/centile/SDS calculation, and chart rendering in the GUI. Requires LMS reference tables (the binary-size variable noted above) and confirmation of RCPCH licensing terms for distribution.

---

## Clinical validation

Each calculator must include: a primary peer-reviewed citation; evidence of clinical utility; test cases with known inputs/outputs from the literature (encoded as unit tests in `calc-core`); documented limitations and contraindications; and a process for incorporating guideline changes.

---

## Licensing

- `calc-core` / `calc-cli`: AGPL-3.0-or-later (consistent with GitEHR).
- Clinical algorithms: implement from primary literature; most scores are public-domain methods. Do not copy proprietary implementations (e.g. MDCalc).
- RCPCH growth charts: confirm licensing terms with RCPCH before distribution.
- All calculators cite original publications and validation studies.

---

## Open questions

- Unit conversion support (metric/imperial)?
- Queryable calculation history (`gitehr calc history`)?
- Printable reports for results in the GUI?
- FHIR Observation export for standardized exchange?
- User-defined / third-party calculators via a plugin system?

## Future enhancements

Calculator plugins; fetching guideline updates from a registry; multi-step decision trees beyond simple scores; auto-populating inputs from current patient state; trending results over time; high-risk-score alerts.

---

This specification establishes GitEHR as a comprehensive clinical decision support tool with auditable, version-controlled calculation results, driven by a single engine that is equally at home in the EHR, at the command line, in an LLM's toolset, and as a standalone app.
