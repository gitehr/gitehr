import json
import os
from typing_extensions import Annotated

# 3RD PARTY
import typer

# GITEHR IMPORTS

from utils import RecordTypes, Record, RecordReader, InitialDir


app = typer.Typer()


def get_repo_url(repo_name: str, base_url: str = None) -> str:
    if base_url is None:
        base_url = os.getcwd()
    return os.path.join(base_url, repo_name)


def check_file_exists(FILE_URL: str) -> bool:
    repo_exists = os.path.exists(FILE_URL)

    if repo_exists:
        typer.secho(
            f"Already exists: {os.path.basename(FILE_URL)}",
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
    repo_name_or_path: Annotated[
        str, typer.Argument(help="Name or path of the GitEHR Repository folder.")
    ],
):
    """
    Creates a new GitEHR Repository.
    """

    gitehr_dir = InitialDir(repo_path=repo_name_or_path)

    REPO_URL = gitehr_dir.get_repo_path()

    if not check_file_exists(REPO_URL):
        typer.secho(
            f"Creating new GitEHR Repository at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        gitehr_dir.initialise_repo()

    # Add first ROOT file
    FILE_PATH = f"{REPO_URL}/_ROOT.md"
    if not check_file_exists(FILE_PATH):
        typer.secho(
            f"Creating _ROOT.md file at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        # Creates initial file inside directory
        gitehr_dir.add_first_root_file_to_repo()

    # Add JSON state file
    FILE_PATH = f"{REPO_URL}/state.json"
    if not check_file_exists(FILE_PATH):
        typer.secho(
            f"Adding state.json file at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        # Creates JSON state file inside directory
        gitehr_dir.add_state_file()


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

    new_record = Record(contents=entry_contents)

    new_record.write_to_file(file_name=new_record.get_filename())


@app.command()
def read_entry(filename: Annotated[str, typer.Argument(help="Name of Record to read")]):
    record = RecordReader().to_record(filename)

    typer.secho(
        f"Reading {record.filename} Entry...\n\nFound contents:",
        fg=typer.colors.GREEN,
    )

    print(record.generate_record_string_as_md())


@app.command()
def debug():
    InitialDir(repo_path="C:Anchit/Chandran/Repo")


if __name__ == "__main__":
    app()
