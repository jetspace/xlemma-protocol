# Research retrieval with vectors and declared dependencies

Decision and prototype: 2026-09-11. xLemma adds an opt-in local search layer over
public research artifacts. Exact verification and published USDC settlement
rules retain their authority. Search returns candidates to inspect and reproduce;
it cannot certify a result, establish novelty, assign reward weight or move money.

The first implementation uses sparse lexical vectors, exact tokens and declared
dependency navigation. It makes no external model calls and requires only Python's
standard library. Learned semantic embeddings remain a possible later adapter,
subject to a meaningful evaluation against these baselines.

## Reproduce the prototype

```sh
python3 scripts/research_search.py build examples/research-search/catalog.json \
  --output /tmp/xlemma-research-index.json
python3 scripts/research_search.py search /tmp/xlemma-research-index.json \
  'midpoint dampng' --limit 3
python3 scripts/research_search.py search /tmp/xlemma-research-index.json \
  'supporting result' --mode graph --from-document midpoint-conservation
make research-search-check
```

Build requires a new output filename and refuses to overwrite a previous snapshot.
To rebuild after a reviewed change, choose another filename. Search revalidates
the snapshot against the current sources; a stale snapshot fails instead of
silently following changed artifacts. JSON results show source locations,
assumptions, limitations, catalog evidence kind and the reason for a graph match.
The raw query is not echoed in the report; a query digest is included instead.
Digests are not anonymization, and caller shell history remains outside this tool.

Use `--mode keyword`, `vector`, `graph` or `hybrid` to compare methods. `hybrid`
is the search command's default mode, but no research, verification or funding
workflow automatically calls this prototype. `--evidence-kind` filters catalog
labels; it does not authenticate evidence. Rejected-proof fixtures are hidden
unless `--include-rejected` is supplied, and retain their rejected label.

## Data and identity boundaries

The [catalog](../examples/research-search/catalog.json) selects nine existing
public source excerpts: oscillator proofs, an arithmetic example, a rejected
proof fixture, numerical tests and research/economic design notes. Every entry
pins the full source SHA-256, line range, human description, assumptions and
limitations. It contains no independently qualified empirical artifact.

Document IDs are local retrieval labels. Source hashes and index hashes are
cache/provenance commitments, not formal ClaimIDs. Changing an index or ranking
does not change any research object, certificate, historical receipt or payout.
The catalog is manually curated and is not an authenticated scientific registry.

The conservation example's dependency on the general balance is represented as
a `declared_dependency` navigation link, with both underlying sources available
for inspection. These document-level links do not introduce a new XLMP edge kind
or replace the evidence requirements for a formal dependency. Graph navigation
uses only explicit seed documents and one outgoing hop. It never invents an
equivalence or infers unification from similarity.

Only explicitly listed source files are opened. Hidden paths, release-excluded
directories, unsupported file types, symlinks, duplicate document IDs, duplicate
source selections, missing targets and changed source hashes fail validation.
Curators must include only public material; a path or hash cannot determine
whether a document's contents are confidential. This prototype performs no
automatic repository crawl, provider upload or credentialed search.

## Algorithms and reproducibility

The versioned index binds its catalog, source excerpts, feature weights, tokenizer
and implementation-source digest. Loading reconstructs that index and rejects
altered vectors or metadata. Deterministic integer arithmetic and document-ID
tie breaking make results stable across source ordering and supported Python
versions. Corpus size, excerpt length, query length and graph expansion are bounded.

| Method | Ranking rule |
|---|---|
| Keyword | Fraction of distinct query tokens occurring in the document's searchable text |
| Vector | Binary within-word character trigrams, integer inverse-document-frequency weights, squared cosine similarity |
| Graph | Declared dependencies of explicitly supplied seed documents |
| Hybrid | Equal reciprocal-rank fusion of the three lists, with constant 60 |

Titles, summaries, source excerpts, assumptions and limitations are searchable.
Repeated occurrences within a document do not increase its feature weight.
Tokenization handles ASCII words and identifier segments. Mathematical symbols
stay visible in the source; this tokenizer does not interpret formulas, negation,
units, physical regimes or semantic equivalence. Similarity scores and ranking
points are not probabilities of correctness or usefulness.

Sparse feature extraction is a conventional numerical representation of text;
it does not require a learned model. See [feature extraction documentation](https://scikit-learn.org/stable/modules/feature_extraction.html#text-feature-extraction)
and [vector similarity search](https://qdrant.tech/documentation/overview/vector-search/)
for background. This implementation installs neither library.

## What the current comparison establishes

The [recorded evaluation](../reports/research-search-evaluation.json) contains
14 author-authored development queries: 13 with relevant documents and one
empirical-evidence filter case whose correct result is empty. The queries and
labels are exposed to the implementation author. They are not an independent
holdout, and their relevance judgments have not received independent review.

| Method | Positive queries with a relevant top-three result | Mean recall at three | Mean reciprocal rank at three | Explicitly misleading suggestions |
|---|---:|---:|---:|---:|
| Keyword | 10/13 | 0.731 | 0.718 | 1 |
| Vector | 11/13 | 0.846 | 0.641 | 1 |
| Graph | 1/13 | 0.077 | 0.077 | 0 |
| Hybrid | 11/13 | 0.808 | 0.769 | 1 |

Only one query supplies a graph seed; graph-only scores show its limited scope,
not the quality of all dependency navigation. The keyword baseline is simple
token coverage, not a tuned BM25 system. Unjudged results are not automatically
counted as harmful; the last column counts only explicit forbidden labels.

The known assumption trap remains visible: a query about energy conservation
with positive damping retrieves the zero-damping conservation theorem. Its
assumptions disallow that use. The lexical-gap query also exposes the limits of
word-form similarity. These failures are retained in the report and regression
checks. No ranking threshold can turn these candidates into accepted proofs or
reward entitlements.

Local timing can be reproduced separately:

```sh
python3 scripts/research_search.py evaluate examples/research-search/catalog.json \
  examples/research-search/evaluation.json --measure > /tmp/xlemma-search-timing.json
```

Timing runs five rounds in rotating mode order and reports median/p95 warm
in-memory ranking latency, plus index construction time. Ranking timings exclude
source/index integrity replay, process startup, proof reproduction and researcher
work. Energy and monetary compute costs remain
unmeasured, not zero. The [local validation record](../reports/research-search-validation.json)
records one run and its scope; timing is not a deterministic CI acceptance check.

## Conditions for wider integration

Keep the prototype opt-in. Next, expand licensed upstream coverage, qualify
scientific metadata and compare against exact-symbol and stronger lexical
retrieval. Independent researchers should preregister unseen tasks, relevance
judgments, assurance requirements and acceptable misleading-suggestion rates.
Measure successful verified reuse, full human/compute/maintenance costs and
access failures separately from retrieval accuracy.

A learned-vector adapter would need explicit public/private data controls,
pinned model/preprocessing versions, reproducible document bindings and a way to
rebuild or export the index. It must leave exact proof identities and economic
eligibility untouched. Promote a search mode into the normal research workflow
only when independent evidence supports that choice. The USDC wallet and
independent physics milestones continue alongside this experiment.
