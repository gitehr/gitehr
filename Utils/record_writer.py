# Python Imports

# GitEHR Imports
from helper_functions import get_current_datetime
from . import Record

class RecordWriter:
    """Takes a Record object and writes to file."""
    
    def __init__(self, record_obj:Record, file_extension:str='.md'):
        self.file_extension=file_extension
        self.record = record_obj
    
    def write(self, filename)->None:
        """Takes Record object's contents and writes to PATH: {destination_path}/{filename}"""

        with open(filename, "w") as entry_file:
            
            contents=self.record.get_contents()

            entry_file.write(contents)
        

if __name__ == "__main__":
    new_entry = Record(
        "This is a new entry for Patient A.\nHe presented with dyspnoea.\nManagement is xyz.",
        meta_data={"created_on": get_current_datetime(), "created_by": "Dr AC"},
    )

    print(new_entry.get_contents())
