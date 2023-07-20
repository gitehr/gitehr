from datetime import datetime
from unittest.mock import patch

from utils.records import Record
from utils.yaml import YAMLFrontmatter
from test_folder import CONSTANTS_FOR_TESTS


@patch("utils.records.get_current_datetime", return_value=datetime(2023, 1, 1))
def test_default_meta_data_correct(mock_get_current_datetime):
    """Check default meta data for a new Record"""
    record = Record()
    expected_default_metadata = {
        "created_datetime": datetime(2023, 1, 1, 0, 0),
        "created_by": "PLACEHOLDER",
        "tags": ["ENCOUNTER"],
    }

    assert record.get_yaml_dict() == expected_default_metadata


@patch("utils.records.get_current_datetime", return_value=datetime(2023, 1, 1))
def test_get_set_filename_correct(mock_get_current_datetime):
    """Check filename for a new record"""

    record = Record()

    assert record.get_filename() == "20230101T000000"


def test_add_meta_data():
    """Check meta data added to .yaml dict attribute"""

    record = Record()
    INPUT_DICT = {"test_attribute": "test_value"}
    record.add_meta_data(**INPUT_DICT)

    assert ("test_attribute", "test_value") in record.get_yaml_dict().items()


def test_get_set_yaml():
    record = Record()

    yaml = YAMLFrontmatter({"test_attribute": "test_value"})

    record.set_yaml(yaml)

    assert record.get_yaml_dict() == {"test_attribute": "test_value"}


def test_get_contents():
    record = Record()

    assert record.get_contents() == ""


def test_get_hash():
    record = Record()
    record._set_hash("TESTHASH")

    assert record.get_hash() == "TESTHASH"
    assert ("hash", "TESTHASH") in record.get_yaml_dict().items()


def test_add_line_to_content():
    """Tests the add line method"""

    record = Record(contents="")

    record.add_line("NEW LINE")

    assert record.get_contents() == "NEW LINE\n"


def test_add_contents():
    """Tests adding multiple lines of content"""

    record = Record()

    CONTENTS_LIST = ["Line1", "Line2", "Line3\n"]
    record.add_contents(CONTENTS_LIST)

    assert record.get_contents() == "\nLine1\nLine2\nLine3\n\n"


@patch("utils.records.get_current_datetime", return_value=datetime(2023, 1, 1))
def test_generate_record_string_as_md(mocked_get_current_datetime):
    """Test the rendering of YAML, contents, public key as md"""

    record = Record()

    assert record.generate_record_string_as_md() == CONSTANTS_FOR_TESTS.EMPTY_DEFAULT_FILE_AS_MD_STRING
