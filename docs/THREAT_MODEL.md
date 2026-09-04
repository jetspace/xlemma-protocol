# Threat model

## Assets to protect

- exact formal claim and proof identity;
- trusted challenge integrity;
- researcher priority and contribution history;
- model prompts, unpublished proofs, and confidential data;
- neutral backing in researcher vaults;
- correctness of credit supply and revenue allocation;
- node private keys and operator identity records;
- committee randomness and selection proofs;
- checker binaries, sandbox images, and dependency roots;
- payment authorization, idempotency, and settlement state;
- availability of proof artifacts and dissenting receipts.

## Trust boundaries

```text
researcher client
  | untrusted network
XLMP message router
  | transport adapter boundary
node discovery / append-only order book
  | untrusted market-data and capacity boundary
credential registry / issuer and delegation verification
  | private-evidence, key-resolution, uniqueness, and revocation boundary
payment adapter / facilitator
  | separate paid service boundary
research prover / ASTRA provider
  | model output is untrusted
verifier adapter / proof-build sandbox
  | hostile code boundary
exported proof object
  | replay outside sandbox
checker nodes
  | independent operator boundary
certificate aggregator
  | cannot rewrite observations
base chain / contracts
  | economic finality only
storage network
  | availability and confidentiality boundary
```

## Principal threats and controls

### False proof through source or environment substitution

Controls: TheoryID, exact challenge, pinned toolchain, dependency lock, artifact root, environment root, proof export, multiple checker families, and append-only receipts.

### Malicious Lean metaprogram or build script

Controls: no-network sandbox, read-only root, explicit writable paths, process/memory/CPU/time limits, seccomp or equivalent, immutable build image, artifact size limits, bounded receipt output, and checker replay outside the sandbox. `LocalCommandRunner` refuses all execution; a separately administered `SandboxRunner` and receipt signer are mandatory.

### Shared checker implementation defect

Controls: independent implementations, binary digests, checker-family quorums, expanded reproduction on divergence, and periodic revalidation after software updates.

### Majority coercion

Controls: formal outcomes use exact deterministic agreement, not stake-weighted or token-weighted votes. A single required-family conflict quarantines the result.

### Sybil or common-control committee

Controls: privacy-preserving verified-participant credentials, exact participant-to-operator-to-node delegation, fresh non-revocation proofs, neutral collateral, role qualification, committed eligible-set roots, future authenticated randomness, deterministic hash ranking, unique VerifiedUserIDs, OperatorIDs, and operator clusters, provider/region diversity, payout-relationship analysis, exact selection proofs, and public challenge. Bond, credential tier, and reputation are eligibility gates, never sortition or formal-vote weight.

### Credential forgery, issuer capture, or stale revocation

Controls: content-derived credential and revocation IDs, deployment-provided
issuer/delegation signature verification, nested validity intervals,
short-lived status proofs bound to an exact append-only registry root, multiple
accepted issuers, and public challenge. Raw legal or uniqueness evidence stays
with the issuer and is not placed in XLMP messages. Revocation preserves
history and removes eligibility; it does not rewrite old observations.

### False node-market advertisement or discovery manipulation

Controls: content-derived AdvertisementIDs, cryptographically verified bounded validity windows,
monotonic append-only supersession, exact price units/assets/decimals, checked
integer arithmetic, capacity and latency constraints, checker-family binding,
evidence-backed reputation snapshots and bonds, reproducible ordered discovery
results, and multiple independent discovery/index providers. A service match is
bound to one exact advertisement sequence and cannot inherit a silent revision.

### Reputation laundering or scalar-score capture

Controls: eight separately evidenced dimensions, minimum sample sizes,
policy-specific per-dimension gates, explicit supersession, operator-cluster
binding, and no universal composite authority score. Honest dissent and valid
`FAIL` observations are not penalized merely for disagreeing with a majority.

### Sortition grinding or input mutation

Controls: eligible-set commitment before reveal, a declared future beacon/VRF
round, domain-separated ranks, bounded reference inputs, published per-member
rank hashes and selection root, and exact third-party reproduction. Production
deployments must authenticate the beacon proof; the seed commitment alone is
insufficient.

### Receipt copying

