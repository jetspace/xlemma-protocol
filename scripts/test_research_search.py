import copy
import hashlib
import json
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import unittest

from research_search import (ROOT, SCALE, build_index, canonical, digest, evaluate,
                             load_index, read_json, search, source_bytes)


CATALOG = ROOT / "examples/research-search/catalog.json"
CASES = ROOT / "examples/research-search/evaluation.json"


class ResearchSearchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = read_json(CATALOG)
        cls.index = build_index(cls.catalog)

    def test_permutations_preserve_index_and_rankings(self):
        before = canonical(self.catalog)
        expected = search(self.index, "midpoint dampng", seeds=["midpoint-conservation"])
        rng = random.Random(4337)
        for _ in range(24):
            reordered = copy.deepcopy(self.catalog)
            rng.shuffle(reordered["documents"])
            rng.shuffle(reordered["dependencies"])
            index = build_index(reordered)
            self.assertEqual(index, self.index)
            self.assertEqual(search(index, "midpoint dampng", seeds=["midpoint-conservation"]), expected)
        self.assertEqual(canonical(self.catalog), before)

    def test_source_drift_and_private_paths_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "lean").mkdir()
            path = root / "lean/example.lean"
            path.write_text("theorem example : True := True.intro\n")
            entry = copy.deepcopy(self.catalog["documents"][0])
            entry.update(source="lean/example.lean", source_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                         start_line=1, end_line=1)
            catalog = {**self.catalog, "documents": [entry], "dependencies": []}
            index = build_index(catalog, root)
            saved = root / "index.json"
            saved.write_bytes(canonical(index))
            self.assertEqual(load_index(saved, root), index)
            path.write_text("axiom changed : False\n")
            with self.assertRaisesRegex(ValueError, "source hash mismatch"):
                load_index(saved, root)
            for name in ["../secret.md", "/etc/passwd", "lean/../secret.md", "lean/.env",
                         "secrets/proof.lean", "lean/artifacts/private.lean", "lean//example.lean"]:
                with self.subTest(path=name), self.assertRaises(ValueError):
                    source_bytes(root, name)
            (root / "lean/alias.lean").symlink_to(path)
            with self.assertRaisesRegex(ValueError, "symlinks"):
                source_bytes(root, "lean/alias.lean")

    def test_index_tampering_cannot_change_rankings_silently(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "index.json"
            forged = copy.deepcopy(self.index)
            first = next(iter(forged["weights"]))
            forged["weights"][first] += 10_000
            forged["index_sha256"] = digest({k: v for k, v in forged.items() if k != "index_sha256"})
            path.write_bytes(canonical(forged))
            with self.assertRaisesRegex(ValueError, "stale or altered index"):
                load_index(path)

    def test_rejected_proof_is_opt_in_and_never_certified(self):
        query = "rejected false Euler conservation fixture"
        for mode in ["keyword", "vector", "hybrid"]:
            result = search(self.index, query, mode, 20)
            self.assertTrue(result["candidate_only"])
            self.assertNotIn("rejected-euler", [doc["document_id"] for doc in result["results"]])
        result = search(self.index, query, limit=20, include_rejected=True)
        rejected = next(doc for doc in result["results"] if doc["document_id"] == "rejected-euler")
        self.assertEqual(rejected["evidence_kind"], "rejected_proof")
        self.assertTrue(rejected["limitations"])
        self.assertNotIn("claim_id", rejected)
        self.assertNotIn("reward", rejected)

    def test_evidence_filter_cannot_promote_a_formal_result_to_measurement(self):
        for mode in ["keyword", "vector", "graph", "hybrid"]:
            result = search(self.index, "experimental oscillator damping", mode,
                            seeds=["midpoint-conservation"], evidence_kind="empirical_evidence")
            self.assertEqual(result["results"], [])

    def test_graph_expansion_is_declared_one_hop_even_with_cycles(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["dependencies"].extend([
            {"source": "midpoint-balance", "target": "midpoint-conservation", "kind": "declared_dependency"},
            {"source": "midpoint-balance", "target": "euler-growth", "kind": "declared_dependency"},
        ])
        index = build_index(catalog)
        result = search(index, "support", "graph", seeds=["midpoint-conservation"])
        self.assertEqual([doc["document_id"] for doc in result["results"]], ["midpoint-balance"])
        self.assertEqual(result["results"][0]["declared_dependency_of"], ["midpoint-conservation"])

    def test_unknown_links_and_duplicate_source_selections_are_rejected(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["dependencies"][0]["target"] = "unknown"
        with self.assertRaisesRegex(ValueError, "invalid dependency"):
            build_index(catalog)
        catalog = copy.deepcopy(self.catalog)
        duplicate = copy.deepcopy(catalog["documents"][0])
        duplicate["document_id"] = "copy"
        catalog["documents"].append(duplicate)
        with self.assertRaisesRegex(ValueError, "duplicate source selection"):
            build_index(catalog)

    def test_requests_are_bounded_and_outputs_do_not_mutate_the_index(self):
        for kwargs in [{"query": ""}, {"query": "x" * 2049}, {"query": "∀ →"},
                       {"query": "energy", "limit": True}, {"query": "energy", "limit": 21},
                       {"query": "energy", "seeds": ["unknown"]}]:
            with self.assertRaises(ValueError):
                search(self.index, **kwargs)
        before = canonical(self.index)
        for doc in self.index["documents"]:
            result = search(self.index, doc["title"], limit=20)
            self.assertNotIn("query", result)
            for candidate in result["results"]:
                self.assertGreaterEqual(candidate["vector_similarity_squared_ppm"], 0)
                self.assertLessEqual(candidate["vector_similarity_squared_ppm"], SCALE)
                candidate["assumptions"].append("mutated")
        self.assertEqual(canonical(self.index), before)

    def test_duplicate_json_keys_and_nonfinite_values_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            for text in ['{"x":1,"x":2}', '{"x":NaN}', '{"x":Infinity}']:
                path.write_text(text)
                with self.assertRaises(ValueError):
                    read_json(path)

    def test_exposed_evaluation_preserves_known_assumption_failure(self):
        cases = read_json(CASES)
        result = evaluate(self.index, cases)
        expected = read_json(ROOT / "reports/research-search-evaluation.json")
        self.assertEqual(result, expected)
        self.assertFalse(result["independent_holdout"])
        self.assertFalse(result["default_workflow_enabled"])
        self.assertFalse(result["research_acceleration_established"])
        self.assertGreater(result["summary"]["hybrid"]["forbidden_suggestions"], 0)
        cases["independent_holdout"] = True
        with self.assertRaises(ValueError):
            evaluate(self.index, cases)

    def test_cli_snapshot_search_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "index.json"
            def run(*args):
                return subprocess.run([sys.executable, str(ROOT / "scripts/research_search.py"), *args],
                                      capture_output=True, text=True, timeout=20)
            built = run("build", str(CATALOG), "--output", str(path))
            self.assertEqual(built.returncode, 0, built.stderr)
            before = path.read_bytes()
            again = run("build", str(CATALOG), "--output", str(path))
            self.assertNotEqual(again.returncode, 0)
            self.assertEqual(path.read_bytes(), before)
            result = run("search", str(path), "support", "--mode", "graph",
                         "--from-document", "midpoint-conservation")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout)["results"][0]["document_id"], "midpoint-balance")


if __name__ == "__main__":
    unittest.main()
