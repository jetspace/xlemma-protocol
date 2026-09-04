# Changelog

## Unreleased — XLMP/1 protocol independence

- Added `xlemma-xlmp` as the canonical wire, message, lifecycle, and adapter-contract layer.
- Added typed, content-derived XLMP MessageIDs and twenty-five XLMP/1 message discriminators, including native user, operator, node credential, and revocation messages.
- Added the constitutional `VerifiedUserID → OperatorID → NodeID(s)` identity chain, pseudonymous credentials, tiered role qualifications, delegation, fresh non-revocation status, and append-only revocation records.
- Added an append-only credential registry with a production signature-verifier adapter boundary and exact canonical credential vectors.
- Added a first-class node-network protocol plane with signed service advertisements, constrained discovery, append-only orders and matches, evidence-backed bonds, and exact advertisement identity vectors.
- Added eight-dimensional operator-primary/node-subrecord reputation without a composite authority score: formal accuracy, availability, latency, novelty calibration, challenge quality, independence, storage quality, and integrity.
- Added deterministic committee sortition over a credential-bound committed eligible set and future randomness, with unique VerifiedUserIDs, OperatorIDs, operator clusters, provider/region diversity, and reproducible member-rank and selection roots.
- Elevated verification, challenge, quarantine, compute, credit, vault, revenue, dividend, license, and publication records into provider-neutral core types and schemas.
- Added canonical HTTP XLMP ingress while retaining REST endpoints as convenience adapters.
- Recast ASTRA, Lean, x402, chains, transports, and storage providers as replaceable adapters that cannot redefine research consensus.
- Bound x402 offers to an XLMP MessageID and renamed the extension namespace to `xlmp`.
- Added XLMP lifecycle, integrity, node-market, sortition, economic backing, and dependency-cap tests.
- Restricted ClaimID and ProofID construction to theory-bound elaborated types
  and claim-bound canonical proof objects, excluding presentation metadata.
- Added and pinned `Cargo.lock` for reproducible Rust 1.82 reference builds.

## 0.2.0 — Researcher-first consensus architecture

- Added sovereign decentralized researcher nodes and one backed research-credit token per researcher.
- Added immutable lemma capsules, rights manifests, contribution graphs, and revenue routes.
- Added Proof of Independent Reproduction (PoIR), generalized role quorums, checker diversity, commit-reveal, divergence quarantine, and challenge logic.
- Added ASTRA proof-production adapter and Lean/comparator verification adapter boundaries.
- Added x402 exact, upto, and batch-settlement service mappings.
- Added compute forward curves, verified-proof-cost curves, and conservative compute-savings dividends.
- Added Solidity reference contracts, JSON schemas, OpenAPI description, Lean and LaTeX packages, deployment templates, threat model, legal boundary notes, and traceability matrix.
- Added domain-separated signing envelopes, nonce replay protection, signed node assignments, role-conflict enforcement, and observation-receipt construction.
- Added executable Solidity unit, fuzz, and stateful solvency-invariant tests plus operational diagrams, governance, telemetry, runbooks, deployment, and production-readiness documents.
- Hardened provider/region quorum diversity, content identities, atomic credit/revenue accounting, payee binding, bounty finality, and capsule interface support.
- Strengthened CI with Lean kernel rechecking, nanoda, axiom auditing, Foundry build/tests, and explicit dependency-lock release gates.
- Added a reference API route that constructs x402 V2 `402 Payment Required` offers carrying the xLemma job extension without conflating payment with proof validity.
- Added a deterministic SHA-256 source manifest, manifest generator, and validator coverage for file-set and digest integrity.
- Hardened OpenAPI validation for operation IDs, response maps, repository-contained external schemas, fragments, and unresolved references.

## 0.1.0 — Proof-carrying knowledge baseline

- Introduced content-addressed theories, claims, proofs, artifacts, and verification receipts.
- Separated mathematical validity, authorship, ownership, licensing, presentation, payment, and economic rights.
