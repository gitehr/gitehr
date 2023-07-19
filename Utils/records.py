# Python Imports

# GitEHR Imports
from .yaml import YAMLFrontmatter
from .pgp import PGPPublicKey
from utils.helper_functions import (
    get_iso_filename,
    get_current_datetime,
)
from utils.record_types import RecordTypes


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
            "current_datetime": current_datetime,
            "created_by": "PLACEHOLDER",
            "tags": [f"{record_type}"],
        }
        self._add_default_meta_data(**DEFAULT_META_DATA)

        # create filename
        self.filename = get_iso_filename(current_datetime)

    def _add_default_meta_data(self, **default_meta_data) -> None:
        """
        Adds **default_meta_data kwargs to object's yaml.
        """
        self.yaml.add_yaml_items(default_meta_data)

    def add_line(self, content: str) -> None:
        self.contents += content + self.separator

    def add_contents(self, _contents_lst: list[str]) -> None:
        _contents_lst[0] = "\n" + _contents_lst[0]
        _contents_joined = "\n".join(_contents_lst)
        self.add_line(_contents_joined)

    def get_formatted_public_key_string(self) -> str:
        return self.public_key._TEMP_METHOD_GET_TEST_GPG_BLOCK()

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

    def get_contents(self) -> str:
        return self.contents

    def get_filename(self) -> str:
        return self.filename

    def write_to_file(self, file_extension=".md"):
        RecordWriter(self, file_extension=file_extension).write()

    def __str__(self):
        return f"{self.filename}"


class RecordWriter:
    """Takes a Record object and writes to file."""

    def __init__(self, record_obj: Record, file_extension: str = ".md"):
        self.file_extension = file_extension
        self.record = record_obj
        self.filename = f"{self.record.get_filename()}{file_extension}"

    def write(self) -> None:
        """Takes Record object's contents and writes to file {filename}"""

        with open(self.filename, "w") as entry_file:
            contents = self.record.generate_record_string_as_md()

            entry_file.write(contents)
