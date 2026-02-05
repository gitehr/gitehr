# GUI Walkthrough

This walkthrough describes the current reference GUI. It is designed for developers and evaluators.

## Open a repository

If the GUI cannot detect a GitEHR repo, it shows an Open Repository screen. Select a folder that contains a `.gitehr` directory.

## Main layout

The GUI is organized into three regions:

- Header: brand mark and a search field.
- Left sidebar: navigation and repository status.
- Main area and right sidebar: patient overview, journal, and stateful summary.

## Journal panel

The Journal panel lets you add and review entries.

- Use the text box to write an entry.
- Select Add to append it to the journal.
- Recent entries show a short preview with timestamp and author.

## Repository status

The left sidebar includes a Repo Status card showing:

- Journal entry count
- Encryption state
- GitEHR repository version

## Stateful summary

The right sidebar shows a summary from `state/` files such as:

- Allergies
- Current medications
- Demographics

This is a read-only view. Update the underlying files directly or via the CLI.

## Activity feed

The Activity feed highlights the most recent journal entries and their timestamps.

## Error handling

If the GUI cannot read the repository, it shows an error banner with guidance to verify the repo path and backend status.
