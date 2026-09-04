# Production-readiness checklist

Every box is a release gate. The reference repository intentionally leaves many production gates unchecked.

## Build and dependency integrity

- [x] Resolve the Rust workspace with the pinned toolchain and commit `Cargo.lock`.
- [ ] Complete independent dependency and license review of the locked Rust graph before a release tag.
- [ ] Pin OpenZeppelin, forge-std, Lean, checker, container, and system dependency revisions by immutable version or digest.
- [ ] Generate and retain software bills of materials and provenance attestations for API, node, checker, and contract builds.
- [ ] Reproduce release artifacts from a clean environment and compare digests.
- [ ] Require protected-branch CI and signed release tags.

## Protocol and identifiers

- [ ] XLMP/1 envelope and all required message vectors independently implemented.
- [ ] Unsupported XLMP versions, unknown required fields, and adapter downgrades fail closed.
- [ ] HTTP and one non-HTTP transport preserve identical MessageIDs.
- [ ] Canonical serialization frozen and independently implemented.
- [ ] RFC 8785-compatible behavior covered by published vectors.
- [ ] ClaimID derived from elaborated Lean expressions, including universes and implicit structure.
- [ ] ProofID derived from a stable exported proof object.
- [ ] Artifact identities exclude mutable provenance fields.
- [ ] Domain separation reviewed cryptographically.
- [ ] Signature domains bind network, contract, nonce, expiry, policy, and artifact.
- [ ] Append-only correction/supersession semantics tested.

## Verifier and checker assurance

- [ ] Exact trusted challenge workflow implemented.
- [ ] `#print axioms` or equivalent inventory enforced.
- [ ] Placeholder/custom-axiom laundering corpus passes.
- [ ] Official Lean kernel replay deployed.
- [ ] Independent checker deployed and qualified.
- [ ] Comparator/build-script threat boundary tested.
- [ ] No-network sandbox hardened and independently audited.
- [ ] Toolchain, dependencies, checker binaries, and images pinned by digest.
- [ ] Required checker divergence always quarantines.
- [ ] Historical revalidation workflow exercised.

## Node network and service marketplace

- [ ] Advertisement signatures and content-derived IDs verified at every ingress.
- [ ] Advertisement supersession and service-order/match history remain append-only.
- [ ] Independent discovery providers reproduce the same constrained ordered set.
- [ ] Price arithmetic is integer, overflow-checked, rounded upward, and asset/unit compatible.
- [ ] Capacity, latency, checker-family, hardware, and terms claims have challengeable evidence.
- [ ] Every reputation dimension has role-specific evidence and minimum sample-size policy.
- [ ] No scalar reputation, bond amount, or token balance becomes committee or formal-vote weight.
- [ ] Eligible-set publication is decentralized and committed before randomness reveal.
- [ ] Production beacon/VRF proofs are authenticated and manipulation-resistant.
- [ ] Committee ranks, unique operator clusters, provider/region diversity, and selection root reproduce independently.

## ASTRA and model layer

- [ ] Provider API behavior and model name are configuration-driven.
- [ ] Researcher approves the exact formal target before economic finalization.
- [ ] Prompts, context, tools, and output roots recorded without leaking secrets.
- [ ] Model usage and settlement reconciled.
- [ ] ASTRA cannot sign final independent verification.
- [ ] Alternative prover adapter tested.
- [ ] Private-data and retention policies exposed before authorization.
- [ ] Success/cost calibration measured point in time.

## PoIR network

- [ ] Committee randomness is manipulation-resistant and publicly verifiable.
- [ ] Operator clusters are conservatively assigned and challengeable.
- [ ] Stake is capped eligibility collateral, not formal vote weight.
- [ ] Provider and region diversity enforced.
- [ ] Commit-reveal resists copying and withholding.
- [ ] Node signatures, membership, timing, and roots are verified.
- [ ] Honest dissent is paid and preserved.
- [ ] Objective slashing evidence and appeal process defined.
- [ ] Watcher network is independent of primary aggregator.
- [ ] Challenge, quarantine, dismissal, and rejection paths tested.

## Research-credit economics

