import json
import os
from typing_extensions import Annotated

# 3RD PARTY
from git import Repo
import typer

# GITEHR IMPORTS

from utils import RecordTypes, Record, RecordReader, InitialRecord


app = typer.Typer()


def get_repo_url(repo_name: str, base_url:str=None) -> str:
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
    repo_name: Annotated[
        str, typer.Argument(help="Name of the GitEHR Repository folder.")
    ],
    repo_path: Annotated[
        str, typer.Option(help="Optionally specify output path of repo")
    ]=None
):
    """
    Creates a new GitEHR Repository.
    """
    BASE_URL = os.getcwd() if repo_path is None else repo_path
    REPO_URL = get_repo_url(repo_name, base_url=BASE_URL)
    
    print(f"{REPO_URL=}")

    if not check_file_exists(REPO_URL):
        typer.secho(
            f"Creating new GitEHR Repository at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        Repo.init(REPO_URL)

    # Add first ROOT file
    FILE_PATH = f"{REPO_URL}/_ROOT.md"
    if not check_file_exists(FILE_PATH):
        typer.secho(
            f"Creating _ROOT.md file at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )
        
        # Creates initial file inside directory
        InitialRecord(repo_name, repo_directory=REPO_URL)

    # Add JSON state file
    FILE_PATH = f"{REPO_URL}/state.json"
    if not check_file_exists(FILE_PATH):
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

    new_record = Record(contents=entry_contents)

    new_record.write_to_file(file_name=new_record.get_filename())

@app.command()
def read_entry(filename:Annotated[str, typer.Argument(help="Name of Record to read")]):
    
    record = RecordReader().to_record(filename)
    
    typer.secho(
        f"Reading {record.filename} Entry...\n\nFound contents:",
        fg=typer.colors.GREEN,
    )
    
    print(record.generate_record_string_as_md())
    

@app.command()
def debug():
    
    record = Record()
    record.write_to_file()


if __name__ == "__main__":
    app()
