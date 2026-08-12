<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Evaluating Open Wearables for streamed vitals

[Open Wearables](https://github.com/the-momentum/open-wearables) is a self-hostable, MIT-licensed platform that normalises data from consumer wearables (Garmin, Oura, Whoop, Suunto, Polar, Ultrahuman, Strava, Fitbit) plus mobile-SDK sources (Apple HealthKit, Samsung Health, Google Health Connect) behind one REST API. It is early-stage: a FastAPI backend with PostgreSQL and Redis, a React developer portal for OAuth connections and API keys, and native SDKs for iOS, Android, Flutter, and React Native. This note assesses whether it is a good fit as an import source feeding wearable observations into GitEHR, per roadmap item R71.

## Why it is a plausible fit

GitEHR does not want to, and should not, become a wearables integration platform itself: each vendor's OAuth flow, API quirks, and data model is a maintenance burden orthogonal to GitEHR's job of being the durable record. Open Wearables already carries that burden and stays self-hosted, which matches GitEHR's local-first posture (see [Files, not databases](files-not-databases.md)) - the raw device data need never leave infrastructure the patient (or their clinician) controls. Its single normalised REST API is a much smaller integration surface for GitEHR than eight separate vendor integrations would be.

## Where the fit is loose

- **It is a server, not a file.** GitEHR's canonical record is a plain-file Git repository ([ADR-0001](../../spec/adr/0001-documents-as-plain-files.md)); Open Wearables is a running service with its own Postgres database. Any integration is necessarily an *import adapter* that polls or is pushed to, then writes GitEHR-native journal/state entries - Open Wearables' database is never the record of truth. That adapter is unbuilt; this evaluation does not reduce it to a small task.
- **Volume mismatch.** GitEHR's journal is append-only and expects clinically meaningful entries, not a stream of raw readings (see the streaming-vitals sketch in [`long-term-ideas.md`](../../spec/long-term-ideas.md#6-real-time-vital-signs-streaming)). Wearable data arrives at a cadence and volume - potentially thousands of readings a day per device - that would need summarisation (rolling-window state, periodic journal summaries) before it belongs in a repository, regardless of which upstream API supplies it.
- **Pre-1.0 and un-audited.** Open Wearables' README flags its own API as subject to change before a stable release. Standing up a service that handles real vitals data for real patients ahead of a 1.0 release, and without an independent security review, is a real cost worth weighing against building or waiting.
- **Operational surface.** Adopting it means a patient (or a clinic on their behalf) now runs and secures an additional FastAPI + Postgres + Redis stack, with its own OAuth credentials to each wearable vendor. That is a meaningfully bigger deployment than GitEHR's current single-binary CLI, and the security and consent model for that stack has not been assessed here.

## Recommendation

Do not adopt Open Wearables yet, but do not rule it out either: revisit it once R25/R26 (recording calculator-style, provenance-bearing state) and the streaming-vitals design sketch have gone from "exploratory" to a concrete plan for how high-volume readings become GitEHR journal/state entries. At that point, the useful next step is a small spike - point a local Open Wearables instance at one real device, and write a throwaway import adapter that turns its normalised API responses into a handful of state entries - to test the data-model and volume assumptions above against reality, rather than a deeper read of its source alone. Until then, R71 stays a documented option rather than a dependency.

See also: [Real-Time Vital Signs Streaming](../../spec/long-term-ideas.md#6-real-time-vital-signs-streaming) for the unresolved architecture questions any wearables import - through Open Wearables or otherwise - would still need to answer.
