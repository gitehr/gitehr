/**
 * gitehr-bridge.js
 *
 * Tiny context-detection and result-dispatch module for GitEHR clinical
 * calculator HTML files.
 *
 * Each calculator HTML file imports this module and calls sendResult(data)
 * when the user has completed the tool.  The bridge works out where it is
 * running and dispatches the result appropriately:
 *
 *   1. Inside Tauri (window.__TAURI__ present)
 *      → emits a 'calculator-complete' event via the Tauri event system.
 *        GitEHR listens for this event and writes the result as a first-class
 *        journal or state entry.
 *
 *   2. Inside an iframe (window.parent !== window)
 *      → postMessage to the parent window with { type: 'calculator-result', ...data }.
 *        The embedding page (another app, an LLM chat, etc.) handles saving.
 *
 *   3. Standalone / GitHub Pages
 *      → no-op dispatch; the calculator's own UI already shows the result.
 *
 * Usage inside a calculator HTML file:
 *
 *   import { sendResult, getPatientContext, getContext } from '../shared/gitehr-bridge.js';
 *
 *   const ctx = getPatientContext();   // read ?patient_id= etc. from URL
 *   // … user fills in form …
 *   sendResult({
 *     calculator: 'asrs_screener',
 *     result: 5,
 *     interpretation: 'Positive screen: 5/6 items…',
 *     working: { part_a_screen_result: 'POSITIVE', total_score: 48 },
 *     reference: 'Kessler et al. (2005) Psychol Med 35(2):245-56',
 *   });
 *
 * Data schema (mirrors Python CalculationResponse):
 *   calculator      {string}  - calculator module name, e.g. 'asrs_screener'
 *   result          {*}       - primary computed value (number, string, …)
 *   interpretation  {string}  - human-readable clinical interpretation
 *   working         {object}  - step-by-step breakdown
 *   reference       {string}  - clinical guideline / citation
 *   patient_context {object?} - echoed back from getPatientContext() if provided
 */

// ---------------------------------------------------------------------------
// Context detection
// ---------------------------------------------------------------------------

/** @returns {'tauri' | 'iframe' | 'standalone'} */
export function getContext() {
  if (typeof window !== 'undefined' && window.__TAURI__) return 'tauri';
  if (typeof window !== 'undefined' && window.parent !== window) return 'iframe';
  return 'standalone';
}

// ---------------------------------------------------------------------------
// Patient context (URL params injected by the host, e.g. GitEHR)
// ---------------------------------------------------------------------------

/**
 * Returns any patient / session context passed via URL query parameters.
 * GitEHR (or any host) can append ?patient_id=…&given_name=… to the URL
 * before opening the calculator so the tool can pre-populate or label results.
 *
 * @returns {{ patient_id?: string, given_name?: string, family_name?: string,
 *             dob?: string, [key: string]: string | undefined }}
 */
export function getPatientContext() {
  if (typeof window === 'undefined') return {};
  const params = new URLSearchParams(window.location.search);
  const ctx = {};
  for (const [k, v] of params.entries()) {
    ctx[k] = v;
  }
  return ctx;
}

// ---------------------------------------------------------------------------
// Result dispatch
// ---------------------------------------------------------------------------

/**
 * Dispatch a completed calculator result to the host environment.
 *
 * In standalone mode this is a no-op — the calculator's own UI already
 * displays the result to the user.
 *
 * @param {Object} data  See module-level schema above.
 * @returns {Promise<void>}
 */
export async function sendResult(data) {
  const context = getContext();
  const payload = { ...data, patient_context: getPatientContext() };

  if (context === 'tauri') {
    try {
      // Tauri 2.x event API (window.__TAURI__.event.emit is async)
      await window.__TAURI__.event.emit('calculator-complete', payload);
    } catch (err) {
      console.warn('[gitehr-bridge] Tauri event emit failed:', err);
    }
    return;
  }

  if (context === 'iframe') {
    window.parent.postMessage(
      { type: 'calculator-result', ...payload },
      '*',   // origin wildcard — host should validate on receipt
    );
    return;
  }

  // standalone — nothing to do; calculator renders its own result card
}

// ---------------------------------------------------------------------------
// Convenience: show a context-appropriate "save" button label
// ---------------------------------------------------------------------------

/**
 * Returns a label for the primary action button after result is available.
 * Calculators can use this to adapt their UI without knowing about the host.
 *
 * @returns {string}
 */
export function saveButtonLabel() {
  switch (getContext()) {
    case 'tauri':  return 'Save to patient record';
    case 'iframe': return 'Send result';
    default:       return 'Copy result';
  }
}

/**
 * Format a plain-text summary of the result without copying it.
 * Useful for populating a clipboard-preview textarea.
 *
 * @param {Object} data  Same schema as sendResult().
 * @returns {string}
 */
export function formatClipboardText(data) {
  return [
    `Calculator: ${data.calculator}`,
    `Result: ${data.result}`,
    `Interpretation: ${data.interpretation}`,
    `Reference: ${data.reference}`,
    `Timestamp: ${new Date().toISOString()}`,
  ].join('\n');
}

/**
 * Copy a plain-text summary of the result to the clipboard.
 * Useful in standalone mode as a fallback action.
 *
 * @param {Object} data  Same schema as sendResult().
 * @returns {Promise<boolean>} true if succeeded
 */
export async function copyToClipboard(data) {
  try {
    await navigator.clipboard.writeText(formatClipboardText(data));
    return true;
  } catch {
    return false;
  }
}
