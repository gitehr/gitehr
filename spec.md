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

## Folder structure

`gitehr init` creates a folder structure inside the current directory. The folders are copied from the gitehr-folder-structure directory.

.gitehr contains hidden files including internal GitEHR config.

All other folders are for clinical information.
