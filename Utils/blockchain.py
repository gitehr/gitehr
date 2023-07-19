"""
Module containing blockchain code. 

Uses sha256 algorithm.
"""

from hashlib import sha256

from utils.helper_functions import get_current_datetime


class Block:
    def __init__(self, data: str, prev_hash: str = "0") -> None:
        self.data = data
        self.prev_hash = prev_hash
        self.hash = self._calculate_hash()

    def _calculate_hash(self) -> str:
        hash_algorithm = sha256()  # update this line if different algo to be used
        encoded_data = f"{self.data}{self.prev_hash}".encode("utf-8")
        hash_algorithm.update(encoded_data)
        return hash_algorithm.hexdigest()

    def get_data(self) -> str:
        return self.data

    def get_hash(self) -> str:
        return self.hash

    def get_prev_hash(self) -> str:
        return self.prev_hash


class BlockChain:
    def __init__(self, genesis_block_data:str=get_current_datetime()):
        """
        Initialise block chain with genesis block.
        """
        genesis_block = self._generate_genesis_block(genesis_block_data)
        self.chain = [genesis_block]

    def _generate_genesis_block(self, data) -> Block:
        """
        First block in chain
        """
        block = Block(data=data, prev_hash="0")
        return block

    def add_block(self, data: str) -> None:
        latest_block = self.chain[-1]

        new_block = Block(data=data, prev_hash=latest_block.hash)

        self.chain.append(new_block)

    def get_chain(self) -> list[Block]:
        return self.chain


if __name__ == "__main__":
    block_chain = BlockChain()
    block_chain.add_block("Block 1")
    block_chain.add_block("Block 2")
    print(block_chain.get_chain()[0].hash)
