import os

import pytest
from typer.testing import CliRunner

from ..main import app
from utils.records import Record, RecordReader

runner = CliRunner()


def initialise_repo(tmpdir):
    # Create a base repo with _ROOT.md file
    REPO_NAME = "TEMP_REPO"
    temp_dir_path = os.path.abspath(tmpdir)

    runner.invoke(app, ["init", REPO_NAME, "--repo-path", temp_dir_path])

    temp_repo_path = os.path.join(temp_dir_path, REPO_NAME)

    return temp_repo_path


def get_hash_from_newly_created_record(
    temp_repo_path,
    delete_file: bool = True,
    file_contents: str = "Test file contents",
):
    # Create new file
    os.chdir(temp_repo_path)
    runner.invoke(app, ["create-entry", "Test file contents"])

    # Get hash of new Record
    new_record_filename = ""
    for filename in os.listdir(temp_repo_path):
        if filename[0].isdigit():
            new_record_filename = filename
            break
    new_record = RecordReader().to_record(new_record_filename)
    new_record_hash = new_record.get_yaml_dict()["hash"]

    # Delete file
    if delete_file:
        os.remove(os.path.join(temp_repo_path, new_record_filename))

    return new_record_hash


def test_hash_same_for_same_file(tmpdir):
    """Tests that, given the same input file, the output file has the same hash"""

    def setup_test():
        temp_repo_path = initialise_repo(tmpdir)

        new_record_hash = get_hash_from_newly_created_record(
            temp_repo_path, delete_file=True
        )
        duplicate_record_hash = get_hash_from_newly_created_record(
            temp_repo_path, delete_file=True
        )

        return new_record_hash, duplicate_record_hash

    new_record_hash, duplicate_record_hash = setup_test()

    assert new_record_hash == duplicate_record_hash


def test_hash_diff_for_diff_file(tmpdir):
    """Tests that, given an input file that is different by 1 character, the output file has a diff hash"""

    def setup_test():
        temp_repo_path = initialise_repo(tmpdir)

        _ = get_hash_from_newly_created_record(
            temp_repo_path, delete_file=False
        )
        duplicate_1_hash = get_hash_from_newly_created_record(
            temp_repo_path, delete_file=True, file_contents='Test'
        )
        duplicate_2_hash = get_hash_from_newly_created_record(
            temp_repo_path, delete_file=True, file_contents='Test ' # note extra whitespace char
        )

        return duplicate_1_hash, duplicate_2_hash

    duplicate_1_hash, duplicate_2_hash = setup_test()

    assert duplicate_1_hash != duplicate_2_hash
