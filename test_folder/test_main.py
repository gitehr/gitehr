from typer.testing import CliRunner

from ..main import app

runner = CliRunner()

def test_app():
    result = runner.invoke(app, ["docs"])
    print(result)