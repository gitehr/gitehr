# Python Imports
from typing import Optional, Dict
import re

# GitEHR Imports
from helper_functions import get_current_datetime

class Record:
    """
    Class to create a GitEHR markdown record including YAML, contents, and PGP signing.
    """

    def __init__(
        self, contents: str = "", separator: str = "\n", meta_data: dict = None
    ):
        self.contents = contents
        self.separator = separator
        self.meta_data = meta_data

    def add_line(self, content: str) -> None:
        self.contents += content + self.separator

    def add_contents(self, _contents_lst: list[str]) -> None:
        _contents_lst[0] = "\n" + _contents_lst[0]
        _contents_joined = "\n".join(_contents_lst)
        self.add_line(_contents_joined)

    def _prepend_yaml(self) -> None:
        yaml = YAMLFrontmatter(self.meta_data)
        yaml_string = yaml.get_string()
        self.contents = "\n".join([yaml_string, self.contents])

    def _add_public_key(self) -> None:
        self.add_contents(
            [
                "-----BEGIN PGP PUBLIC KEY BLOCK-----",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x",
                "=nUop",
                "-----END PGP PUBLIC KEY BLOCK-----",
            ]
        )

    def get_contents(self, sign=True) -> str:
        self._prepend_yaml()

        if sign:
            self._add_public_key()

        return self.contents

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
        

class YAMLFrontmatter:
    """Helper Class to aid interaction with YAML.

    Instantiate with a dictionary of meta data items.

    Useful methods:
        - `.add_yaml_items()` -> takes a dictionary of meta data items to add.
        - `.get_string()` -> turns current contents into a YAML string
        - `.extract_yaml_from_string()` -> searches input string and returns YAML contents, or None if one can't be found.
        - `.get_meta_data()` -> getter
    """

    def __init__(self, meta_data: dict = None):
        self.meta_data = meta_data

    def get_meta_data(self) -> dict:
        return self.meta_data

    def _convert_meta_dict_to_list(self, meta_dict: dict) -> list[str]:
        return [f"{key}:{val}" for key, val in meta_dict.items()]

    def _sandwich_dashes(self, meta_data_str_list: list[str]) -> list[str]:
        return ["---"] + meta_data_str_list + ["---"]

    def extract_yaml_from_string(self, input_string: str) -> str:
        matches = re.search(r"---\n(([\s|\S])*)(?<!-)---(?=\n)", input_string)
        if matches:
            return matches.group(0).replace("\n    ", "\n")

    def add_yaml_items(self, items_to_add: dict) -> None:
        if self.meta_data is not None:
            self.meta_data.update(items_to_add)
        else:
            self.meta_data = items_to_add

    def get_string(self):
        meta_data_str_list = self._convert_meta_dict_to_list(self.meta_data)

        yaml_list = self._sandwich_dashes(meta_data_str_list)

        return "\n".join(yaml_list)


if __name__ == "__main__":
    new_entry = Record(
        "This is a new entry for Patient A.\nHe presented with dyspnoea.\nManagement is xyz.",
        meta_data={"created_on": get_current_datetime(), "created_by": "Dr AC"},
    )

    print(new_entry.get_contents())
