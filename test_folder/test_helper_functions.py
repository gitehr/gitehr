from datetime import datetime

from utils.helper_functions import get_iso_filename

def test_get_iso_filename():
    
    INPUT = datetime(2023,1,1) # 2023-01-01 00:00:00
    EXPECTED_OUTPUT_ISO = "20230101T000000"
    
    assert get_iso_filename(INPUT) == EXPECTED_OUTPUT_ISO