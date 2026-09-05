import unittest
from types import SimpleNamespace

from discovery_chain import Rpc, digest, observe


class DiscoveryReceiptSecurityTests(unittest.TestCase):
    def test_rpc_credentials_cannot_be_redirected_to_cleartext_remote_hosts(self):
        for url in ["http://rpc.example", "file:///etc/passwd", "ftp://rpc.example"]:
            with self.assertRaises(ValueError):
                Rpc(url)

    def test_wrong_chain_or_unconfirmed_deposit_never_becomes_funding(self):
        args = SimpleNamespace(confirmations=12, transaction="0x" + "a"*64)
        with self.assertRaisesRegex(ValueError, "wrong chain"):
            observe(args, {"chain_id":31337}, {}, {}, lambda method,params: "0x1")
        receipt = {"status":"0x1","transactionHash":args.transaction,"blockNumber":"0xa"}
        values={"eth_chainId":"0x7a69","eth_getTransactionReceipt":receipt,"eth_blockNumber":"0xb"}
        with self.assertRaisesRegex(ValueError,"insufficient confirmations"):
            observe(args, {"chain_id":31337}, {}, {}, lambda method,params:values[method])

    def test_reorged_receipt_is_not_accepted_as_settled_funding(self):
        args=SimpleNamespace(confirmations=2,transaction="0x"+"a"*64)
        receipt={"status":"0x1","transactionHash":args.transaction,"blockNumber":"0xa","blockHash":"0x"+"b"*64}
        values={"eth_chainId":"0x7a69","eth_getTransactionReceipt":receipt,"eth_blockNumber":"0xb", "eth_getBlockByNumber":{"hash":"0x"+"c"*64}}
        with self.assertRaisesRegex(ValueError,"noncanonical block"):
            observe(args,{"chain_id":31337},{},{},lambda method,params:values[method])

    def test_malformed_digest_cannot_enter_abi_arguments(self):
        for value in ["0x1", "$(cat secret)", "xla:blake3:"+"z"*64]:
            with self.assertRaises(ValueError):
                digest(value)


if __name__ == "__main__":
    unittest.main()
