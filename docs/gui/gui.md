# GUI Walkthrough

This walkthrough describes the current reference GUI. It is designed for developers and evaluators.

## Open a repository

If the GUI cannot detect a GitEHR repo or configured Store, it shows an Open Repository screen. Select either a Store root containing `gitehr-mpi.json` or a subject repo containing `.gitehr`.

## Patient Index

Opening a Store root shows the Patient Index: a searchable list of subjects (name, ID, NHS number, MRN) with an **Add Subject** control. Opening a subject shows its record; **Patient list** returns to the index.

## Main layout

- Header: GitEHR identity plus typed patient demographics, all active allergies, and identifiers.
- Left sidebar: the **Patient list** button when a Store is open; otherwise empty.
- Right sidebar: reserved, intentionally empty until its workflows are real.
- Main area: read-only Problems, Medications, Observations and Vaccinations cards, then the Journal panel.

## Journal panel

The Journal panel lets you add and review entries.

- Use the text box to write an entry.
- Select Add to append it to the journal.
- Select Document to choose one or more local files and attach them to the record.
- If the text box contains narrative text when Document is selected, the text and selected files are saved together as one journal entry.
- Recent entries show a short preview with timestamp and author.
- Supported attachments preview inline: images render large in the journal; PDFs render in a scrollable embedded viewer where the platform webview supports PDF display.

## Header data

The record header reads typed state through Tauri commands backed by the CLI:

- `gitehr demographics show --json` for title, name, address, DOB, and identifiers.
- `gitehr allergies list --json` for active allergy warnings.

## Error handling

If the GUI cannot read the repository, it shows an error banner with guidance to verify the repo path and backend status.
