<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Multiple Stores are selected by the GUI, not managed by the CLI

A clinician may work with more than one independent GitEHR Store, such as a daytime general-practice Store and a separate cosmetic-practice Store. Each Store remains a complete, independent root containing its own MPI and any number of subject repos. We decided that choosing between those Stores is an application-level concern owned by the GUI, not a new CLI concept or command family.

## Decision

- The GUI has one **active Store** at a time. Its patient index, subject search, selected repo, and cached record data are scoped to that Store.
- The GUI offers a Store chooser for opening an existing Store, creating one, and returning to recently opened Stores. It provides a visible Store switcher after a Store is open.
- GUI recent-Store paths and optional display labels are local application preferences. A Store does not need extra shared metadata merely to appear in the chooser; its directory name remains its portable identity.
- Launch context wins: if the GUI is launched from a subject repo or Store root, it opens that Store. Otherwise, the GUI may offer the configured CLI Store as its initial default, then let the user choose another Store.
- Switching Store clears the selected subject, patient search, and cached record data. The GUI must warn before discarding an unsaved draft when such drafts exist.

## CLI boundary

The CLI continues to operate on exactly one Store or subject repo resolved from its current working directory, `GITEHR_STORE_PATH`, or the configured `store_path`. It needs no `store switch`, Store registry, or multiple-Store command syntax. The GUI invokes the CLI with the active Store or selected repo as its working context, so the same CLI commands have identical semantics in every Store.

`store_path` remains a useful default for direct CLI use and for the GUI's first launch outside GitEHR context. Selecting another Store in the GUI must not rewrite that global default or change the behaviour of unrelated terminal sessions.

## Consequences

- A clinician can keep separate practices operationally and visually distinct without creating a bespoke CLI mode for each practice.
- Search, MPI identifiers, and subject selection never cross Store boundaries. A combined cross-Store view requires an explicit future feature with its own access-control and safety design.
- The Store chooser is the natural place for future local conveniences such as pinned Stores and local display labels. These are GUI preferences, not clinical data and do not travel with the Store.

## Considered options

- **One very large Store with practice tags:** rejected. It combines otherwise independent MPIs and makes separation of policy, workflow, and search an application convention rather than a filesystem boundary.
- **A CLI Store switch command or global active-Store state:** rejected. It introduces hidden mutable process context and duplicates the GUI's selection workflow, while shell users already have explicit context through their directory and environment.
- **A shared Store-name field in the core format:** deferred. A portable Store does not need one for correct CLI operation, and a local GUI label meets the current UX need without expanding the stored format.
