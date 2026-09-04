# Requirement traceability matrix

Every major concept requested or developed in the design is mapped to its primary normative specification and implementation location.

| Concept | Specification / documentation | Reference implementation |
|---|---|---|
| Canonical provider-neutral protocol | `spec/018-xlmp-wire-protocol.md` | `xlemma-xlmp`, `xlmp-envelope.schema.json` |
| XLMP/1 envelope and MessageID integrity | same | `XlmpEnvelope::validate_integrity`, HTTP message ingress |
| Canonical XLMP message vocabulary | same | `XlmpMessage`, `MessageKind` |
| Protocol lifecycle | `spec/018-xlmp-wire-protocol.md`, `spec/014-api-protocol.md` | `ResearchLifecycleState`, `ensure_lifecycle_transition` |
| Research prover neutrality | `spec/007-astra-lean.md` | `ResearchProver`; ASTRA reference adapter |
| Formal-system neutrality | same | `VerifierAdapter`; Lean default adapter |
| Payment neutrality | `spec/008-x402-transport.md` | `PaymentAdapter`; x402 optional adapter |
| Transport neutrality | `spec/018-xlmp-wire-protocol.md` | `TransportAdapter`; HTTP XLMP ingress |
| Chain/finality neutrality | `spec/017-deployment-operations.md` | `FinalityAdapter` and chain reference contracts |
| Storage neutrality | `spec/015-storage-availability.md` | `StorageAdapter`, `xlemma-storage` |
| Decentralized researcher as target user | `FULL_DESIGN.md` §§1, 5–7 | `xlemma-core::ResearcherNodeManifest` |
| One researcher token, many lemma capsules | `FULL_DESIGN.md` §§1, 5–6 | `ResearcherCredit.sol`, `LemmaCapsule1155.sol` |
| Researcher identity and treasury | `spec/002-proof-rights-capsule.md`, `spec/004-researcher-credit.md` | `ResearcherNodeManifest`, `ResearchVault.sol` |
| Backed research credit | `spec/004-researcher-credit.md` | `xlemma-economics::BackedCreditLedger`, `ResearchVault.sol` |
| Pay nodes using researcher's own credit | `spec/004-researcher-credit.md`, `spec/008-x402-transport.md` | `PaymentAdapter`, `ResearchVault.authorize/settle` |
| Stable node settlement | `spec/004-researcher-credit.md` | `BackedCreditLedger::settle`, `ResearchVault.settle` |
| Profit adds future research capacity | `FULL_DESIGN.md` §7 | `allocate_revenue`, `compoundRevenue` |
| Realized profit definition | `spec/006-revenue-and-dividends.md` | `RevenueInputs`, `RevenueRouter.sol` |
| Auto-compound/cash split | `spec/006-revenue-and-dividends.md` | `allocate_revenue` |
| Formula/proof ownership correction | `docs/LEGAL_BOUNDARIES.md` | `RightsManifest`, non-transferable capsule semantics |
| Origin certificate | `spec/002-proof-rights-capsule.md` | `OriginCertificate` |
| Contribution graph | `spec/002-proof-rights-capsule.md` | `ContributionManifest` and schema |
| Human versus machine contribution | `docs/LEGAL_BOUNDARIES.md`, `spec/007-astra-lean.md` | `MachineContributionRecord`, `AstraComputeReceipt` |
| Rights manifest | `spec/002-proof-rights-capsule.md` | `RightsManifest`, `rights.schema.json` |
| Employment/university/grant clearance | `docs/LEGAL_BOUNDARIES.md` | `RightsManifest.employer_university_grant_clearance` |
| Immutable object graph | `spec/001-identifiers.md` | `xlemma-core`, `ProofRegistry.sol` |
| TheoryID | `spec/001-identifiers.md` | `TheoryId`, `theory.schema.json` |
| ClaimID from elaborated Lean type | `spec/001-identifiers.md` | `ClaimId::from_canonical_elaborated_type`, `ClaimManifest::derive_claim_id`, Lean exporter boundary |
| ProofID from proof object | `spec/001-identifiers.md` | `ProofId::from_canonical_proof_object`, `ProofManifest::derive_proof_id` |
| ArtifactID / Merkle bundle | `spec/015-storage-availability.md` | `xlemma-storage::build_bundle_manifest` |
| ReceiptID | `spec/001-identifiers.md` | `ReceiptId` and receipt structs |
| Formal equivalence edge | `spec/001-identifiers.md` | Lean example pattern; graph specification |
| ASTRA formalization | `spec/007-astra-lean.md`, `docs/ASTRA_PROMPTS.md` | `AstraProverAdapter::formalize` |
| ASTRA proof search/repair | same | `AstraProverAdapter::search_proof` |
| ASTRA LaTeX explanation | same | `AstraProverAdapter::explain` |
| ASTRA compute receipt | same | `AstraComputeReceipt` |
| ASTRA cannot self-certify | `spec/007-astra-lean.md` | service separation and receipt types |
| Lean annotation | `docs/LEAN_LATEX_GUIDE.md` | `lean/XLemma/Metadata.lean` |
| Lean export marker | same | `lean/XLemma/Export.lean` |
| Axiom inventory | `spec/007-astra-lean.md` | `#print axioms`, `xlemma-lean::extract_axioms` |
| Trusted challenge | same | `LeanVerificationRequest.trusted_challenge_path` |
| Sandboxed build | `docs/THREAT_MODEL.md` | `SandboxPolicy`, `SandboxRunner` |
| Official Lean kernel | `spec/003-poir-consensus.md` | `CheckerFamily::LeanKernel` |
| Independent checker / nanoda | same | `CheckerFamily::Nanoda`, `xlemma-lean` |
| Verification ladder | `FULL_DESIGN.md` §10 | `AssuranceLevel`, `FormalStatus` |
| LaTeX macros | `docs/LEAN_LATEX_GUIDE.md` | `latex/xlemma.sty` |
| PresentationID separation | `spec/001-identifiers.md` | presentation metadata and docs |
| Proof of Independent Reproduction | `spec/003-poir-consensus.md` | `xlemma-consensus` |
| Evidence consensus, not truth vote | same | `evaluate_formal_consensus` |
| Five consensus planes | same | `ConsensusDomain` |
| Generalized role quorum | same | `FormalConsensusPolicy` |
| Divergence quarantine | same | formal evaluator and state machine |
| Role-specific committees | `spec/009-node-roles.md` | `select_committee` |
| VRF/sortition design | same | deterministic seed-based reference selector |
| Stake as bond, not voting weight | same | `EligibleNode`, `NodeBondRegistry.sol` |
| OperatorClusterID | same | core ID and committee/evaluator checks |
| Provider/region diversity | same | policy and checker receipts |
| Commit-reveal | `spec/003-poir-consensus.md` | `observation_commitment`, `xlemma-node`, `BountyEscrow` |
| Formal exact aggregation | same | `evaluate_formal_consensus` |
| Provenance signature aggregation | `spec/002-proof-rights-capsule.md` | origin and contribution receipts |
| Novelty weighted posterior | `spec/010-novelty-significance.md` | `aggregate_novelty` |
| Significance matures to observed use | same | graph/economics design |
| Availability consensus | `spec/015-storage-availability.md` | `availability_satisfied` |
| Node role separation | `spec/009-node-roles.md` | `xlemma-node::roles_conflict`, role types and policy checks |
| Node assignment state machine | `spec/009-node-roles.md` | `xlemma-node::{validate_assignment, transition, build_observation_receipt}` |
| Verifier paid for execution | same | economics specification |
| Slash only objective misconduct | `docs/THREAT_MODEL.md` | `NodeBondRegistry.slash` comments/policy |
| Watcher/challenger nodes | `spec/013-governance-disputes.md` | certificate and bond registries |
| Verification state machine | `spec/014-api-protocol.md` | `VerificationState`, `ensure_transition` |
| x402 XLMP binding | `spec/008-x402-transport.md` | `XlmpPaymentExtension` and schema |
| HTTP 402 offer construction | `spec/008-x402-transport.md` | `xlemma-api` payment-required endpoint and `xlemma-x402` header codec |
| x402 exact | same | `PaymentScheme::Exact` |
| x402 upto | same | `PaymentScheme::Upto` |
| x402 batch settlement | same | `PaymentScheme::BatchSettlement` |
| Payment facilitator separation | same | `PaymentFacilitator` trait |
| Payment idempotency | same | `payment_identifier` |
| Reverse-direction bounty | `spec/012-bounties-and-support.md` | `BountyEscrow.sol` |
| Compute spot/forward offers | `spec/005-compute-curve.md` | `ServiceOffer` |
| Verified Proof Cost | same | `quote_verified_proof_cost` |
| Model migration spread | same | `migration_spread` |
| Research Lead Signal | same | `research_lead_signal` |
| Spot/economy/deadline/reserved routing | same | policy documentation/config |
| Compute-savings dividend | `spec/006-revenue-and-dividends.md` | `compute_savings_dividend` |
| Lower confidence bound | same | `ComputeSavingsEvidence` |
| Final dependency requirement | same | dividend guard |
| Revenue cap / no recursive explosion | same | `ComputeSavingsPolicy` |
| Bounties/grants/pre-purchase/co-development | `spec/012-bounties-and-support.md` | manifests and bounty contract |
| Negative-result funding | same | open-research pool design |
| Optional ERC-1155 proof capsule | `spec/011-tokenization.md` | `LemmaCapsule1155.sol` |
| Transferable license editions | same | token kind design |
| Separate public profit-linked wrapper | `docs/LEGAL_BOUNDARIES.md` | intentionally not implemented |
| Content-addressed storage | `spec/015-storage-availability.md` | `xlemma-storage` |
| Encrypted artifact delivery | `spec/016-privacy.md` | delivery-mode schema/design |
| Signature replay protection | `docs/THREAT_MODEL.md` | `xlemma-crypto` domains/nonces, x402 identifiers, contract nonces/IDs |
| Domain-separated signed envelopes | `docs/THREAT_MODEL.md` | `xlemma-crypto::{SignatureDomain, SignedEnvelope}` |
| Anti-triviality and spam | `spec/010-novelty-significance.md` | novelty/use gates |
| Dependency stuffing | `docs/THREAT_MODEL.md` | final dependency and cap rules |
| Self-citation rings | same | graph-analysis design |
| Append-only corrections | `spec/013-governance-disputes.md` | `ProofRegistry.supersede/quarantine` |
| Existing-chain launch | `spec/017-deployment-operations.md` | Solidity reference contracts |
| Future BFT / succinct verification | `ROADMAP.md` | planned, not falsely implemented |
| Architecture and trust diagrams | `docs/ARCHITECTURE_DIAGRAMS.md` | Mermaid system, sequence, state, graph and deployment views |
| Researcher/supporter/node workflows | `docs/RESEARCHER_USER_JOURNEYS.md` | End-to-end operational journeys |
| Governance limits and emergency powers | `docs/GOVERNANCE_CONSTITUTION.md` | Constitutional protocol constraints |
| Point-in-time data and telemetry | `docs/DATA_AND_TELEMETRY.md` | Event envelope, compute observations, learning loop |
| Production service topology | `docs/DEPLOYMENT_ARCHITECTURE.md` | Trust zones, stores, sandbox, HA and key hierarchy |
| Operator procedures and incidents | `docs/OPERATOR_RUNBOOK.md` | Daily preflight, divergence, solvency, compromise, recovery |
| Test/property/adversarial plan | `docs/TESTING_STRATEGY.md` | Unit, property, conformance, fuzz, chaos and release gates |
| Production readiness gates | `docs/PRODUCTION_CHECKLIST.md` | Formal, economic, contract, privacy, legal and operations checklist |
| Prior art and protocol differentiation | `docs/PRIOR_ART_AND_DIFFERENTIATION.md` | Comparative positioning and defensibility |
| Snapshot validation status | `docs/VALIDATION_REPORT.md` | Executed checks and explicit unexecuted native suites |
