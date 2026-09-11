#!/usr/bin/env python3
"""Local candidate retrieval. Scores never establish validity, novelty or rewards."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import statistics
import time

from source_inventory import EXCLUDED_PARTS

ROOT = Path(__file__).resolve().parents[1]
VERSION = "xlemma-sparse-retrieval-v1"
CATALOG_VERSION = "xlemma-research-catalog-v1"
SCALE = 1_000_000
MAX_JSON = 8 * 1024 * 1024
KINDS = {"formal_example", "computational_example", "design_note", "rejected_proof", "empirical_evidence"}
MODES = ("keyword", "vector", "graph", "hybrid")


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def exact_keys(value, keys):
    require(isinstance(value, dict) and set(value) == set(keys), "unexpected or missing fields")


def integer(value, minimum, maximum):
    return type(value) is int and minimum <= value <= maximum


def bounded_text(value, maximum=4096):
    return isinstance(value, str) and 0 < len(value) <= maximum


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON field")
            result[key] = value
        return result
    with Path(path).open("rb") as stream:
        raw = stream.read(MAX_JSON + 1)
    require(len(raw) <= MAX_JSON, "JSON input exceeds bound")
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda _: require(False, "nonfinite JSON value"))


def source_bytes(root, name):
    require(bounded_text(name, 512), "invalid source path")
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and relative.as_posix() == name
            and relative.parts and relative.parts[0] in {"lean", "docs", "spec", "examples", "crates", "scripts"}
            and all(p not in EXCLUDED_PARTS and not p.startswith(".") for p in relative.parts)
            and relative.suffix in {".md", ".lean", ".rs", ".py"}, "source is outside public source paths")
    path = Path(root)
    for part in relative.parts:
        path = path / part
        require(not path.is_symlink(), "source symlinks are not permitted")
    require(path.is_file(), "source must be a regular file")
    with path.open("rb") as stream:
        raw = stream.read(256 * 1024 + 1)
    require(len(raw) <= 256 * 1024, "source exceeds bound")
    return raw


def tokens(text):
    # Deliberately portable ASCII lexical baseline; mathematical symbols remain
    # in source excerpts, not interpreted or equated by this tokenizer.
    return sorted(set(word.lower() for word in re.findall(r"[A-Za-z0-9]+", text)))


def features(words):
    # Binary within-word character trigrams are lexical features, not learned embeddings.
    result = set()
    for word in words:
        if len(word) < 3:
            continue
        padded = "^" + word + "$"
        result.update(padded[i:i + 3] for i in range(len(padded) - 2))
    return sorted(result)


def build_index(catalog, root=ROOT):
    exact_keys(catalog, ["version", "visibility", "documents", "dependencies"])
    require(catalog["version"] == CATALOG_VERSION and catalog["visibility"] == "public", "unsupported catalog")
    require(isinstance(catalog["documents"], list) and 1 <= len(catalog["documents"]) <= 64, "invalid corpus size")
    require(isinstance(catalog["dependencies"], list) and len(catalog["dependencies"]) <= 256, "too many dependencies")
    docs, ids, selections = [], set(), set()
    for entry in catalog["documents"]:
        exact_keys(entry, ["document_id", "title", "summary", "source", "source_sha256", "start_line",
                           "end_line", "evidence_kind", "assumptions", "limitations"])
        doc_id = entry["document_id"]
        require(isinstance(doc_id, str) and re.fullmatch(r"[a-z][a-z0-9-]{0,63}", doc_id)
                and doc_id not in ids, "invalid or duplicate document ID")
        ids.add(doc_id)
        require(bounded_text(entry["title"], 256) and bounded_text(entry["summary"]), "invalid description")
        require(entry["evidence_kind"] in KINDS, "unsupported evidence kind")
        for key in ["assumptions", "limitations"]:
            require(isinstance(entry[key], list) and 1 <= len(entry[key]) <= 16
                    and all(bounded_text(item, 512) for item in entry[key]), "invalid scientific metadata")
        require(integer(entry["start_line"], 1, 100000) and integer(entry["end_line"], entry["start_line"], 100000),
                "invalid source range")
        raw = source_bytes(root, entry["source"])
        require(hashlib.sha256(raw).hexdigest() == entry["source_sha256"], "source hash mismatch; review the catalog")
        lines = raw.decode("utf-8").splitlines()
        require(entry["end_line"] <= len(lines), "source range exceeds file")
        selection = (entry["source"], entry["start_line"], entry["end_line"])
        require(selection not in selections, "duplicate source selection")
        selections.add(selection)
        excerpt = "\n".join(lines[entry["start_line"] - 1:entry["end_line"]])
        require(0 < len(excerpt.encode()) <= 32768, "source excerpt exceeds bound or is empty")
        searchable = " ".join([entry["title"], entry["summary"], excerpt, *entry["assumptions"], *entry["limitations"]])
        words = tokens(searchable)
        docs.append({**entry, "excerpt": excerpt, "terms": words, "features": features(words)})
    edges = []
    for edge in catalog["dependencies"]:
        exact_keys(edge, ["source", "target", "kind"])
        require(edge["kind"] == "declared_dependency" and edge["source"] in ids
                and edge["target"] in ids and edge["source"] != edge["target"], "invalid dependency")
        require(edge not in edges, "duplicate dependency")
        edges.append(edge)
    docs.sort(key=lambda item: item["document_id"])
    frequency = Counter(feature for doc in docs for feature in doc["features"])
    # Integer inverse-frequency weights and squared cosine avoid platform float ordering.
    weights = {key: (len(docs) + 1) * 1024 // (count + 1) for key, count in sorted(frequency.items())}
    payload = {"version": VERSION, "tokenizer": "ascii-identifiers-v1",
               "implementation_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
               "catalog": {**catalog, "documents": sorted(catalog["documents"], key=lambda x: x["document_id"]),
                           "dependencies": sorted(edges, key=lambda x: (x["source"], x["target"]))},
               "documents": docs, "weights": weights}
    result = {**payload, "index_sha256": digest(payload)}
    require(len(canonical(result)) <= MAX_JSON, "index exceeds size bound")
    return result


def load_index(path, root=ROOT):
    index = read_json(path)
    exact_keys(index, ["version", "tokenizer", "implementation_sha256", "catalog", "documents", "weights", "index_sha256"])
    require(index["version"] == VERSION and index["tokenizer"] == "ascii-identifiers-v1",
            "unsupported index or tokenizer; rebuild explicitly")
    rebuilt = build_index(index["catalog"], root)
    require(canonical(index) == canonical(rebuilt), "stale or altered index")
    return rebuilt


def search(index, query, mode="hybrid", limit=5, seeds=(), evidence_kind=None, include_rejected=False):
    require(bounded_text(query, 2048) and mode in MODES and integer(limit, 1, 20), "invalid search request")
    require(evidence_kind is None or evidence_kind in KINDS, "invalid evidence filter")
    require(isinstance(seeds, (tuple, list)) and len(seeds) <= 8 and len(set(seeds)) == len(seeds), "invalid graph seeds")
    doc_ids = {doc["document_id"] for doc in index["documents"]}
    require(all(seed in doc_ids for seed in seeds), "unknown graph seed")
    query_terms = set(tokens(query))
    use_vectors = mode in {"vector", "hybrid"}
    query_features = set(features(query_terms)) if use_vectors else set()
    require(query_terms, "query has no searchable terms")
    weights = index["weights"]
    # Unseen query features retain weight, penalizing unrelated long queries.
    unseen_weight = (len(index["documents"]) + 1) * 1024
    qnorm = sum(weights.get(term, unseen_weight) ** 2 for term in query_features)
    rankings = {key: [] for key in MODES[:3]}
    candidates = {}
    for doc in index["documents"]:
        if doc["evidence_kind"] == "rejected_proof" and not include_rejected:
            continue
        if evidence_kind and doc["evidence_kind"] != evidence_kind:
            continue
        doc_id = doc["document_id"]
        matched = sorted(query_terms.intersection(doc["terms"]))
        word_score = SCALE * len(matched) // len(query_terms)
        vector_score = 0
        if use_vectors:
            shared = query_features.intersection(doc["features"])
            dot = sum(weights[term] ** 2 for term in shared)
            dnorm = sum(weights[term] ** 2 for term in doc["features"])
            vector_score = SCALE * dot * dot // (qnorm * dnorm) if qnorm and dnorm else 0
        parents = sorted(edge["source"] for edge in index["catalog"]["dependencies"]
                         if edge["source"] in seeds and edge["target"] == doc_id)
        graph_score = SCALE if parents else 0
        for key, score in [("keyword", word_score), ("vector", vector_score), ("graph", graph_score)]:
            if score:
                rankings[key].append((doc_id, score))
        candidates[doc_id] = {
            "document_id": doc_id, "title": doc["title"], "source": doc["source"],
            "start_line": doc["start_line"], "end_line": doc["end_line"], "source_sha256": doc["source_sha256"],
            "evidence_kind": doc["evidence_kind"], "assumptions": list(doc["assumptions"]),
            "limitations": list(doc["limitations"]), "matched_terms": matched,
            "vector_similarity_squared_ppm": vector_score if use_vectors else None, "declared_dependency_of": parents,
        }
    for ranking in rankings.values():
        ranking.sort(key=lambda item: (-item[1], item[0]))
    # Equal reciprocal-rank fusion; one-hop explicit graph seeds, no inferred proof edges.
    if mode == "hybrid":
        fused = Counter()
        for ranking in rankings.values():
            for rank, (doc_id, _) in enumerate(ranking, 1):
                fused[doc_id] += SCALE // (60 + rank)
        selected = sorted(fused.items(), key=lambda item: (-item[1], item[0]))[:limit]
    else:
        selected = rankings[mode][:limit]
    return {"index_sha256": index["index_sha256"], "algorithm": VERSION, "mode": mode,
            "candidate_only": True, "query_sha256": digest(query), "seeds": list(seeds),
            "results": [{**candidates[doc_id], "retrieval_score": score} for doc_id, score in selected]}


def evaluate(index, cases):
    exact_keys(cases, ["version", "independent_holdout", "cases"])
    require(cases["version"] == "xlemma-retrieval-evaluation-v1" and cases["independent_holdout"] is False,
            "this runner supports only exposed development cases")
    require(isinstance(cases["cases"], list) and 1 <= len(cases["cases"]) <= 128, "invalid evaluation size")
    ids = {doc["document_id"] for doc in index["documents"]}
    seen, rows = set(), []
    for case in cases["cases"]:
        exact_keys(case, ["case_id", "query", "relevant", "forbidden", "seeds", "evidence_kind", "include_rejected"])
        require(bounded_text(case["case_id"], 64) and case["case_id"] not in seen, "duplicate evaluation case")
        seen.add(case["case_id"])
        require(type(case["include_rejected"]) is bool, "invalid rejected-proof option")
        for key in ["relevant", "forbidden"]:
            require(isinstance(case[key], list) and len(case[key]) <= 64
                    and all(doc_id in ids for doc_id in case[key]) and len(set(case[key])) == len(case[key]), "invalid relevance label")
        require(not set(case["relevant"]).intersection(case["forbidden"]), "conflicting relevance labels")
        for mode in MODES:
            results = search(index, case["query"], mode, 3, case["seeds"], case["evidence_kind"], case["include_rejected"])["results"]
            found = [item["document_id"] for item in results]
            ranks = [i for i, doc_id in enumerate(found, 1) if doc_id in case["relevant"]]
            rows.append({"case_id": case["case_id"], "mode": mode, "returned": found,
                         "relevant_count": len(case["relevant"]), "relevant_retrieved": len(ranks),
                         "first_relevant_rank": min(ranks) if ranks else None,
                         "forbidden_retrieved": sorted(set(found).intersection(case["forbidden"]))})
    summary = {}
    for mode in MODES:
        subset = [row for row in rows if row["mode"] == mode]
        positive = [row for row in subset if row["relevant_count"]]
        summary[mode] = {"cases_with_relevant_result_at_3": sum(bool(row["relevant_retrieved"]) for row in positive),
                         "positive_cases": len(positive),
                         "recall_at_3_ppm": sum(SCALE * row["relevant_retrieved"] // row["relevant_count"] for row in positive) // max(len(positive), 1),
                         "mrr_at_3_ppm": sum(SCALE // row["first_relevant_rank"] for row in positive if row["first_relevant_rank"]) // max(len(positive), 1),
                         "forbidden_suggestions": sum(len(row["forbidden_retrieved"]) for row in subset)}
    return {"version": "xlemma-retrieval-report-v1", "index_sha256": index["index_sha256"],
            "evaluation_sha256": digest(cases), "author_operated": True, "independent_holdout": False,
            "default_workflow_enabled": False, "research_acceleration_established": False,
            "summary": summary, "cases": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    build = sub.add_parser("build")
    build.add_argument("catalog", type=Path)
    build.add_argument("--output", type=Path, required=True)
    lookup = sub.add_parser("search")
    lookup.add_argument("index", type=Path)
    lookup.add_argument("query")
    lookup.add_argument("--mode", choices=MODES, default="hybrid")
    lookup.add_argument("--limit", type=int, default=5)
    lookup.add_argument("--from-document", action="append", default=[])
    lookup.add_argument("--evidence-kind", choices=sorted(KINDS))
    lookup.add_argument("--include-rejected", action="store_true")
    evaluation = sub.add_parser("evaluate")
    evaluation.add_argument("catalog", type=Path)
    evaluation.add_argument("cases", type=Path)
    evaluation.add_argument("--check", type=Path)
    evaluation.add_argument("--measure", action="store_true", help="Include local lookup timing; incompatible with --check")
    args = parser.parse_args()
    if args.command == "build":
        result = build_index(read_json(args.catalog))
        # Exclusive create protects older snapshots and rejects existing symlinks.
        with args.output.open("x", encoding="utf-8") as stream:
            json.dump(result, stream, indent=2, ensure_ascii=False)
            stream.write("\n")
        print(json.dumps({"index_sha256": result["index_sha256"], "documents": len(result["documents"])}))
        return
    if args.command == "search":
        result = search(load_index(args.index), args.query, args.mode, args.limit,
                        args.from_document, args.evidence_kind, args.include_rejected)
    else:
        require(not (args.check and args.measure), "timing measurements cannot be compared to a deterministic report")
        started = time.perf_counter()
        index = build_index(read_json(args.catalog))
        build_ms = (time.perf_counter() - started) * 1000
        cases = read_json(args.cases)
        result = evaluate(index, cases)
        if args.check:
            require(canonical(result) == canonical(read_json(args.check)), "evaluation differs from recorded results")
            print(json.dumps({"status": "matched", "summary": result["summary"]}, indent=2))
            return
        if args.measure:
            measurements = {mode: [] for mode in MODES}
            for trial in range(5):
                for mode in MODES[trial % 4:] + MODES[:trial % 4]:
                    for case in cases["cases"]:
                        started = time.perf_counter()
                        search(index, case["query"], mode, 3, case["seeds"], case["evidence_kind"], case["include_rejected"])
                        measurements[mode].append((time.perf_counter() - started) * 1000)
            result["local_measurement"] = {
                "scope": "Warm in-memory ranking; excludes source/index replay, process startup, proof checking and researcher work",
                "index_build_ms": round(build_ms, 3), "lookups_per_mode": 5 * len(cases["cases"]),
                "external_model_calls": 0, "compute_cost_usdc": None, "energy_measurement": None,
                "modes": {mode: {"median_ms": round(statistics.median(values), 3),
                                 "p95_ms": round(sorted(values)[(95 * len(values) + 99) // 100 - 1], 3)}
                          for mode, values in measurements.items()},
            }
    print(json.dumps(result, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, TypeError, KeyError, OSError, UnicodeError, RecursionError) as error:
        # Avoid dumping source contents, query text or arbitrary transport diagnostics.
        message = str(error) if type(error) is ValueError else "research search failed; check inputs and source paths"
        raise SystemExit(message) from None
