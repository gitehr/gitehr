---
name: build-calculator
description: Use this skill when asked to build, scaffold, or add a new clinical calculator to this project. Covers the full workflow: reading the spec, designing bespoke UI, implementing scoring logic, wiring the bridge, and following all result-card conventions.
version: 1.0.0
---

# Build a Clinical Calculator

## Before you start

Read these files in full — they are the source of truth:

- `spec/calculators.md` — architecture (one core, many surfaces), the `Calculator` trait, bridge API, result-card conventions, the roadmap
- `calc-core/src/calculators/feverpain.rs` and `asrs.rs` — the canonical scoring logic and test-vector pattern; **`calc-core` is the source of truth for scoring**
- `calc-web/shared/gitehr-bridge.js` — the bridge module the web tools import
- `calc-web/shared/styles.css` — the shared stylesheet (do not duplicate these styles locally)
- An existing web calculator (e.g. `calc-web/calculators/feverpain.html`) — study its structure end-to-end before writing a single line

The roadmap (with NICE guideline references) lives in `spec/calculators.md § Calculator library roadmap`.

A complete new calculator means: a `calc-core` implementation with unit tests, a `calc-cli` subcommand, and optionally a `calc-web` HTML tool whose JS logic is validated against the `calc-core` test vectors.

---

## File to create

```
calc-web/calculators/<calculator-name>.html
```

One self-contained HTML file. No build step. No external JS beyond the bridge and optional CDN CSS. (The scoring logic itself belongs in `calc-core` first — see below.)

---

## Structure of a calculator file

```
<head>
  charset, viewport, title ("Tool Name — GitEHR Clinical Calculators")
  DaisyUI 5 + Tailwind CSS 4 via CDN          ← shared CDN stack
  <link rel="stylesheet" href="../shared/styles.css" />   ← shared styles
  <style type="text/tailwindcss">
    /* ONLY calculator-specific rules here */
    /* Do NOT redefine body, .site-header, .disclaimer, */
    /* .freq-group/.freq-btn, .progress-*, .result-card, */
    /* .calc-btn/.calc-btn-outline, details/summary      */
  </style>
</head>

<body>
  <header class="site-header">               ← header bar (styles from shared)
  <div style="background: var(--dark-blue)"> ← hero / intro band with live counter
  <div class="disclaimer">                   ← clinical disclaimer
  <main>
    <!-- input fields / questions -->

    <!-- Result card (hidden until complete) -->
    <div id="result-card" class="result-card hidden">
      <!-- score tiles -->
      <!-- interpretation text -->
      <!-- breakdown (collapsible details) -->
      <!-- reference citation -->

      <!-- Clipboard preview -->
      <div id="clipboard-preview-wrap" class="hidden mb-4">
        <label for="clipboard-preview" ...>Text to copy — edit if needed</label>
        <textarea id="clipboard-preview" rows="7|10" class="... font-mono resize-y"
                  style="outline-color: var(--mid-blue);"></textarea>
      </div>

      <!-- Action buttons (rendered by JS) -->
      <div id="result-actions"></div>
    </div>
  </main>
  <footer>GitEHR Clinical Calculators · Not a substitute for clinical judgement</footer>

  <script type="module">
    import { sendResult, getPatientContext, getContext,
             saveButtonLabel, formatClipboardText }
      from '../shared/gitehr-bridge.js';

    // 1. Question / field data
    // 2. State object (answers, treatment decisions, currentScore, currentInterp)
    // 3. Scoring function (mirrors Python exactly)
    // 4. Interpret function
    // 5. Render inputs
    // 6. Handle input events → call showResult() when complete
    // 7. showResult() → sets currentScore / currentInterp
    // 8. renderBreakdown()
    // 9. renderActionButtons()   ← see conventions below
    // 10. refreshPreview()       ← called by any post-result selection change
    // 11. applyPatientContext()
    // 12. Init calls
  </script>
</body>
```

---

## Brand tokens (from shared/styles.css — reference only, do not redeclare)

```css
:root {
  --dark-blue:   #003087;
  --mid-blue:    #005EB8;
  --light-blue:  #41B6E6;
  --color-green: #009639;
  --warm-grey:   #F2F2F0;
  --mid-grey:    #D9D9D9;
}
```

---

## Scoring logic

- The **source of truth is `calc-core`**. Implement the typed `Input`, pure `compute()`, `build_response()`, and `Calculator` impl there first, with unit tests against known vectors from the literature. Add a `calc-cli` subcommand.
- Any web (`calc-web`) implementation mirrors `calc-core` and must be validated against the same test vectors — variable names and logic should match so it stays trivially auditable.
- Use the authoritative source material in `calc-web/clinical-source-references/` as the clinical reference.
- For lookup tables or non-trivial statistics, prefer embedding the data in `calc-core`; in the browser, Pyodide may run authoritative Python (see `spec/calculators.md`).

