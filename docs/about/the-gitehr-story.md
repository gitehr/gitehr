---
author: Dr Marcus Baw
reviewers: Dr Anchit Chandran
---

### By [Dr Marcus Baw](./contributors.md#dr-marcus-baw)

As a clinician starting to become interested in health tech around 2011 - as the [National Programme for IT (NPfIT)](https://en.wikipedia.org/wiki/National_Programme_for_Information_Technology) was collapsing - I found it difficult to find a 'way in' to the world of General Practice IT. Clinicians were not wanted in a world dominated by large companies delivering unusable and 'legacy' software from day one.

Eventually, I found two ways in: *[NHS Hack Day](https://nhshackday.com/)*, and the *[RCGP](https://www.rcgp.org.uk/) Health Informatics Group*. These were my starting points, but since then, I've been almost full-time in health informatics in various roles, from the unorthodox: freelance General [Hacktitioner](../glossary.md#hacktitioner) to the very much orthodox: Chair of the RCGP Health Informatics Group (HIG).

I've had a somewhat unusual Clinical Informatics career in that I've also learned a lot about the **technical** underpinnings of the software. Unlike many clinical informaticians and NHS 'digerati', I learned to write code and become deeply technical, giving me a different view of the proffered health tech solutions.

I couldn't understand that **every single** current health tech solution is **database-backed**, and thereby inherently Organisation-Centric.

Even proposed fixes such as [FHIR](https://hl7.org/fhir/) and [OpenEHR](https://openehr.org/), designed to improve interoperability, are imperfect as they are still centralised and [organisation-centric](../glossary.md#organisation-centric), database-backed solutions which fall into the same trap as all the others.

Size doesn't matter: eventually, all organisations have their 'edges'. Here is where absolute and unresolvable problems of interoperability exist.

I've been puzzling this over for a decade for a solution. In that time, I've learned more about how the tech world works and what existing solutions can be brought into health.

[Git](https://git-scm.com/) has been my primary [DVCS](https://en.wikipedia.org/wiki/Distributed_version_control) during this time too. It took me a long time to realise that this issue in healthcare is identical to the [CVS](https://en.wikipedia.org/wiki/Version_control#Source-management_models) vs [DVCS](https://en.wikipedia.org/wiki/Distributed_version_control) debate of 15-20 years ago in 'normal' tech. A debate that was unequivocally won by DVCS in the form of Git.

At the beginning of my health tech journey, I remember googling 'medical file format' thinking the solution was here. There was little to find aside from some barmy XML dialect. Also, technology people disparaged files, pointing to all the work surrounding databases. I've spent years of experience learning about these systems, eventually coming full circle to where I began.

## Files are the answer

It feels like starting completely from scratch.

In some ways, *it is*.

This doesn't mean all the work accomplished in the world of healthcare interoperability needs to be discarded. For example, much can be learned from [FHIR](https://www.hl7.org/fhir/) and [OpenEHR](https://www.openehr.org/) to help inform the structure of a health record.

!!! tip
    If it helps, you can do what my friend [Kevin Monk](https://twitter.com/kevinmonk) did: realise that *a file is like a 'mini database all of its own' on a disk*. And now, relax.
