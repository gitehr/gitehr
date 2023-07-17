from datetime import datetime

def get_iso_filename()->str:
    """
    Returns a filename, as string, created from the current ISO datetime at which the function is called, with symbols ":.-" removed.
    """
    
    current_datetime = datetime.now()
    
    # Convert datetime to ISO string
    iso_string = current_datetime.isoformat()
    
    # Modify the ISO string to make it suitable for a filename
    FILENAME = iso_string.replace(':', '').replace('.', '').replace('-','')
    
    return FILENAME