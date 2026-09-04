# Roadmap

This roadmap distinguishes a checked-in reference implementation from a
production deployment. A checked item is reproducible in this repository; it
does not imply an independent audit, live-network deployment, or legal review.

## Current execution focus — Phase 0 closure

- [x] Publish versioned, domain-separated ID derivation and RFC 8785 canonicalization code.
- [x] Provide a safe content-addressed bundle builder with an explicit timestamp and a byte-for-byte deterministic vector.
- [x] Add content-derived axiom profiles, fail-closed trust policies, a canonical registry snapshot, schemas, tests, and a CLI evaluator.
- [x] Publish reference vectors for statement alignment, capsule economic modes, and separate evidence/economic graph edges.
- [x] Run all eleven documented participant journeys through a repeatable conformance harness with machine-readable and human-readable reports.
- [x] Replace volatile API history with a single-writer durable, hash-chained event journal that fsyncs before acknowledgement and fails closed during restart recovery.
- [x] Add canonical length-delimited XLMP framing for non-HTTP binary transports with exact MessageID preservation and malformed/non-canonical frame rejection.
- [ ] Obtain clean-room cross-implementation results before declaring the encodings or schemas frozen.
- [ ] Complete and independently test the Lean environment exporter; the current `#xlemma_export` command remains a prototype print boundary.

## Phase 0 — deterministic research objects

- Freeze canonical encodings for TheoryID, ClaimID, ProofID, ArtifactRoot and ReceiptID after the published reference vectors pass clean-room reproduction.
- Complete the Lean exporter for elaborated expression, universe, dependency, proof-term, and trust-evidence serialization.
- Extend deterministic vectors to every XLMP message and receipt type in a second implementation.
- Authenticate trust-registry root publication through production governance and key resolution.
- Freeze sovereignty, residual-right, portability, economic-constitution/compliance, generalized-verification, funding, capture, governance, trust-policy, and axiom-profile schemas after independent implementation review.

## Phase 1 — independent verification network

- Harden the XLMP/1 reference sortition into a production beacon/VRF integration and distributed eligible-set registry; deterministic selection and proof reproduction are implemented.
- Integrate audited credential issuers, issuer/delegation key resolution, privacy-preserving uniqueness proofs, and a distributed revocation accumulator; structural credential records and the verifier adapter boundary are implemented.
- Harden sandbox isolation with no-network execution, seccomp, namespaces, immutable images and resource controls.
- Integrate Lean `comparator`, official kernel replay and independent checkers.
- Deploy PoIR commit-reveal committees with verified-user, OperatorID, and operator-cluster controls.
- Add watcher, dispute, quarantine and revalidation services.
- Scale the implemented durable single-writer API event journal to a replicated transactional log/outbox, preserving hash-chain verification and the authenticated-observation certificate gate.

## Phase 2 — researcher credits and x402

- Deploy audited researcher-vault factory and fully backed restricted credits.
- Integrate production x402 SDKs and a production facilitator or self-facilitation path.
- Add exact, upto, batch-settlement, idempotency and refund reconciliation.
- Add stable-asset accounting, reserves and external attestations.

## Phase 3 — research marketplace

- Launch one sponsor-backed ASTRA/Lean formalization and certification vertical with identifiable external buyers before broadening domains.
- Deploy the implemented XLMP node advertisement, constrained discovery, service-order, immutable match, multidimensional-reputation, and bond records through multiple independent indexers.
- Add public bounties, grants, certified proof services, encrypted delivery, and license manifests.
- Qualify ASTRA and alternative implementations in the provider-neutral research-prover marketplace.
- Add calibrated novelty-review committees and formal equivalence graphing.
- Deploy user-owned compute cooperatives and independently measured eight-layer capture dashboards through multiple frontends.

## Phase 4 — compute economics

- Publish spot and forward compute offers.
- Train independent protocol success estimators on proof-state, model, domain and checker telemetry; provider self-estimates do not control routing.
- Introduce conservative compute-impact signals with randomized holdout re-proving and separately authorized bounded impact pools.
- Add public-goods and negative-result funding.
- Deploy the implemented market, commons, and assurance rails with independently reconciled external settlement.
- Defer tradeable compute futures until service profiles and settlement history are standardized and liquid.

## Phase 5 — optional rights wrappers

- Add audited ERC-1155 capsule and license editions.
- Add jurisdiction-specific contractual rights vaults.
- Audit and deploy the append-only `ResearchCommitmentRegistry` projection of policy, committee, rights, contribution, and supersession roots.
- Keep public profit-sharing or tokenized-vault interests outside the protocol core and subject to separate legal design.
- Keep per-lemma speculative tokens, universal mandatory royalties, and universal research-value units outside the protocol core.

## Production blockers

- Independent cryptographic, smart-contract and sandbox audits.
- Deterministic Lean exporter and cross-checker test corpus.
- Audited participant uniqueness, credential privacy/revocation, and conservative operator-cluster resistance.
- Economic conservation and insolvency proofs.
- Privacy, sanctions, identity, tax and jurisdictional review.
- Incident response, key rotation, disaster recovery and monitoring.
- Independent clean-room reconstruction from portability manifests and an exercised constitutional fork/funds-exit path.