- [ ] Settlement asset behavior is compatible and audited.
- [ ] `backing >= totalSupply` holds under stateful fuzzing.
- [ ] Deposit, authorization, settlement, cancellation, compounding, and redemption tested.
- [ ] Payee binding and exact refund behavior verified.
- [ ] Reentrancy, role, token-callback, fee-on-transfer, and rebase risks handled.
- [ ] Daily independent solvency reconciliation deployed.
- [ ] Auto-compound receives external assets before minting.
- [ ] Unrealized token price is excluded from revenue.
- [ ] Public/profit-linked token layer legally and technically separated.

## Revenue and compute dividends

- [ ] Gross-to-net revenue definition is explicit and auditable.
- [ ] Waterfall shares and contributor shares conserve value.
- [ ] Rounding treatment is explicit.
- [ ] Revenue events are replay-safe.
- [ ] Dependency graph uses only final proof dependencies.
- [ ] Equivalence clustering and cycle/self-citation detection deployed.
- [ ] Compute-savings estimator uses conservative uncertainty bounds.
- [ ] Dividends are capped by realized downstream net revenue.
- [ ] Measurement disputes and corrections are append-only.

## Payment adapters

- [ ] At least one non-x402 payment adapter passes common authorization, settlement, replay, and receipt tests.
- [ ] Current official SDK/protocol version reverified.
- [ ] Exact, upto, and batch flows tested against selected facilitator/network.
- [ ] Idempotency survives retries and lost responses.
- [ ] Quote expiry, maximum authorization, actual settlement, and refund reconcile.
- [ ] Payment and proof receipts are separate.
- [ ] Facilitator has no research-consensus role.
- [ ] Metadata privacy reviewed.
- [ ] Bounties remain a separate reverse-payment escrow.

## Smart contracts

- [ ] Foundry unit, fuzz, and invariant tests pass.
- [ ] Static analysis and symbolic execution reviewed.
- [ ] Two independent audits completed.
- [ ] Public security contest completed for material value.
- [ ] Role graph and admin-key controls reviewed.
- [ ] Threshold governance and emergency paths tested.
- [ ] Chain/network assumptions and reorg behavior tested.
- [ ] Deployment addresses, source verification, and reproducible bytecode published.
- [ ] Value caps and staged rollout configured.
- [ ] User redemption/exit procedure tested.

## Storage and privacy

- [ ] Content-addressed bundle builder has cross-language vectors.
- [ ] Traversal, symlink, decompression, and resource attacks tested.
- [ ] Encryption and key-release path audited.
- [ ] Multiple independent operators/providers/regions satisfy policy.
- [ ] Custody challenges test actual retrievability.
- [ ] Private content absent from ordinary logs and public headers.
- [ ] Retention/deletion limitations disclosed.
- [ ] Backups and clean-room restoration tested.

## Novelty, rights, and research integrity

- [ ] Prior-art corpus and cutoff are recorded.
- [ ] Reviewers disclose conflicts and are calibration-scored.
- [ ] Minority reports remain accessible.
- [ ] Triviality/spam gates are implemented.
- [ ] Rights clearance covers employer, university, sponsor, grant, and collaborator claims.
- [ ] AI/human contribution record is accurate.
- [ ] Commercial licenses bind actual rights and jurisdictions.
- [ ] Negative results have an honest status and funding route.
- [ ] Marketing claims report denominators, uncertainty, and policy versions.

## Operations

- [ ] SLOs, alerts, escalation, and status communication defined.
- [ ] Key rotation and compromise drill completed.
- [ ] Checker compromise and mass revalidation drill completed.
- [ ] Vault insolvency drill completed.
- [ ] Facilitator/model/storage/chain outage drills completed.
- [ ] RPO/RTO targets demonstrated.
- [ ] SBOM, provenance, signed releases, and dependency scanning deployed.
- [ ] Bug bounty and disclosure policy active.
- [ ] On-call and legal/compliance escalation established.

## Launch decision

- [ ] All critical security gates complete.
- [ ] All economic conservation gates complete.
- [ ] All formal-verification gates complete.
- [ ] Independent legal review complete for target jurisdictions and users.
- [ ] Residual risks accepted in writing.
- [ ] Canary value and user caps configured.
- [ ] Rollback, quarantine, redemption, and public communication plans ready.
