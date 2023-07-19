import re

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

    def __str__(self):
        return f"{self.get_meta_data()}"