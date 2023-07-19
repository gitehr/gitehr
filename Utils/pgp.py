class PGPPublicKey:
    """
    Helper Class to allow extra functionality of Public PGP Keys within Records.

    Currently just returns a dummy pgp block string.
    """

    def __init__(self, public_key: str = None):
        self.public_key = public_key

    # TODO - remove once added functionality implemented
    def _TEMP_METHOD_GET_TEST_GPG_BLOCK(self) -> str:
        start = "-----BEGIN PGP PUBLIC KEY BLOCK-----"
        end = "-----END PGP PUBLIC KEY BLOCK-----"

        return "\n".join(
            [
                start,
                "THIS IS A FAKE EXAMPLE PGP PUBLIC KEY",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x=nUop",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x=nUop",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x=nUop",
                "mQINBFRUAGoBEACuk6ze2V2pZtScf1Ul25N2CX19AeL7sVYwnyrTYuWdG2FmJx4x=nUop",
                end,
            ]
        )
    
    def get_public_key(self):
        if self.get_public_key is None:
            return self._TEMP_METHOD_GET_TEST_GPG_BLOCK()
        return self.public_key


# DEBUGGING CODE
if __name__ == "__main__":
    key = PGPPublicKey()._TEMP_METHOD_GET_TEST_GPG_BLOCK()

    print(key)
