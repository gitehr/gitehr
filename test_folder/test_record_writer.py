import os
from datetime import datetime
from unittest.mock import patch

from utils.records import Record, RecordWriter
from test_folder import CONSTANTS_FOR_TESTS

@patch("utils.records.get_current_datetime", return_value=datetime(2023, 1, 1))
def test_record_writer_outputs_file(mock_get_current_datetime, tmpdir):
    """Asserts a file with correct name is outputted"""
    
    record = Record()
    RecordWriter(record, directory=tmpdir).write()

    temp_dir_path = os.path.abspath(tmpdir)
    
    temp_dir_files = os.listdir(temp_dir_path)
    
    assert len(temp_dir_files) == 1
    assert temp_dir_files[0] == "20230101T000000.md"
    
@patch("utils.records.get_current_datetime", return_value=datetime(2023, 1, 1))
def test_record_writer_file_contents(mock_get_current_datetime, tmpdir):
    """Asserts the outputted file contains expected contents"""
    
    record = Record()
    RecordWriter(record, directory=tmpdir).write()

    temp_dir_path = os.path.abspath(tmpdir)
    outputted_file_path = os.path.join(temp_dir_path, '20230101T000000.md')
    
    
    with open(outputted_file_path, 'r') as f:
        contents = f.read()
    
    assert contents == CONSTANTS_FOR_TESTS.EMPTY_DEFAULT_FILE_AS_MD_STRING
    