Controls: commit-reveal, execution-trace roots, timing analysis, custody challenges, and randomized test vectors. Copying cannot be eliminated solely cryptographically; economic and operational evidence are combined.

### Front-running a proof or bounty

Controls: content-derived proof, certificate, and bounty identifiers; account-namespaced vault registration; claim and solution commitments; encrypted submissions; reveal deadlines; and exact artifact/salt/solver/certificate binding.

### Research-credit insolvency

Controls: backing ≥ total supply, vault-address-only mint/burn authority, internally owned authorization records, atomic burn-and-release, exact-balance rejection of fee/rebase behavior, no issuance against token price or expected profits, external asset reconciliation, and invariant tests.

### Payment replay or duplicate settlement

Controls: payment identifier, nonce, expiration, network and verifying-contract binding, facilitator reconciliation, authorization state, and one-time settlement.

### Adapter substitution or protocol downgrade

Controls: the XLMP protocol name, major version, MessageID, signer, policy roots, and payload are bound together; unknown major versions and unknown required fields fail closed; payment, transport, finality, storage, prover, and verifier receipts remain separate. An adapter cannot translate a message into weaker research semantics without changing its MessageID and invalidating its signature.

### Forged or unauthenticated XLMP envelope

Controls: mandatory API bearer authentication outside `/health`, an explicit
Ed25519 sender allowlist, a NodeID-to-signer map, domain-separated signing
bytes, cryptographically verified inner observation signatures, prior signed
commit lookup, job-specific committee-roster matching, body/concurrency limits,
and append-only MessageID storage. The reference nonce and state stores remain
in-memory and must be replaced by durable transactional stores before HA use.

### Revenue fabrication

Controls: only finalized external settlement events enter gross revenue; costs, refunds, and reserves are deducted first; related-party demand is labeled; unrealized token changes are excluded.

### Dependency stuffing and royalty farming

Controls: only final proof-term dependencies, equivalence clusters, fixed upstream pool, per-result cap, graph-cycle analysis, and delayed payment based on measured use.

### Novelty cartel or popularity contest

Controls: cryptographically verified review and calibration evidence, unique reviewer nodes and operator clusters, independent corpus evidence, capped reviewer weights, conflict disclosure, minority reports, challenge periods, and later replacement by observed impact.

### Rights laundering

Controls: signed clearance, source-agreement roots, employer/university/grant fields, legal wrappers, dispute notices, and no implication that tokenization itself creates rights.

### Private research leakage

Controls: client-side encryption, minimal public commitments, wrapped key delivery after authorization, provider data classification, redacted receipts, and separate public/private indexes.

### Governance capture

Controls: immutable historical objects, policy-version IDs, timelocks, narrowly scoped emergency quarantine, no governance power to alter checker evidence, and forkable open specifications.

## Slashing standard

Slashing requires objectively provable misconduct: equivocation, false artifact binding, fabricated evidence, unauthorized key use, false custody claim, hidden common control where disclosure was required, or failure to reveal after a binding commitment. Honest dissent and checker incompatibility are not automatically slashable.

## Residual risks

- A trusted challenge may itself be misleading or wrong.
- All checker implementations may share a logic flaw.
- Sandboxing may fail.
- Operator clustering can be evaded.
- A credential issuer can mis-verify uniqueness, be compromised, censor applicants, or leak private evidence.
- Novelty corpora are incomplete.
- Compute counterfactuals are model-dependent.
- Rights disputes can exceed what on-chain evidence resolves.
- Stable backing can face issuer, chain, bridge, custody, and regulatory risk.
- Succinct proof systems add their own trusted setup, circuit, and implementation assumptions.
- The prototype HTTP ingress uses one baseline Ed25519 profile; production key rotation, threshold keys, durable replay state, and issuer-backed key resolution remain deployment work.
- Public beacon/VRF authentication and decentralized eligible-set publication are not yet integrated into the prototype service.
- Capacity, latency, hardware, reputation evidence, and operator clustering still require independent measurement and challenge infrastructure.
- The reference credential registry supplies deterministic structure and an adapter boundary; production issuer, delegation-signature, key-resolution, privacy, and accumulator-proof implementations remain deployment work.
