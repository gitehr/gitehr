import json
import os
from typing_extensions import Annotated

# 3RD PARTY
from git import Repo
import typer

# GITEHR IMPORTS
from helper_functions import (
    get_iso_filename,
    get_current_datetime,
)
from utils import (
    RecordTypes,
    Record,
    RecordWriter,
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
    entry_contents: Annotated[str, typer.Argument(help="Contents of the entry.")],
    entry_type: Annotated[
        type(RecordTypes), typer.Option(parser=RecordTypes.parse_custom_class)
    ] = RecordTypes.ENCOUNTER.name,
):
    """Creates a new GitEHR record within the same directory.

    Args:
        `entry_contents` (str): Initial contents to add to created file.

        `entry_type` (RecordType, optional): Type of GitEHR Record to generate - determines file attributes.
    """

    with open("state.json", "r") as state_file:
        state = json.load(state_file)
        repo_name = state["repo_name"]

    typer.secho(
        f"Creating new {entry_type.name} Entry inside Record Directory {repo_name}",
        fg=typer.colors.GREEN,
    )

    current_datetime = get_current_datetime()
    FILENAME = get_iso_filename(current_datetime) + entry_type.file_type

    # TODO: add some way to automatically add this meta data to record
    meta_data = {
        "created_on": current_datetime,
        "created_by": "PLACEHOLDER",
        "tags": f"#{entry_type.name}",
    }

    new_record = Record(contents=entry_contents, meta_data=meta_data)

    writer = RecordWriter(new_record, file_extension=".md")

    writer.write(filename=FILENAME)


if __name__ == "__main__":
    app()
