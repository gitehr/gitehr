# GitEHR

GitEHR is a decentralised and portable health record powered by Git, cryptography, and open standards.

## The Problem

Traditional health interoperability focuses on sending data from one **Organisation-Centric Database** to a different **Organisation-Centric Database**.

Databases work fine *inside* the walls of a particular organisation that invested resources in developing a bespoke solution for their needs.

The problem arises when trying to interoperate databases *between* organisations. Even in instances where organisations have the **exact same** EHR deployed, their local deployment customisations will make it difficult, if not impossible, to transfer data. If data *can* be moved, an associated cost is attached to 'translating' the data from one organisation's structure to another. This must then be done between every single organisation, for every data item: an exponential problem. One solution may be creating *larger* databases - instead of siloing to a single organisation, span the database to cover a region. Unfortunately, the same interoperability issues still exist and may even compound at regional, national, or international scales.

Looking at the well-established tech world, there are no successful examples of the traditional healthcare approach working at scale.

## GitEHR's Solution

*Distributed Version Control Software (DVCS)*, such as *Git*, provides exemplary proof of how to engineer multi-contributor and secure. Crucially, these private repositories are agnostic of the software used to view or manipulate them.

Though areas which require specialised healthcare technology exist, these health-specific solutions are not *always* required. Many problems faced in healthcare have already been solved by the wider tech world - which is why GitEHR draws extensively on existing technology.

!!! quote "Rob Dyke Says"
    "All software has already been written."

Medical records need interoperable and extensible file formats. GitEHR will have an open standard with a reference implementation that is open-source. It will enable many other implementations to exist. Multiple platforms for reading, writing, and processing records can exist.

GitEHR solves the interoperability problem, setting the foundations to solve many other problems of health technology.

The paradigm must change. This is the beginning.
