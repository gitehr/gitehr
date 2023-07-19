# Python Imports
import os

# GitEHR Imports
from utils.records import Record

class RecordWriter:
    """Takes a Record object and writes to file."""
    
    def __init__(self, record_obj:Record, file_extension:str='.md'):
        self.file_extension=file_extension
        self.record = record_obj
    
    def write(self, filename)->None:
        """Takes Record object's contents and writes to file {filename}"""

        with open(filename, "w") as entry_file:
            
            contents=self.record.get_contents()

            entry_file.write(contents)
        

if __name__ == "__main__":
    
    
    current_files = os.listdir()
    
    print(current_files)
