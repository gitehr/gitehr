# gitehr

A Git-based, decentralised, multi-contributor Electronic Health Record

# GitEHR specification

## Commands

### `gitehr init`

Initializes a new GitEHR repository in the current directory, creating the necessary folder structure, and including a copy of the gitehr binary in the `.gitehr` folder.

### `gitehr add`

Adds a new clinical document to the GitEHR repository.

### `gitehr encrypt`

Encrypts the repository using a supplied key

### `gitehr decrypt`

Decrypts the repository using a supplied key.

### `gitehr status`

Displays the current status of the GitEHR repository, including any uncommitted changes and the status of the encryption.

### `gitehr transport`

Converts the repository into a single-file format for easier transport

### `gitehr extract`

Extracts the contents of a GitEHR repository from the single-file format back to a folder structure

## GitEHR repository lifecycle

### Initialization

`gitehr init` creates a folder structure inside the current directory. The folders are copied from the `gitehr-folder-structure` directory. On creation of the repository, the first file is created with random data and a timestamp.

### Adding entries

The `journal` contains sequential clinical entries in chronological order. Each file is named with a timestamp and GUID, meaning that in a normal file listing or file manager chronological ordering is easily possible. Each subsequent file stores the hash of its parent in its YAML front matter metadata.

### Journal file contents
Each new file should have YAML front matter with the following fields:
  - date: the date of the entry
  - time: the time of the entry
  - location: the location of the entry
  - provider: the provider of the entry
  - type: the type of the entry
  - tags: a list of tags for the entry

`.gitehr` contains hidden files including internal GitEHR config.

All other folders are for clinical information.

