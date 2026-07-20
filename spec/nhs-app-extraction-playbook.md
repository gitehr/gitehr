<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# NHS App extraction playbook

*Status: recon draft. The concrete per-portal companion to [`gitehr-patient-activated-extraction-synopsis.md`](gitehr-patient-activated-extraction-synopsis.md) for the highest-value UK source, the NHS App (England). Written from desk research; the live DOM/network specifics are a recon template to fill in from an authenticated session. Relates to #85 (structured observations) and #100 (lifestyle domain).*

## TL;DR

There is **no open, patient-callable API** to pull your own GP data. The NHS App's test results come from the patient's GP system (EMIS, TPP SystmOne) via the **GP Connect Patient Facing FHIR APIs** - but those are gated to approved, PDS-compliant, RBAC-managed, clinical-safety-officer-backed *consumer organisations* (the NHS App is one). A personal tool cannot call them.

So the pragmatic route is **adversarial interoperability: drive the patient's own authenticated NHS App web session and capture the data**, in two modes, best first:

1. **Network capture** - intercept the web SPA's own XHR/fetch responses to its backend. The data underneath is FHIR-derived, so these are likely JSON close to `Observation`/`DiagnosticReport`. Cleanest; minimal transform.
2. **DOM scrape** - parse the rendered results table. Robust fallback, brittle to redesigns.

## Auth model - why you cannot just replay a token

