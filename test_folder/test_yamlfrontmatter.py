from ..RecordWriter import (
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
    
    input_str = """---
    created_on:2023-07-18
    created_by:PLACEHOLDER
    ---
    Hi guys
    This is an entry for Patient
    Patient presented today
    Management is...

    -----BEGIN PGP PUBLIC KEY BLOCK-----
    mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x
    =nUop
    -----END PGP PUBLIC KEY BLOCK-----
    """
    
    output = YAMLFrontmatter(input_str).extract_yaml_from_string(input_string=input_str)
    
    assert output == """---\ncreated_on:2023-07-18\ncreated_by:PLACEHOLDER\n---"""
    
def test_YAMLFrontmatter_extract_YAML_returns_None():
    """Tests the YAMLFrontmatter method correctly returns None input YAML content has no valid YAML"""
    
    input_str_1 = """Hi guys
    This is an entry for Patient
    Patient presented today
    Management is...

    -----BEGIN PGP PUBLIC KEY BLOCK-----
    mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x
    =nUop
    -----END PGP PUBLIC KEY BLOCK-----
    """
    input_str_2 = """--
    created_on:2023-07-18
    created_by:PLACEHOLDER
    ---"""
    input_str_3 = """---
    created_on:2023-07-18
    created_by:PLACEHOLDER
    --"""
    
    for input_string in [input_str_1,input_str_2,input_str_3]:
        
        output = YAMLFrontmatter().extract_yaml_from_string(input_string=input_string)
        
        assert output is None

def test_YAMLFrontmatter_is_None():
    
    yaml = YAMLFrontmatter()
    
    assert yaml.meta_data == None

def test_YAMLFrontmatter_add_items():
    """Tests the YAMLFrontmatter method correctly adds items"""

    yaml = YAMLFrontmatter()
    
    ITEMS_TO_ADD = {'NHS_number':'327189122','dob':'1997-18-10'}
    
    yaml.add_yaml_items(ITEMS_TO_ADD)
    
    assert yaml.meta_data == ITEMS_TO_ADD