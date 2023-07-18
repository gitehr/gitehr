from typing import Optional, Dict

from helper_functions import get_current_datetime


class RecordWriter:
    """
    Returns a markdown-formatted string, to be written to a GitEHR Record file.
    """

    def __init__(
        self, contents: str = "", separator: str = "\n", meta_data: dict = None
    ):
        self.contents = contents
        self.separator = separator
        self.meta_data = meta_data

        self.add_yaml_meta(self.meta_data)

    def add_yaml_meta(self, meta_data) -> None:
        """Adds initial meta data to frontmatter contents."""
        self._add_initial_frontmatter(meta_data)

    def _add_initial_frontmatter(self, data: dict) -> None:
        """Adds frontmatter to top of contents str."""

        frontmatter_str = self._create_frontmatter_str(data)

        # Appends frontmatter to beginning of contents str
        self.contents = "\n".join([frontmatter_str, self.contents])

    def _create_frontmatter_str(self, data: dict) -> str:
        """Method to create a YAML frontmatter string including the key-values provided."""

        initial_frontmatter = ["---", "---"]

        for key, val in data.items():
            initial_frontmatter.insert(-1, f"{key}:{val}")

        return "\n".join(initial_frontmatter)

    def add_line(self, content: str) -> None:
        self.contents += content + self.separator

    def add_contents(self, _contents_lst: list[str]) -> None:
        _contents_lst[0] = "\n" + _contents_lst[0]
        _contents_joined = "\n".join(_contents_lst)
        self.add_line(_contents_joined)

    def _add_public_key(self) -> None:
        self.add_contents(
            [
                "-----BEGIN PGP PUBLIC KEY BLOCK-----",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x",
                "=nUop",
                "-----END PGP PUBLIC KEY BLOCK-----",
            ]
        )

    def get_contents(self, sign=False) -> str:
        if sign:
            self._add_public_key()
        return self.contents

class YAMLFrontmatter:
    def __init__(self, meta_data:dict):
        self.meta_data = meta_data
    
    def _convert_meta_dict_to_list(self, meta_dict:dict)->list[str]:
        
        return [f"{key}:{val}" for key,val in meta_dict.items()]
    
    def _sandwich_dashes(self, meta_data_str_list:list[str])->list[str]:
        
        return ['---'] + meta_data_str_list + ['---']
    
    def _extract_yaml_from_string(self, input_string:str)->str:
        pass
    
    def get_string(self):

        meta_data_str_list = self._convert_meta_dict_to_list(self.meta_data)
        
        yaml_list = self._sandwich_dashes(meta_data_str_list)
        
        return "\n".join(yaml_list)


if __name__ == "__main__":
    
    yaml = YAMLFrontmatter({
                "created_on": get_current_datetime(),
                "created_by": "PLACEHOLDER",
            })
    
    print(yaml.get_string())
    
    
