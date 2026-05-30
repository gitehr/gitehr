# Getting Started

Welcome to GitEHR. This guide will help you open and interact with a health record using the GitEHR interface.

GitEHR is designed to give you ownership and a clear history of your health information. In this early preview, we focus on how you can view and add to an existing record.

## Before You Begin

This is a developer preview. Currently, the application cannot create new record folders from within the interface. 

To use this guide, you should already have a GitEHR record folder on your computer (likely provided by an administrator or created for testing). This folder contains all the health data for a single individual.

## 1. Open the GitEHR Application

Launch the GitEHR app on your computer. When it starts, it will look for a health record to display.

## 2. Select a Record Folder

If the app doesn't automatically load a record, it will ask you to choose one:

1. Click the **Open** or **Select Folder** button.
2. Navigate to the folder on your computer that contains the health record.
3. Select the folder and confirm. 

*Note: The app specifically looks for a folder that contains a hidden .gitehr marker. If you select a folder without this, the app will let you know.*

## 3. Verify the Patient Record

Once the record is loaded, the interface will display the current status.
- Check the **Summary** panel on the right.
- Ensure the information matches the record you intended to open (such as the patient identifier or current health status).

## 4. Add a Journal Entry

The **Journal** is a permanent, chronological log of clinical events and notes.

1. Navigate to the **Journal** tab in the main view.
2. You will see a list of previous entries. To add a new one, find the text box at the bottom.
3. Type your note or observation.
4. Click **Add** to save it to the record.

Your new entry will appear at the top of the list with a timestamp.

## 5. Understanding the Health Summary

The right-hand panel provides a "State Summary." This shows the most up-to-date information extracted from the record, such as:
- Current Medications
- Known Allergies
- Active Health Problems

In this preview version, the summary panel is **read-only**. Updates to these specific fields are currently managed through administrative tools, while the Journal remains the primary way to add new narrative information.

## Next steps

- [GUI Walkthrough](guides/gui-walkthrough.md): Learn more about the different parts of the interface.
- [Repository Structure](guides/repository-structure.md): See how the data is actually organized in your folder.
