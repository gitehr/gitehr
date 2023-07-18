from ..RecordWriter import (
    RecordWriter,
    YAMLFrontmatter,
)


def test_YAMLFrontmatter_expected_output():
    """Tests the YAMLFrontmatter returns expected output for given input dict."""

    input_dict = {
        "created_on": "2022-10-01",
        "created_by": "PLACEHOLDER",
        "another_key": 20,
    }

    # create expected output
    expected_output_as_list = (
        ["---"] + [f"{key}:{val}" for key, val in input_dict.items()] + ["---"]
    )
    expected_output = "\n".join(expected_output_as_list)

    yaml = YAMLFrontmatter(input_dict)

    assert yaml.get_string() == expected_output

def test_YAMLFrontmatter_extracts_yaml_from_string_method():
    """Tests the YAMLFrontmatter method correctly extracts YAML content from input string"""
    
    pass