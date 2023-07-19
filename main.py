import json
import os
from typing_extensions import Annotated

# 3RD PARTY
from git import Repo
import typer

# GITEHR IMPORTS
from utils.helper_functions import (
    get_iso_filename,
    get_current_datetime,
)
from utils import (
    RecordTypes,
    Record,
    Block,
    BlockChain,
    YAMLFrontmatter
)
from utils.constants import (
    meta_files,
)

app = typer.Typer()


def get_repo_url(repo_name: str) -> str:
    return os.path.join(os.getcwd(), repo_name)


def check_file_exists(FILE_URL:str) -> bool:
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
):
    """
    Creates a new GitEHR Repository.
    """
    BASE_URL = os.path.join(os.getcwd(), repo_name)
    REPO_URL = get_repo_url(repo_name)

    if not check_file_exists(BASE_URL):
        typer.secho(
            f"Creating new GitEHR Repository at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )

        Repo.init(BASE_URL)

    # Add first ROOT file
    FILE_PATH = f"{REPO_URL}/_ROOT.md"
    if not check_file_exists(FILE_PATH):
        typer.secho(
            f"Adding _ROOT.md file at {REPO_URL}...",
            fg=typer.colors.GREEN,
        )
        new_record = Record(
            contents=f"ROOT FILE FOR {repo_name}",
            meta_data={
                "created_on": get_current_datetime(),
            },
        )
        writer = RecordWriter(new_record, file_extension=".md")
        writer.write(filename=FILE_PATH)
        
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

    new_record.write_to_file()


@app.command()
def debug():
    
    record = Record(contents="Test entry")

    print("CONTENTS###")
    print(record.get_contents())
    print("CONTENTS###\n\n")
    # print(record.get_contents())
    print(record.generate_record_string())
    # print(record.generate_record_string())


if __name__ == "__main__":
    app()
