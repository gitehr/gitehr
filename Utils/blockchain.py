"""
Module containing blockchain code. 

Uses sha256 algorithm.
"""

from hashlib import sha256
from datetime import datetime


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

    def get_data(self)->str:
        return self.data

    def get_hash(self)->str:
        return self.hash

    def get_prev_hash(self)->str:
        return self.prev_hash


class BlockChain:
    def __init__(self):
        """
        Initialise block chain with genesis block.
        """
        genesis_block = self._generate_genesis_block()
        self.chain = [genesis_block]

    def _generate_genesis_block(self) -> Block:
        """
        Genesis block's hash should always be unique on each instantiation, as data to hash will be the current datetime.
        """
        block = Block(data=f"{str(datetime.now())}", prev_hash="0")
        return block

    def get_chain(self) -> list[Block]:
        return self.chain


if __name__ == "__main__":
    block_chain = BlockChain()
    print(block_chain.get_chain()[0].hash)
