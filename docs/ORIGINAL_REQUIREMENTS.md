# Original requirements captured in this repository

This file preserves the full functional intent of the design conversation so later refactors do not narrow the protocol accidentally.

## User and purpose

- The target user is a decentralized researcher.
- A researcher should be able to design a new formula, theorem, lemma, proof, method, or reusable research artifact.
- The creator should be supported economically and retain durable priority, authorship, contribution, rights, and revenue records.
- The system should interface easily with Lean and LaTeX.
- ASTRA should help formalize, discover, repair, and explain proofs.
- Independent Lean verification must remain separate from ASTRA generation.
- A researcher should have their own token/credit, use it to pay for verification, and have realized research profits replenish future research capacity.
- The system should tie research economics to a compute curve and reward reusable lemmas for conservatively measured compute savings.
- Nodes should be used as sources of decentralized consensus in a way superior to simple majority voting.

## Required corrections and safeguards

- Mathematical truth cannot be owned merely by minting an NFT.
- Token ownership, legal rights, attribution, formal validity, novelty, significance, payment, and LaTeX interpretation must remain distinct.
- Researcher tokens must not be unbacked self-minted value used to bribe or control verifiers.
- Researcher tokens must not weight formal consensus.
- Node consensus should certify independently reproduced evidence, not vote on truth.
- Checker disagreement must produce divergence and quarantine, not majority acceptance.
- Operator diversity must be measured conservatively; public-key count is insufficient.
- Verifiers must be paid for execution and evidence, not for returning `PASS`.
- Honest dissent must not be slashed.
- Profit means settled external revenue after service, compute, refunds, and reserves—not token appreciation.
- Upstream rewards must be capped to avoid recursive royalty explosions and dependency stuffing.
- Rights manifests must disclose employment, university, sponsor, grant, collaborator, and AI-assistance concerns.
- Formal claim IDs must be derived from elaborated Lean expressions under a pinned theory, not source strings.
- Equivalence between claims must itself be formally proved.
- Human-readable LaTeX cannot override the formal Lean statement.
- All corrections, disputes, revocations, and supersessions must remain append-only.

## Required modules and outputs

- Rust-first workspace and command line interface.
- Protocol IDs, manifests, proof rights capsules, receipts, and state machines.
- ASTRA Responses API adapter.
- Lean build/checker/sandbox boundary and independent-checker policy.
- LaTeX package and Lean annotation package.
- x402 exact, upto, and batch-settlement transport types and extension payload.
- Research-credit and stable-backing ledger.
- Revenue waterfall and auto-compounding.
- Compute spot/forward offers, Verified Proof Cost, model migration spread, and Research Lead Signal.
- Compute-savings dividend.
- Proof of Independent Reproduction, generalized quorum, commit-reveal, role-specific committee selection, novelty aggregation, challenges, and operator clustering.
- Optional ERC-1155 proof capsules and license editions.
- Neutral node bonds, proof registry, certificate finality, bounties, vaults, and revenue routing contracts.
- Content-addressed bundles, encrypted-delivery design, storage attestations, and availability thresholds.
- OpenAPI, JSON schemas, deployment configuration, examples, tests, threat model, source register, and traceability matrix.
