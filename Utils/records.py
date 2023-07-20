# Python Imports
import os

# GitEHR Imports
from .yaml import YAMLFrontmatter
from .pgp import PGPPublicKey
from utils.helper_functions import (
    get_iso_filename,
    get_current_datetime,
)
from utils.constants import meta_files
from utils.record_types import RecordTypes
from utils.blockchain import Block


class Record:
    """
    Class to create a GitEHR markdown record including YAML, contents, and PGP signing.
    """

    def __init__(
        self,
        contents: str = "",
        separator: str = "\n",
        meta_data: YAMLFrontmatter = YAMLFrontmatter(),
        public_key: PGPPublicKey = PGPPublicKey(),
        record_type: str = RecordTypes.ENCOUNTER.name,
    ):
        self.contents = contents
        self.separator = separator
        self.yaml = meta_data
        self.public_key = public_key
        self.record_type = record_type

        # to be used for metadata and filename attributes
        current_datetime = get_current_datetime()

        # add default meta data here
        DEFAULT_META_DATA = {
            "created_datetime": current_datetime,
            "created_by": "PLACEHOLDER",
            "tags": [f"{record_type}"],
        }
        self.add_meta_data(**DEFAULT_META_DATA)

        # create filename
        self.filename = get_iso_filename(current_datetime)

    def add_meta_data(self, **meta_data) -> None:
        """
        Adds **meta_data kwargs to object's yaml.
        """
        self.yaml.add_yaml_items(meta_data)

    def set_yaml(self, new_yaml: YAMLFrontmatter) -> None:
        self.yaml = new_yaml

    def add_line(self, content: str) -> None:
        self.contents += content + self.separator

    def add_contents(self, _contents_lst: list[str]) -> None:
        _contents_lst[0] = "\n" + _contents_lst[0]
        _contents_joined = "\n".join(_contents_lst)
        self.add_line(_contents_joined)

    def get_formatted_public_key_string(self) -> str:
        return self.public_key.get_public_key()

    def generate_record_string_as_md(self) -> str:
        yaml = self.yaml.get_string()
        contents = self.contents
        key = self.get_formatted_public_key_string()
        return (
            "\n\n".join(
                [
                    yaml,
                    contents,
                    key,
                ]
            )
            + "\n"
        )

    def _set_hash(self, hash_value: str) -> None:
        """Sets self.hash attribute AND adds to YAML meta data"""
        self.hash = hash_value
        self.add_meta_data(hash=self.hash)

    def get_contents(self) -> str:
        return self.contents

    def get_filename(self) -> str:
        return self.filename

    def get_yaml_dict(self) -> dict:
        return self.yaml.get_meta_data()

    def get_hash(self) -> str:
        return self.hash
    
    def _create_initial_file(self, repo_name:str) -> None:
        """Creates initial file inside Repo. Should only be run once."""
        
        # GENERATE HASH FOR THIS FILE USING PREVIOUS FILE'S CONTENTS
        new_hash = Block(data=self.generate_record_string_as_md()).get_hash()

        # SET THE HASH
        self._set_hash(new_hash)

        RecordWriter(
            record_obj=self,
            directory=repo_name,
            file_name="_ROOT",
            file_extension='.md',
        ).write()

    def write_to_file(
        self, directory: str = None, file_name: str = None, file_extension=".md"
    )->None:
        """
        Writes current Record's contents to file.

        First gets the contents of the most recent file, hashes it, and sets this Record's YAML data relating to hash and prev_hash.
        """
        
        current_records_in_dir = [
            file for file in os.listdir() if file not in meta_files.META_FILES
        ]
        

        # ONLY INIT RECORD PRESENT
        if len(current_records_in_dir) == 1:
            HEAD_FILENAME = current_records_in_dir[0]

        else:
            sorted_records = sorted(current_records_in_dir)
            HEAD_FILENAME = sorted_records[-2]

        # Find previous file's YAML for this file's hash
        head_record = RecordReader().to_record(filepath=HEAD_FILENAME)
        head_record_yaml = head_record.get_yaml_dict()

        # ADD PREV_HASH TO THIS YAML
        self.add_meta_data(prev_hash=head_record_yaml["hash"])

        # GENERATE HASH FOR THIS FILE USING PREVIOUS FILE'S CONTENTS
        new_hash = Block(data=head_record.generate_record_string_as_md()).get_hash()

        # SET THE HASH
        self._set_hash(new_hash)

        RecordWriter(
            record_obj=self,
            directory=directory,
            file_extension=file_extension,
            file_name=file_name,
        ).write()

    def __str__(self):
        return f"{self.filename}"


class InitialRecord:
    """Creates initial _ROOT.md record at given repo_name."""

    def __init__(self, repo_name):
        self.repo_name = repo_name

        init_record = self._create_init_record()

        init_record._create_initial_file(repo_name=repo_name)

    def _create_init_record(self) -> Record:
        return Record(
            contents=f"ROOT FILE FOR {self.repo_name}",
            meta_data=YAMLFrontmatter({"prev_hash": "0"}),
        )


class RecordReader:
    """Takes in a text file, to generate a Record object."""

    def __init__(self):
        pass

    def to_record(self, filepath: str) -> Record:
        with open(filepath, "r") as file:
            file_contents = file.readlines()

        # get yaml -> input to YAMLFrontmatter() requires dict
        # first line should always be YAML start string: "---"
        yaml_end_idx = file_contents[1:].index("---\n") + 1
        yaml_dict = self._get_yaml_dict_from_list(file_contents[1:yaml_end_idx])
        yaml = YAMLFrontmatter(yaml_dict)

        # get contents
        start_contents_idx = yaml_end_idx + 2
        end_contents_idx = (
            file_contents.index("-----BEGIN PGP PUBLIC KEY BLOCK-----\n") - 1
        )
        contents = "".join(file_contents[start_contents_idx:end_contents_idx]).strip(
            "\n"
        )

        # get signature
        start_signature_idx = end_contents_idx + 1
        signature_str = "".join(file_contents[start_signature_idx:]).strip("\n")
        public_key = PGPPublicKey(signature_str)

        record = Record(
            contents=contents,
            public_key=public_key,
        )
        record.set_yaml(yaml)

        return record

    def _get_yaml_dict_from_list(self, yaml_contents_list: list) -> dict:
        yaml_dict = {}
        for content in yaml_contents_list:
            content = content.strip("\n")

            # Can't just split on ":" as would break datetime items
            first_colon = content.find(":")
            key, val = content[:first_colon], content[first_colon + 1 :]
            yaml_dict.update({key: val})
        return yaml_dict


class RecordWriter:
    """Takes a Record object and writes to file."""

    def __init__(
        self,
        record_obj: Record,
        directory: str = None,
        file_name: str = None,
        file_extension: str = ".md",
    ):
        self.file_extension = file_extension
        self.record = record_obj
        self.filename = (
            f"{self.record.get_filename()}{file_extension}"
            if not file_name
            else f"{file_name}{file_extension}"
        )
        self.directory = directory

    def write(self) -> None:
        """Takes Record object's contents and writes to file {filename}"""
        FILE_PATH = (
            f"{self.directory}/{self.filename}" if self.directory else self.filename
        )
        with open(FILE_PATH, "w") as entry_file:
            contents = self.record.generate_record_string_as_md()

            entry_file.write(contents)
