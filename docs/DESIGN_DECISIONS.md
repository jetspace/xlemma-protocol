# Architecture decision record

## ADR-001 — Existing chain first

Use an existing chain for ordering and economic finality. Research nodes create evidence certificates off-chain. A bespoke chain is unnecessary until throughput or governance requirements justify it.

## ADR-002 — Evidence consensus, not truth voting

Formal validity uses deterministic reproduction and generalized checker-family quorums. Token voting is prohibited.

## ADR-003 — One researcher credit, many capsules

Each researcher has one primary backed service credit. Each result receives an immutable capsule. This avoids a separate speculative market and liquidity pool for every lemma.

## ADR-004 — Full backing in V1

Research credits are issued only against independently valued assets or settled revenue. This breaks circular self-payment and makes verifier compensation credible.

## ADR-005 — ASTRA is a producer, not verifier

ASTRA can formalize and search aggressively, but independent Lean/checker nodes certify exact artifacts.

## ADR-006 — Exact formal IDs

Claim identity uses canonical elaborated Lean expressions under a theory ID. Human text and source formatting are not authoritative identifiers.

## ADR-007 — Append-only history

Corrections create new nodes and explicit lineage. No administrator silently mutates past claims, contributions, rights, or verification records.

## ADR-008 — Optional tokenization

Token handles are projections over immutable records. Proof capsules and origin certificates are non-transferable by default; license editions may be transferable if backed by real rights.

## ADR-009 — Compute savings use conservative counterfactuals

Rewards use a lower confidence bound and a cap on realized downstream net revenue. The protocol does not create uncapped recursive royalties.

## ADR-010 — Reviewer calibration over conformity

Novelty reviewers are rewarded for evidence and delayed calibration rather than agreement with a present majority.
