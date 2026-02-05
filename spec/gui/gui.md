<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `GitEHR GUI`

In no situations should we expect a clinicians to use the GitEHR CLI directly. The CLI is an interoperability layer and a tool for developers and technical users to interact with the GitEHR repository, but it is expressly not designed for clinical use. Clinicians need a more intuitive and user-friendly interface to access and manage patient records, which is why we have developed the GitEHR GUI.

The GitEHR GUI (`gitehr gui`) is a cross-platform graphical user interface application designed to provide clinicians and developers with an intuitive way to interact with a GitEHR repository. Each repository represents a single patient's complete medical record.

The GUI wraps the GitEHR CLI commands in a user-friendly interface, allowing users to easily navigate and visualize the patient's clinical data. It provides features such as timelines, charts, and summaries to help clinicians quickly understand the patient's medical history and current status.

The GUI cannot implement a feature that is not supported by the GitEHR CLI. It serves as a visual layer on top of the existing CLI functionality. In short, if we want a feature in the GUI, we need to implement it in the CLI first. The GUI is designed to be a companion tool that enhances the user experience while leveraging the capabilities of the GitEHR CLI.


## Features

* Wraps the GitEHR CLI commands in a user-friendly graphical interface.
* Provides visualizations of the patient's clinical data, including timelines, charts, and summaries.
* Enables easy navigation and searching of the medical record.

## Initial Load

Upon launching the GUI, users will be prompted to select an existing GitEHR repo or create a new GitEHR repository. Once a repository is selected or created, the GUI will load the patient's medical record and display an overview of the clinical data, including recent journal entries, current medications, allergies, and demographic information.

## Layout

* **Left Sidebar**: Navigation menu for accessing different sections (e.g., Journal, State, Imaging, Documents).
* **Main Content Area**: Displays clinical data, forms, and visualizations.
* **New Entry Button**: Quick access to add new journal entries.
* **Stateful Area**: Displays information from the `state/` directory, such as allergies, current medications, and demographic information.