# GUI Quick Start

This guide walks you through opening and interacting with a health record in the GitEHR GUI.

GitEHR is designed to give you ownership and a clear history of your health information. In this early preview, we focus on how to view and add to an existing record. See [Install the GUI](../install/gui.md) if you have not built the GUI yet.

## Before You Begin

This is a developer preview. You can create a new Store from the opening screen, or open an existing Store or subject repository. The GUI needs the `gitehr` CLI installed and on your `PATH`; if it cannot find it, it shows a warning.

## 1. Open the GitEHR Application

Launch the app. If a Store is configured, the GUI opens it automatically. To set one once:

```bash
gitehr config set-store ~/gitehr-store
```

## 2. Open or Create a Store

If nothing is detected, the **No GitEHR Repository Detected** screen appears:

1. To open existing data, select **Open Repository** and choose either a Store root (contains `gitehr-mpi.json`) or a subject repository (contains `.gitehr`).
2. To start fresh, type a name in **First subject name** (a person, family member, or pet) and select **Create New Store**.

## 3. Choose a Subject

A Store opens on the **Patient Index**. Use the search box to filter by subject name, ID, NHS number or MRN. Type a name and select **Add Subject** to create another subject in the Store. Open a subject from the list to view their record. Select **Patient list** in the left sidebar to go back.

## 4. Verify the Record

The header shows the subject's name, title, date of birth, address, allergies and identifiers. Check these match the person you intended to open. Allergies show every recorded allergen; "None recorded" means no allergy has been recorded, not that none exists.

## 5. Review the Current State

Below the heading, four read-only cards show Problems, Medications, Observations and Vaccinations. They are changed through the CLI (for example `gitehr medications add`), not in the GUI.

## 6. Add a Journal Entry

The **Journal** is a permanent, chronological log.

1. Type in the text box ("Write a new journal entry...").
2. Select **Add** to append it to the record.
3. To attach files, select **Document** and choose them. Any text in the box is saved with the files as a single entry.

Recent entries appear with a timestamp and author. Existing entries cannot be edited.

## Next steps

- [GUI overview](gui.md) - the rest of the interface (search, status, activity feed).
- [Repository Structure](../design/repository-structure.md) - how the data is actually organised on disk.
