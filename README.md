# xLemma Protocol

**Proof-carrying decentralized research for sovereign researchers, ASTRA-assisted proof production, Lean verification, x402 payments, node-based independent reproduction, and compute-linked research economics.**

> Status: architectural reference implementation and prototype. The Rust, Lean, Solidity, payment, and cryptographic components have not been independently audited. Do not deploy with real funds until the missing production work in `ROADMAP.md` is complete.

## What xLemma is

xLemma lets an individual or collective operate as a **Decentralized Researcher Node** with:

- a sovereign researcher identity;
- a fully backed, closed-loop Research Credit token, `Rᵢ`;
- a Research Vault, `Vᵢ`;
- immutable Lemma Capsules for claims, proofs, papers, code, data, and rights;
- ASTRA-assisted formalization, proof search, repair, and exposition;
- Lean plus independently implemented checker verification;
- Proof of Independent Reproduction (PoIR) node certificates;
- x402 payment for proof search, verification, review, storage, and access;
- revenue routing to contributors, dependencies, public goods, and future compute;
- compute-forward pricing and conservative compute-savings dividends.

The flywheel is:

```text
research funding
  → ASTRA compute
  → Lean proof candidate
  → independent reproduction
  → certified lemma capsule
  → paid use / bounty / license / service
  → settled external revenue
  → researcher income + newly backed research credits
  → more research
```

## The central separation

xLemma never treats token ownership or node popularity as mathematical truth.

```text
Lean validity
  ≠ authorship
  ≠ legal ownership
  ≠ token ownership
  ≠ license rights
  ≠ novelty
  ≠ significance
  ≠ LaTeX interpretation
  ≠ payment settlement
```

Under a pinned environment, formal validity is deterministic. Nodes independently reproduce and sign observations. The network reaches consensus over **evidence sufficiency and state transitions**, not over truth by majority vote.

## Researcher-first object model

```text
DecentralizedResearcherNode
├── ResearcherID
├── ResearchCredit Rᵢ
├── ResearchVault Vᵢ
├── ContributionIdentity
├── ReputationRecord
├── LemmaCapsules[]
├── ComputePositions[]
├── RevenueAccounts[]
├── Licenses[]
└── GovernancePolicy

LemmaCapsule
├── LemmaID
├── TheoryID
├── ClaimID
├── ProofID
├── ArtifactRoot
├── LeanArtifact
├── LaTeXPresentation
├── OriginCertificate
├── ContributionManifest
├── MachineContributionRecord
├── DependencyRoot
├── VerificationReceipts
├── NoveltyReceipts
├── RightsManifest
├── RevenueRoute
├── ComputeHistory
└── VersionLineage
```

One researcher has one primary research-credit economy and many immutable lemma capsules. xLemma deliberately avoids launching one freely tradable token for every lemma.

## Proof of Independent Reproduction

A node does not merely vote `PASS`. It signs:

```text
I independently executed checker implementation X,
against artifact Y,
inside environment Z,
under policy P,
and observed result R.
```

A Gold formal certificate requires a generalized role quorum such as:

```text
2 independent official Lean-kernel executions
AND 1 independently implemented checker execution
AND identical ClaimID, ProofID, ArtifactRoot and DependencyRoot
AND identical permitted axiom inventory
AND distinct operator clusters
AND no unresolved challenge
```

If one required checker family disagrees, the result is `DIVERGENT` and moves to `QUARANTINED`. It is never accepted by a 2-to-1 vote.

## Research-credit economics

`Rᵢ` is initially a **fully backed research service credit**, not a speculative profit token.

```text
1,000 USDC deposited into Vᵢ
  → at most 1,000 Rᵢ minted

40 Rᵢ spent on verification
  → 40 Rᵢ burned
  → up to 40 USDC released to independent service nodes
```

Credits can be minted only from independently valued backing, settled external revenue, grants, bounties, or conservatively valued prepaid compute. They cannot be minted against an unverified lemma, expected profits, or their own secondary-market price.

A researcher may auto-compound a chosen fraction of settled contributor revenue into new backed credits:

```text
creator revenue = 650 USDC
auto-compound rate = 60%

390 USDC retained in vault + 390 Rᵢ minted
260 USDC paid to researcher
```

## Compute curve

xLemma tracks service-specific forward curves:

- `Fᴬˢᵀᴿᴬ(d,T)` — model-assisted proof production;
- `Fᴸᵉᵃⁿ(T)` — build and deterministic verification;
- `Fʳᵉᵛⁱᵉʷ(d,T)` — novelty and expert review;
- `Fˢᵗᵒʳᵃᵍᵉ(T)` — replicated proof availability.

The expected cost of a Gold-verified, novelty-cleared result is represented by a Verified Proof Cost curve:

```text
VPC(d,T) = min over models and compute classes of
           (generation + verification + expert review cost)
           / (probability of Gold verification × probability of novelty clearance)
```

Reusable upstream lemmas may receive a capped share of **conservatively measured downstream compute savings**, but only from realized protocol revenue and only when the lemma is present in the final dependency graph.

## ASTRA and Lean

ASTRA is a pluggable proof-production agent. It may:

- formalize prose and LaTeX into candidate Lean statements;
- decompose goals;
- search for proof strategies;
- generate and repair Lean code;
- select existing lemmas;
- explain verified results in LaTeX;
- produce signed compute and generation receipts.

ASTRA does not certify itself. Final formal assurance comes from pinned Lean builds, trusted challenge matching, axiom inspection, the official kernel, independently implemented checkers, sandboxing, and open challenge periods.

