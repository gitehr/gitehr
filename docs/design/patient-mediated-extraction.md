---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Patient-mediated data extraction

Most of a patient's health data does not live in GitEHR. It lives in a GP system, a hospital portal, a private lab's dashboard - places the patient is entitled to see, but not entitled to query, export, or hold a durable copy of in any structured form. GitEHR's position is that closing this gap is the patient's own right to exercise, and that a browser agent acting in the patient's authenticated session is a legitimate way to exercise it.

## The right, not a workaround

UK GDPR gives every data subject the right of access (Article 15) and the right to data portability (Article 20). Those rights do not depend on the medium of access, and they do not evaporate because exercising them is automated rather than manual. A patient who is authorised to view their own test results in the NHS App, and who uses a tool to capture what they are already entitled to see, is exercising a statutory right - not probing a system they have no claim on.

Health record suppliers rarely expose the APIs that would make this portability convenient, because the walled garden is frequently part of the business model. That commercial incentive does not change the underlying entitlement. This is [Cory Doctorow's adversarial interoperability](https://www.eff.org/deeplinks/2019/10/adversarial-interoperability-reviving-elegant-weapon-more-civilized-age): building a tool that interoperates with an incumbent platform on the user's behalf, without needing that platform's cooperation.

## The line GitEHR holds

The framing above only stays defensible if the practice stays inside a narrow lane. GitEHR's position:

- **Patient-run, not service-run.** The agent acts inside the patient's own authenticated session, initiated by the patient, for that patient's own data. It is not a service scraping on other people's behalf, and it is not offered as a way to access anyone else's record.
- **Own-data only.** The scope is what the patient can already see in their own portal session - never an attempt to reach data the patient is not authorised to view.
- **Local-first.** Extracted data is written to the patient's own local GitEHR repository. Nothing is routed through a third-party server as part of the extraction itself.
- **Passive where possible.** The preferred technique observes the traffic the portal's own page already generates (see [the browser-extension extractor proposal](https://github.com/gitehr/gitehr/blob/main/spec/browser-extension-extractor.md)) rather than issuing extra requests, and falls back to reading the rendered page. Either way, the portal sees nothing it would not see from the patient using it normally.
- **Credentials stay out of it.** The agent never captures, stores, or transmits the patient's login credentials or session tokens; it operates within a session the patient has already established.

## What this is not

This is not an argument that portal terms of service are irrelevant, and it is not an invitation to build a scraping service that acts for many patients at once without their direct, session-by-session involvement - that would be a different, and much less defensible, product. It is also not a substitute for suppliers doing the right thing: a patient-callable API (for example, an approved [GP Connect Patient Facing Services](https://digital.nhs.uk/services/gp-connect/gp-connect-in-your-organisation/gp-connect-patient-facing) integration) is a better long-term answer than any extraction agent, and GitEHR would rather use one than route around its absence.

## Why bother, if the API route exists in principle

Because today it largely doesn't, for an individual patient. GP Connect Patient Facing Services are gated to approved, PDS-compliant, clinical-safety-officer-backed consumer organisations, not to a patient's own personal tooling ([see the NHS App extraction playbook](https://github.com/gitehr/gitehr/blob/main/spec/nhs-app-extraction-playbook.md) for the concrete recon behind this). Until that changes, or until a patient-run tool clears that bar itself, the choice is between an inert statutory right and an activated one. GitEHR chooses activation, inside the constraints above.

## Demand is the lever

Suppliers change their posture on interoperability when patients visibly demand it, not when it is merely correct in principle. A community of patients routinely exercising, and talking about, their own data access rights is the most credible pressure toward suppliers eventually offering that access properly - at which point the extraction agents become unnecessary, which is the outcome GitEHR actually wants.

See also: [Files, not databases](files-not-databases.md) for why the extracted data is worth capturing in a durable, canonical format rather than left as a folder of portal exports.
