from dataclasses import dataclass

@dataclass
class RecordType:
    name: str
    file_type: str=".txt"
    
@dataclass
class RecordTypes:
    ENCOUNTER = RecordType(name='ENCOUNTER')
    MEDICATIONS = RecordType(name='MEDICATIONS')
    ALLERGIES = RecordType(name='ALLERGIES')
    
    def parse_custom_class(self, value:str):
        return getattr(self, value)