The model name is configuration-driven through `OPENAI_MODEL`; the default in this snapshot is `gpt-6-astra`.

## x402 transport

xLemma maps research services onto x402 payment schemes:

| Service | Scheme |
|---|---|
| Fixed basic Lean check | `exact` |
| Variable ASTRA proof search | `upto` |
| Metered repair session | `upto` |
| Repeated proof-state calls | `batch-settlement` |
| Certified artifact download | `exact` |
| Continuous research-agent session | `batch-settlement` |

The payment facilitator validates and settles payment payloads. It is intentionally outside research consensus. `PaymentReceipt` and `VerificationReceipt` remain separate, though both bind the same immutable job and artifact identifiers.

## Repository map

```text
crates/
  xlemma-core/           IDs, manifests, capsules, receipts and state types
  xlemma-crypto/         domain-separated envelopes, signatures and replay protection
  xlemma-consensus/      PoIR, generalized quorums, commit-reveal and novelty aggregation
  xlemma-node/           node capabilities, assignments, role separation and receipt workflow
  xlemma-economics/      backed credits, vault conservation, revenue and dividends
  xlemma-compute-curve/  service offers, forward curves and proof-cost quotes
  xlemma-astra/          configurable OpenAI Responses API adapter and proof prompts
  xlemma-lean/           sandbox/checker command boundaries and receipt generation
  xlemma-x402/           HTTP 402 headers, schemes and xLemma extension payloads
  xlemma-storage/        content-addressed bundle manifests and availability receipts
  xlemma-api/            HTTP reference server
  xlemma-cli/            command-line reference client

contracts/               unaudited Solidity reference contracts
lean/                    Lean tag attribute, export marker and example
latex/                   xlemma.sty and example document
schemas/                 JSON Schema definitions for protocol objects
spec/                    normative protocol specifications
docs/                    integrated design, diagrams, economics, governance, operations, testing and threats
openapi/                  REST API contract
config/                   default policy configuration
examples/no-arbitrage/    end-to-end illustrative lemma package
deploy/                   local container deployment templates
scripts/                  validation, simulation and archiving tools
```

## Start here

1. Read `docs/FULL_DESIGN.md` for the complete integrated architecture.
2. Read `docs/ARCHITECTURE_DIAGRAMS.md` for system, trust-plane, lifecycle, economic, and deployment diagrams.
3. Read `docs/TRACEABILITY_MATRIX.md` to locate every requested concept.
4. Read `docs/RESEARCHER_USER_JOURNEYS.md` for researcher, supporter, node, bounty, reuse, and correction workflows.
5. Read `spec/000-overview.md` and `spec/003-poir-consensus.md` before modifying consensus.
6. Review `docs/THREAT_MODEL.md`, `docs/GOVERNANCE_CONSTITUTION.md`, and `docs/LEGAL_BOUNDARIES.md` before deployment.
7. Use `docs/OPERATOR_RUNBOOK.md`, `docs/TESTING_STRATEGY.md`, and `docs/PRODUCTION_CHECKLIST.md` as implementation gates.
8. Run `python3 scripts/validate_repo.py` to validate schemas, OpenAPI references, invariants, the source manifest, and repository completeness.
9. Verify every source digest with `sha256sum -c MANIFEST.sha256`.
10. Read `docs/VALIDATION_REPORT.md` for checks executed in this source snapshot and explicit limitations.

> **Source-snapshot note:** this archive intentionally does not fabricate a `Cargo.lock` or contract dependency lock without native dependency resolution. The first dependency-enabled build must generate, review, and commit those locks before a production release.

## Local commands

```bash
cp .env.example .env
python3 scripts/validate_repo.py
python3 scripts/simulate_economics.py
sha256sum -c MANIFEST.sha256

# Requires Rust 1.82+
cargo test --workspace
cargo run -p xlemma-api
cargo run -p xlemma-cli -- --help
```

Lean and Solidity toolchains are optional for the structural validator and required for their respective modules.

## Non-negotiable invariants

1. Researchers cannot pay nodes with unbacked self-minted value.
2. ASTRA may produce proofs but cannot certify its own output.
3. Verifiers are paid for reproducible execution, not agreement or a passing verdict.
4. Required checker disagreement causes divergence and quarantine.
5. Researchers cannot independently finalize their own claims.
6. Realized token appreciation is not research revenue.
7. Profit distributions originate only from settled external revenue after costs and reserves.
8. Only final proof dependencies qualify for dependency rewards.
9. Formal claim changes create new `ClaimID`s.
10. Corrections, revocations, dissent, and supersession remain append-only.
11. Payments, formal validity, novelty, rights, and availability have separate receipts.
12. Token ownership cannot rewrite authorship or Lean validity.
13. A verified Lean theorem may still be trivial, previously known, misleadingly described, or commercially valueless.
14. Rights manifests cannot create intellectual-property rights the contributor never owned.
15. Quorum requirements measure independent operator and implementation diversity, not public-key count.

## Sources and design basis

The dated source register is in `docs/SOURCES.md`; prior-art positioning is in `docs/PRIOR_ART_AND_DIFFERENTIATION.md`. Key foundations include official OpenAI Responses API documentation, official x402 V2 documentation, the Lean proof-validation guide, ERC-1155 and ERC-4626 specifications, RFC 8785, EIP-712, content-addressed storage literature, generalized Byzantine quorum research, and cryptographic sortition research.
