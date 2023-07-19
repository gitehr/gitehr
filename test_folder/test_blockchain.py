import sys
print(sys.path)

from utils.blockchain import Block, BlockChain

def test_same_hash():
    """When exact same data is used, hash should be exactly same"""
    
    data = "Inserting some test data"
    
    block1 = Block(data=data)
    block2 = Block(data=data)
    
    assert block1.get_hash() == block2.get_hash()

def test_different_hash():
    """When different data used, hash should be different"""
    
    block1 = Block(data="Inserting some test data")
    block2 = Block(data="Inserting some test data.")
    
    assert block1.get_hash() != block2.get_hash()

def test_add_block():
    """Ensure adding blocks works"""
    
    blockchain = BlockChain()
    blockchain.add_block("Block 1")
    blockchain.add_block("Block 2")

    for block in blockchain.get_chain():
        print(f"{block.get_data()=}\n{block.get_hash()=}\n\n")
    assert len(blockchain.get_chain()) == 3