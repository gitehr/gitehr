<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# GitEHR browser-extension extractor

*Status: proposal (draft). Answers "what technique gives a clean patient-portal extract, and could a Chrome extension do it?" - yes, and it is the right tool. Productises the patient-activated-extraction thesis ([`gitehr-patient-activated-extraction-synopsis.md`](gitehr-patient-activated-extraction-synopsis.md)) and the per-portal playbook ([`nhs-app-extraction-playbook.md`](nhs-app-extraction-playbook.md)) as an installable browser extension. Relates to #85 (observations), #86 (provenance/acquisition), #100 (lifestyle domain).*

## Why a browser extension is the right technique

The NHS App recon found the blocker precisely: the data endpoint (`/v1/patient/test-result`) is authorised by a **bearer token held in the page's JS runtime, not a cookie** - so you cannot replay a cookie, and you cannot re-request from outside the page. But you do not need to: **a browser extension runs inside the already-authenticated page**, so it can observe the page making its own authenticated calls.

Two capabilities an extension has that headless automation and external scripts do not:

1. **It is in the user's real, logged-in browser** - the session is present for free; no credential/token handling, no 2FA re-auth, and far less bot/headless detection (it is a genuine human browser).
2. **It can run in the page's MAIN world** and **hook `fetch` / `XMLHttpRequest`** - passively capturing the JSON responses the SPA already fetches (including the authenticated `/v1/patient/test-result` payloads), *without issuing any extra requests*. This yields the **clean JSON**, not just scraped DOM - solving the exact problem the recon surfaced.

Passive network observation is also the lowest-footprint, most robust approach: it reads what the page already loads, so it survives cosmetic DOM redesigns and adds no suspicious traffic. DOM scraping stays as the fallback when an endpoint is unknown.

## Architecture (Manifest V3)

- **Content script** (isolated world) per supported portal, gated by `host_permissions` / `matches` (e.g. `nhsapp.service.nhs.uk`, `systmonline.tpp-uk.com`, EMIS / Patient Access, ...).
- **Main-world injected hook** (`world: 'MAIN'`, or an injected `<script>`) patching `window.fetch` and `XMLHttpRequest.prototype` to capture request URL + response body for allow-listed portal endpoints, and posting captures back to the content script via `window.postMessage`.
- **Per-portal adapter ("playbook")**: (a) how to trigger loading all data (navigate / expand / paginate), (b) which endpoints to capture, (c) the map from captured JSON → FHIR / gitehr `Observation` / `Condition` / ... . Plus a **DOM-scrape fallback**. Adapters are community-contributable - one per portal.
- **Normaliser**: captured payloads → canonical gitehr shapes (FHIR-aligned), with terminology mapping (LOINC for labs) where possible.
- **Provenance / consent layer**: records what was captured, from which portal, when, and by which adapter version → feeds the acquisition/provenance model (#86). The user sees and approves what leaves the browser.
- **Handoff to gitehr** (options, simplest first):
  1. **Download a bundle** (JSON / Markdown / FHIR) the user drops into `gitehr import`. Zero coupling.
  2. **Local HTTP handoff** - the extension POSTs to a local `gitehr` listener (CLI/MCP running in a serve mode), which writes into the store with provenance.
  3. **Native messaging host** - the extension talks to a small local helper that runs `gitehr import`. Tightest integration, most setup.

## Legal / governance

Patient-run, in the patient's own browser, observing the patient's own authenticated session's own traffic, exporting locally. UK GDPR Art 15/20 territory; the defensible framing (per the synopsis) is patient-mediated, own-data, local-first - never a service scraping on others' behalf. The extension issues **no extra requests** to the portal (passive capture), keeping it clearly within "using the service as intended and keeping a copy of my own data."

## Roadmap shape

1. **v0** - one portal (NHS App), network-capture + DOM fallback, "download gitehr bundle". Proves the loop end-to-end.
2. **v1** - adapter framework + 2-3 portals (SystmOnline, EMIS / Patient Access), provenance layer, local handoff to `gitehr import`.
3. **v2** - community adapter contributions; normalisation to FHIR; terminology mapping; the observations/lifestyle domains (#85, #100).

This is the "extraction agents" layer of the synopsis made concrete and durable: not a one-off script per session, but an installable, community-extensible bridge from the walled portals into a patient-owned record. It also composes with GitEHR's Tauri GUI (a future in-app webview could host the same adapters).

Cross-ref: `gitehr-patient-activated-extraction-synopsis.md`, `nhs-app-extraction-playbook.md`, #85, #86, #100. The public position statement behind this approach is published at [`docs/design/patient-mediated-extraction.md`](../docs/design/patient-mediated-extraction.md).