- NHS login is **OIDC on OAuth 2.0**: authorisation-code flow, `private_key_jwt` at the token endpoint, Vectors of Trust (identity-assurance levels), and 2FA. ([NHS login OIDC flow](https://nhsconnect.github.io/nhslogin/oidc-login-flow/))
- The web app keeps tokens **server-side (backend-for-frontend pattern)** and hands the browser only an **httpOnly session cookie**. The browser never holds a usable bearer token to lift.
- Consequence: automation must operate **inside the live authenticated browser session** (the cookie), not by replaying a token or credentials. The session is httpOnly, likely short-TTL, and probably headless/bot-detected.
- This is precisely why "log in on normal Chrome, reuse the cookie in headless Playwright" **does not work**: headless launches an isolated browser profile (no cookie); and even copying the cookie is fragile (httpOnly, server-session binding via the BFF, TTL, device/UA/context checks).

## Session-sharing options for a browser agent (ranked)

1. **Agent in the user's real, already-authenticated tab** - the Claude-in-Chrome extension or a userscript. Shares the session natively; proven to work on the NHS App. Lowest friction.
2. **Playwright/CDP attach to the user's running Chrome** - start Chrome with `--remote-debugging-port=9222`, connect Playwright over CDP; reuses the real profile + live session. Best for scripted repeatability.
3. **Playwright persistent context on the user's Chrome profile dir** - needs Chrome closed (profile lock) and the correct profile path; brittle.
4. **`storageState` replay** - export cookies/localStorage into Playwright. Fragile here (httpOnly + BFF server-session + TTL); the session is often already invalid.

## Data source & format

- Test results = GP-record data from EMIS/TPP surfaced via **GP Connect Patient Facing Services** (FHIR). ([GP Connect Patient Facing](https://digital.nhs.uk/services/gp-connect/gp-connect-in-your-organisation/gp-connect-patient-facing), [Access Record FHIR API](https://digital.nhs.uk/developer/api-catalogue/gp-connect-patient-facing-access-record-fhir))
- So the SPA's backend responses are likely FHIR-derived JSON (`Observation`, `DiagnosticReport`). Capturing those maps almost directly onto gitehr's observations primitive (#85) - little transformation.
- If only the DOM is available: each result row typically carries analyte name, value, unit, reference range, date, and an abnormal flag - map those fields.

## Recon findings — first live run (2026-07)

Target: `https://www.nhsapp.service.nhs.uk/patient/test-results/gp-ordered-test-results`. First extraction done via the Claude-in-Chrome agent in an authenticated session (one patient, GP-ordered results). This **confirms the auth analysis above** with a concrete endpoint.

- **Backend endpoint (confirmed):** detail pages call `GET https://api.nhsapp.service.nhs.uk/v1/patient/test-result?testResultId=<id>` (observed id `80425d56...`, HTTP 200). A separate list/index endpoint presumably backs the results-list page (capture next run). The name (`test-result`, not `Observation`) suggests a **bespoke JSON shape, not raw FHIR** — confirm by capturing a body.
- **Auth reality (confirms the BFF/token model):** that call is authorised by a **bearer token held in the page's JS runtime, not a shared cookie**. A re-fetch issued from an out-of-page script came back **unauthenticated/empty**. So you can neither lift-and-replay a cookie **nor** re-request the endpoint from outside the page.
  - **Implication for the extraction agent:** to obtain the clean JSON you must run **inside the page's JS context** and either (a) read the runtime token (credential material — avoid extracting/displaying it), or (b) **hook `fetch`/`XHR`** to capture responses as the page itself issues them. The lowest-risk path **proven to work** is reading the **rendered detail-page DOM** — identical values, no credential handling.
- **DOM extraction worked end-to-end:** every analyte's name, value, unit, reference range, date and out-of-range flag are present on rendered detail pages. Results are grouped into panels; each result has a detail view carrying the reference range. No clinician free-text was exposed on these panels.
- **Data shape:** panels are grouped by specimen/collection; the app separates specimens collected minutes apart on the same date.
- **Pagination:** the most-recent section fitted one page (single collection date); behaviour across many historical dates still to capture.

### Recommended agent recipe (NHS App test results)

1. Operate in the user's own authenticated tab (Claude-in-Chrome, or an in-page userscript).
2. Navigate to the GP-ordered results page; expand each panel / open each detail view.
3. Capture by **DOM scrape of the detail pages** (robust, no credential handling) — or, for cleaner JSON, inject a `fetch`/`XHR` hook to capture `/v1/patient/test-result` responses in-page.
4. Emit both a Markdown table and a JSON array of `{test, value, unit, ref_range, date, flag, comment}` → maps to the gitehr `Observation` primitive (#85).

### Still to capture (next runs)

- [ ] The list/index endpoint behind the results-list page, and its response shape.
- [ ] A captured `/v1/patient/test-result` response body (bespoke JSON vs FHIR?).
- [ ] Session TTL and what triggers re-auth (idle timeout, navigation).
- [ ] Pagination behaviour across many historical dates; "GP-ordered vs other" filtering.
- [ ] DOM selectors / `data-*` / ARIA of a result row, for a resilient scraper.

## Legal / governance framing

- Patient extracting **their own** data via **their own** authenticated session is UK GDPR Art 15 (access) / Art 20 (portability) territory. The statutory right does not evaporate because access is automated.
- The NHS App ToS may prohibit automated access; the defensible public position is **patient-run, own-data, local-first** (per the synopsis). Keep it patient-mediated, never a service scraping on others' behalf.
- The GP Connect PFS APIs are the "front door" if gitehr ever pursues approved-consumer status - heavy (PDS, RBAC, DCB0129, clinical safety officer). A long-term option, not v1.

## Alternatives / complementary sources

- **Approved third-party GP Connect PFS apps** (Evergreen Life, Airmid UK, Patients Know Best, ...) already give patients structured GP-record access via NHS login consent; some may export. A legitimate bridge worth evaluating.
- **GP online services** (SystmOnline / EMIS Patient Access / Patient Access) show the same data; some allow print/export.
- **SAR** (UK GDPR Art 15) to the GP practice for the full structured record - slow but complete.

## Playbook shape for gitehr

One skill/playbook per portal (NHS App, SystmOnline, EMIS Patient Access, ...), each packaging: (a) navigation steps in the authenticated session; (b) the capture method (network > DOM) with concrete endpoints/selectors; (c) the data → FHIR / gitehr-`Observation` mapping. Community-contributable. This document is the NHS App instance; extract once against a live session to complete the recon checklist above.