---

## Bridge usage

```js
import { sendResult, getPatientContext, getContext,
         saveButtonLabel, formatClipboardText }
  from '../shared/gitehr-bridge.js';

// Read patient context from URL params (may be empty in standalone)
const patientCtx = getPatientContext();

// Build result payload (mirrors Python CalculationResponse)
const resultData = {
  calculator:    'calculator_name',   // snake_case module name
  result:        primaryValue,        // number or short string
  interpretation: interpretationText,
  working:       { ...breakdown },    // all intermediate values
  reference:     'Author et al. ...',
};

// Dispatch to host
sendResult(resultData);

// Format clipboard text (for the preview textarea)
const text = formatClipboardText(resultData);
// OR write a bespoke buildSummaryText() when richer narrative is needed
```

---

## Result card conventions (mandatory)

Follow the order in `spec.md § Result Card UI Conventions`:

### Clipboard preview textarea

Always present. Populate it from `renderActionButtons` when the result is first shown:

```js
const previewWrap = document.getElementById('clipboard-preview-wrap');
const previewTA   = document.getElementById('clipboard-preview');
previewTA.value = formatClipboardText(resultData);   // or buildSummaryText()
previewWrap.classList.remove('hidden');
```

Reset it on "Start over":

```js
document.getElementById('clipboard-preview-wrap').classList.add('hidden');
```

### Dynamic refresh for treatment decisions

Any post-result selection (prescribing strategy, dosing decision, follow-up choice) must
update the preview in real time. Store score/interp at module level and refresh on every
change:

```js
let currentScore = null;
let currentInterp = null;

function refreshPreview() {
  if (currentScore === null) return;
  const previewTA = document.getElementById('clipboard-preview');
  if (previewTA) previewTA.value = buildSummaryText(currentScore, currentInterp);
}

// In renderActionButtons / showResult:
currentScore = score;
currentInterp = interp;

// In every treatment-decision change listener:
radioInput.addEventListener('change', () => { selectedStrategy = r.value; refreshPreview(); });
```

Clear `currentScore` / `currentInterp` on "Start over".

### Action buttons

```js
function renderActionButtons(resultData) {
  const container = document.getElementById('result-actions');
  container.innerHTML = '';

  const previewTA = document.getElementById('clipboard-preview');
  const ctx = getContext();

  if (ctx === 'tauri' || ctx === 'iframe') {
    const saveBtn = document.createElement('button');
    saveBtn.className = 'calc-btn';
    saveBtn.textContent = saveButtonLabel();
    saveBtn.addEventListener('click', () => sendResult(resultData));
    container.appendChild(saveBtn);
  }

  const copyBtn = document.createElement('button');
  copyBtn.className = 'calc-btn' +
    (ctx === 'tauri' || ctx === 'iframe' ? ' calc-btn-outline' : '');
  copyBtn.textContent = 'Copy result';
  copyBtn.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(previewTA.value);
      copyBtn.textContent = 'Copied ✓';
    } catch {
      copyBtn.textContent = 'Copy failed';
    }
    setTimeout(() => (copyBtn.textContent = 'Copy result'), 2000);
  });
  container.appendChild(copyBtn);

  const resetBtn = document.createElement('button');
  resetBtn.className = 'calc-btn calc-btn-outline';
  resetBtn.textContent = 'Start over';
  resetBtn.addEventListener('click', resetAll);
  container.appendChild(resetBtn);
}
```

---

## Patient context

Always call `applyPatientContext()` on init. If the host passes patient info via URL params,
display it unobtrusively in the header. Include `patient_context: getPatientContext()` in the
`resultData` payload.

---

## Checklist before finishing

- [ ] Scoring output matches Python test vectors for all significant inputs
- [ ] Result card order: tiles → interpretation → breakdown → clipboard preview → action buttons
- [ ] Clipboard preview appears on result, hides on reset
- [ ] Copy button reads `previewTA.value` (not a re-generated string)
- [ ] Treatment decision changes call `refreshPreview()` immediately
- [ ] `sendResult` called with full payload including `working` and `reference`
- [ ] `applyPatientContext()` called on init
- [ ] "Start over" resets all answers, visual state, clears `currentScore`/`currentInterp`, hides result card and preview, scrolls to top
- [ ] No RCPCH branding anywhere — use "GitEHR Clinical Calculators" in title, header, footer
- [ ] No shared styles redeclared locally (`shared/styles.css` covers them)
- [ ] Keyboard navigable; sufficient colour contrast; screen-reader labels on interactive groups
- [ ] Card added to `index.html`
