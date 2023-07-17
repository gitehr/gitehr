import shutil

from typer.testing import CliRunner

from ..main import app

runner = CliRunner()

def test_docs():
    """Tests `docs` argument"""
    result = runner.invoke(app, ["docs"])
    assert result.exit_code == 0

def test_init_runs():
    """Tests `init` argument creates a folder called Test"""
    
    TEST_DIR_NAME = 'TEMP_TEST_FOLDER'
    
    result = runner.invoke(app, ['init',TEST_DIR_NAME])
    
    assert result.exit_code == 0
    assert "Creating new GitEHR Repository at" in result.stdout
    
    # TEAR DOWN
    shutil.rmtree(TEST_DIR_NAME)

def test_init_correct_output_repo_already_exists():
    """Tests `init` doesn't add another folder if the Repo already exists."""
    
    TEST_DIR_NAME = 'TEMP_TEST_FOLDER'
    
    pass