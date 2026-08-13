---
title: Acknowledgements
---

# Acknowledgements

GitEHR builds on ideas from a number of people and projects that came before it. We want to acknowledge those contributions here.

## Prior art

### mdr-git - Amitai Burstein (Gizra)

[mdr-git](https://github.com/amitaibu/mdr-git) is a proof-of-concept for plain-text medical records with a Git backend, created by [Amitai Burstein](https://github.com/amitaibu) of [Gizra](https://www.gizra.com/), an Israeli software company.

The project arose from Gizra's work with [The Ihangane Project](https://www.theihanganeproject.com/), an HIV prevention program in Rwanda. After visiting health centres in rural Rwanda and seeing the challenges of offline-first medical record keeping in low-infrastructure settings, Amitai was inspired by [plain-text accounting](https://www.gizra.com/content/plain-text-accounting-hledger/) (hledger) to ask a simple but powerful question: what if medical records were stored as plain-text files and Git handled syncing, offline work, and versioning?

The result was a POC in which all patient data lives in human-readable YAML files, with Git as the transport and concurrency layer, and a lightweight local PHP/Symfony server (running via [Termux](https://termux.com/) on Android devices) as a nicer interface for editing those files. The accompanying [blog post](https://www.gizra.com/content/plain-text-medical-records/) (January 2020) lays out the reasoning with great clarity.

mdr-git is, so far as we know, the earliest explicit articulation of the idea that Git - a distributed version control system - is a natural fit for medical records in resource-constrained environments, and that storing clinical data as human-readable text rather than in a database has independent value. It was experimental and never moved beyond a proof-of-concept, but the core insight directly prefigures and inspired GitEHR.

We are grateful to Amitai for sharing this idea openly.