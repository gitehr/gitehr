import os

import pytest
from typer.testing import CliRunner

from ..main import app
from utils.records import Record, RecordReader

runner = CliRunner()


def test_hash_same_for_same_file(tmpdir):
    """Tests that, given the same input file, the output file has the same hash"""

    def setup_test():
        def initialise_repo():
            # Create a base repo with _ROOT.md file
            REPO_NAME = "TEMP_REPO"
            temp_dir_path = os.path.abspath(tmpdir)
            runner.invoke(app, ["init", REPO_NAME, "--repo-path", temp_dir_path])
            temp_repo_path = os.path.join(temp_dir_path, REPO_NAME)

            # Get root file hash
            root_file_path = os.path.join(temp_repo_path, "_ROOT.md")
            root_record = RecordReader().to_record(root_file_path)
            root_record_hash = root_record.get_yaml_dict()["hash"]
            return temp_repo_path

        temp_repo_path = initialise_repo()

        def get_hash_from_newly_created_record():
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
            os.remove(os.path.join(temp_repo_path, new_record_filename))

            return new_record_hash

        new_record_hash = get_hash_from_newly_created_record()
        duplicate_record_hash = get_hash_from_newly_created_record()

        return new_record_hash, duplicate_record_hash

    new_record_hash, duplicate_record_hash = setup_test()

    assert new_record_hash == duplicate_record_hash
