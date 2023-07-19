import sys
print(sys.path)

from utils.blockchain import Block

def test_block_same_data_same_hash():
    """When exact same data is used, hash should be exactly same"""
    
    data = "Inserting some test data"
    
    block1 = Block(data=data)
    block2 = Block(data=data)
    
    assert block1.get_hash() == block2.get_hash()
    