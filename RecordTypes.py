from enum import Enum

class RecordType(str, Enum):
    ENCOUNTER = 'ENCOUNTER'
    MEDICATIONS = 'MEDICATIONS'
    ALLERGIES = 'ALLERGIES'
