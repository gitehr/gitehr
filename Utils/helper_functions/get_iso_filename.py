from datetime import datetime

from .get_current_datetime import get_current_datetime

def get_iso_filename(current_datetime:datetime = get_current_datetime())->str:
    """
    Returns a filename, as string, created from the current ISO datetime at which the function is called, with symbols ":.-" removed.
    """
    
    # Convert datetime to ISO string
    iso_string = current_datetime.isoformat()
    
    # Modify the ISO string to make it suitable for a filename
    FILENAME = iso_string.replace(':', '').replace('.', '').replace('-','')
    
    return FILENAME