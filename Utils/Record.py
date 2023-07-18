from .YAMLFrontmatter import YAMLFrontmatter

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