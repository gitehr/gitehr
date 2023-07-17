import json
import os
from typing_extensions import Annotated

# 3RD PARTY
import typer
from git import Repo

# GITEHR IMPORTS
from RecordTypes import RecordType
from helper_functions import (
    get_iso_filename,
)

app = typer.Typer()


def get_repo_url(repo_name: str) -> str:
    return os.path.join(os.getcwd(), repo_name)


def check_repo_exists(REPO_URL) -> bool:
    repo_exists = os.path.exists(REPO_URL)

    if repo_exists:
        typer.secho(
            f"Looks like the repo already exists at {REPO_URL}",
            fg=typer.colors.MAGENTA,
        )

    return repo_exists


@app.command()
def docs():
    """
    Opens the Git EHR Documentation website at https://gitehr.org/.
    """
    typer.launch("https://gitehr.org/")


@app.command()
def init(
    repo_name: Annotated[
        str, typer.Argument(help="Name of the GitEHR Repository folder.")
    ],
):
    """
    Creates a new GitEHR Repository.
    """
    BASE_URL = os.path.join(os.getcwd(), repo_name)
    REPO_URL = get_repo_url(repo_name)

    if not check_repo_exists(REPO_URL=BASE_URL):
        typer.secho(
            f"Creating new GitEHR Repository at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        Repo.init(BASE_URL)

    # Add JSON state file
    FILE_PATH = f"{REPO_URL}/state.json"
    if not os.path.exists(FILE_PATH):
        typer.secho(
            f"Adding state.json file at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        with open(FILE_PATH, "w") as json_file:
            json.dump({"repo_name": repo_name}, json_file)


@app.command()
def create_entry(
    entry_type: RecordType = RecordType.ENCOUNTER,
):
    """
    Adds new entry in GitEHR Repository.
    """
    
    with open("state.json", "r") as state_file:
        state = json.load(state_file)
        repo_name = state["repo_name"]
    
    
    typer.secho(
        f"Creating new {entry_type.value} Entry inside Record {state['repo_name']}",
        fg=typer.colors.GREEN,
    )
    
    FILENAME = get_iso_filename()

    with open(f"{FILENAME}.txt", "w") as entry_file:
        entry_file.write(f"Entry {FILENAME} created inside {state['repo_name']}")


if __name__ == "__main__":
    app()
