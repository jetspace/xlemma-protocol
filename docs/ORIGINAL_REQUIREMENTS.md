# Original requirements captured in this repository

This file preserves the full functional intent of the design conversation while
incorporating the conservation-law refinements in XLIP-021. The launch market
is intentionally narrow; the provider-neutral protocol remains extensible.

## User and purpose

- The target user is a decentralized researcher.
- A researcher should be able to design a new formula, theorem, lemma, proof, method, or reusable research artifact.
- The creator should be supported economically and retain durable priority, authorship, contribution, rights, and revenue records.
- The system should interface easily with Lean and LaTeX.
- ASTRA should help formalize, discover, repair, and explain proofs.
- Independent Lean verification must remain separate from ASTRA generation.
- A researcher should have their own token/credit, use it to pay for verification, and have realized research profits replenish future research capacity.
- The system should price concrete research services per job and may use conservatively measured compute savings as one signal for a bounded, separately authorized impact pool.
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
- A formal dependency must never create a payment obligation by itself; evidence and economic-policy graphs remain separate.
- Formal validity and human/domain statement alignment must have separate receipts and badges.
- Commons is the default capsule mode and imposes no mandatory per-use protocol fee; Reciprocal is explicit, bounded, nonrecursive, and nonblocking.
- Provider-advertised success rates must not control quality-adjusted compute routing.
- Per-lemma speculative tokens, universal research-value units, and tradeable compute futures are outside the core launch.
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
- Compute spot/reserved-service offers, Quality-Adjusted Certification Cost, model migration spread, and Research Lead Signal.
- Conservative compute-impact allocation gated by settled revenue and explicit impact-pool authorization.
- Proof of Independent Reproduction, generalized quorum, commit-reveal, role-specific committee selection, novelty aggregation, challenges, and operator clustering.
- Optional ERC-1155 proof capsules and license editions.
- Neutral node bonds, proof registry, certificate finality, bounties, vaults, and revenue routing contracts.
- Content-addressed bundles, encrypted-delivery design, storage attestations, and availability thresholds.
- OpenAPI, JSON schemas, deployment configuration, examples, tests, threat model, source register, and traceability matrix.

## Researcher-sovereignty and anti-capture expansion

- Protect identity, custody, economic participation, portability, and credible exit without enclosing mathematical truth.
- Represent origin, attribution, artifact control, economic participation, license control, governance consent, and portability as a Researcher Sovereignty Bundle.
- Use nonexclusive, no-veto, bounded, nonrecursive Researcher Residual Rights assignable only through bilateral signed agreements.
- Maintain descriptive evidence edges separately from prescriptive economic obligations.
- Require Commons, Reciprocal, Commercial Artifact, or Sponsored Challenge economic constitutions before monetization.
- Reward the full human and machine-assisted research production function, including questions, data, methods, formalization, verification, application, and maintenance.
- Measure decentralization across identity, compute, models, verification, storage, settlement, discovery, and governance by the weakest critical layer.
- Support plural privacy-preserving credential issuers, verified participant/operator/node delegation, user-owned compute cooperatives, and beneficial-control clustering.
- Support scarce, abundant, and agent-economy compute regimes using provider-neutral routing and staged nontransferable procurement.
- Treat knowledge productivity as a revisable impact signal, not a debt or universal scientific-value unit.
- Fund work through parallel market, commons, and assurance rails; pay nodes for evidenced settled work rather than inflation or agreement.
- Use multi-chamber constitutional governance with capped influence, public simulation, timelocks, and fork/exit plans; never govern truth or immutable origin.
- Keep only commitments, ordering, bonds, challenges, and settled economics on-chain while retaining full content-addressed evidence off-chain.
- Generalize independent reproduction to Formal, Computational, Statistical, Simulation, Empirical, and Hybrid verification profiles.
