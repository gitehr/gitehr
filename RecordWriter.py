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


if __name__ == "__main__":
    rw = RecordWriter(
        "Hi guys",
        meta_data={"created_on": get_current_datetime(), "created_by": "PLACEHOLDER"},
    )

    rw.add_contents(
        ["This is an entry for Anchit", "Anchit presented today", "Management is..."]
    )

    c = rw.get_contents(sign=True)
    
    print(c)
