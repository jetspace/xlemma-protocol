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

Compute savings are an uncertain impact signal. Allocation requires a lower
confidence bound, a separately authorized impact-pool budget, a settled
revenue event, and non-recursive treatment. The protocol does not create an
invoice from a dependency or permit uncapped recursive royalties.
Evidence is content-addressed and asset-bound; checked fixed-point integers,
not floating-point arithmetic, determine any proposed allocation.

## ADR-010 — Reviewer calibration over conformity

Novelty reviewers are rewarded for evidence and delayed calibration rather than agreement with a present majority.

## ADR-011 — Formal validity is not statement alignment

Lean checks the exact formal statement under declared definitions and axioms.
Domain reviewers issue a separate content-addressed
`StatementAlignmentReceipt` for the informal claim and presentation. User
interfaces display both statuses.

## ADR-012 — Evidence and economic graphs are separate

Formal dependency edges describe use. Only explicit economic-policy edges can
authorize a bounded payment from settled external revenue. Open Commons uses
no mandatory per-use fee.

## ADR-013 — Protocol-calibrated service pricing

Compute offers contain concrete price and capacity. Quality-adjusted routing
uses signed, time-bounded protocol outcome estimates, never a provider's own
success claim. Quotes use fixed-point probabilities, checked integer money
math, conservative rounding, and deterministic tie-breaking. Service
reservations precede any financial derivative layer.

## ADR-014 — Narrow external-buyer launch

Launch with one sponsor-backed ASTRA/Lean formalization and certification
vertical. Expand only after repeat external demand and calibrated completion
history exist.
