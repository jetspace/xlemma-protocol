import tempfile
import unittest
from pathlib import Path

from source_inventory import source_files


class ReleaseInventoryTests(unittest.TestCase):
    def test_local_secrets_and_runtime_data_are_not_packaged(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            public = ["src/lib.rs", ".env.example", "README.md"]
            private = [".env", ".env.production", "nested/.env.local", "keys/node.key",
                       "keys/tls.pem", ".codex/config.toml", ".agents/notes.md",
                       "secrets/credentials.json", "artifacts/private-proof.lean", "api.log"]
            for name in public + private:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("test fixture")
            self.assertEqual({str(p.relative_to(root)) for p in source_files(root)}, set(public))

    def test_source_symlink_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source").write_text("source")
            (root / "alias").symlink_to(root / "source")
            with self.assertRaises(RuntimeError):
                source_files(root)


if __name__ == "__main__":
    unittest.main()
