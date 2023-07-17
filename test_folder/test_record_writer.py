from ..RecordWriter import RecordWriter


def test_clean_RW_empty():
    """Base RW instance should be empty string."""
    rw = RecordWriter()

    assert rw.get_contents() == ""


def test_rw_start_contents():
    """RW instantiated with contents should have only those contents."""

    TEST_CONTENTS = "Some test contents"
    rw = RecordWriter(TEST_CONTENTS)

    assert rw.get_contents() == TEST_CONTENTS


def test_rw_add_line():
    """Test RW with multiple calls to .add_line method returns expected output."""

    rw = RecordWriter()

    rw.add_line("Hello")
    rw.add_line("This is")
    rw.add_line("a test.")

    assert rw.get_contents() == "Hello\nThis is\na test.\n"
