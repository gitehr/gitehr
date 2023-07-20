import os

import pytest
from typer.testing import CliRunner

from ..main import app

runner = CliRunner()

@pytest.mark.skip(reason="actually opens an internet link which can be annoying during local dev")
def test_docs():
    """Tests `docs` argument"""
    result = runner.invoke(app, ["docs"])
    assert result.exit_code == 0


def test_init_runs(tmpdir):
    """Tests `init` command creates a folder called Test"""
    
    REPO_NAME = 'TEMP_REPO'
    temp_dir_path = os.path.abspath(tmpdir)
    
    result = runner.invoke(app, ['init',REPO_NAME, "--repo-path", temp_dir_path])

    assert result.exit_code == 0
    assert f"Creating new GitEHR Repository at {temp_dir_path}" in result.stdout
    


def test_init_correct_output_repo_already_exists():
    """Tests `init` doesn't add another folder if the Repo already exists."""
    
    TEST_DIR_NAME = 'TEMP_TEST_FOLDER'
    
    pass