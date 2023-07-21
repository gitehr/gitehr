from datetime import datetime

def get_current_datetime()->datetime:
    """Simple wrapper for datetime.now() to enable ease of testing."""
    return datetime.now()