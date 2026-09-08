import unittest
import io
from unittest.mock import Mock
from types import SimpleNamespace

from discovery_chain import Rpc, digest, observe


class DiscoveryReceiptSecurityTests(unittest.TestCase):
    def test_rpc_failure_never_yields_a_result(self):
        for raw in [b'{', b'{"id":2,"result":"0x1"}',
                    b'{"id":1,"result":"0x1","error":{}}',
                    b'{"id":1}', b' ' * (2 * 1024 * 1024 + 1)]:
            with self.subTest(response=raw[:80]):
                rpc = Rpc("http://127.0.0.1:8545")
                rpc.opener = Mock()
                rpc.opener.open.return_value = io.BytesIO(raw)
                with self.assertRaises(ValueError):
                    rpc("eth_chainId", [])
                self.assertEqual(rpc.opener.open.call_args.kwargs["timeout"], 15)
        rpc.opener.open.side_effect = TimeoutError("fixture timeout")
        with self.assertRaises(TimeoutError):
            rpc("eth_chainId", [])

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
