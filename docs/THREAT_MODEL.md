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
x402 gateway / facilitator
  | paid service boundary
ASTRA provider
  | model output is untrusted
proof-build sandbox
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

Controls: no-network sandbox, read-only root, explicit writable paths, process/memory/CPU/time limits, seccomp or equivalent, immutable build image, artifact size limits, and checker replay outside the sandbox. The included local runner is not a production sandbox.

### Shared checker implementation defect

Controls: independent implementations, binary digests, checker-family quorums, expanded reproduction on divergence, and periodic revalidation after software updates.

### Majority coercion

Controls: formal outcomes use exact deterministic agreement, not stake-weighted or token-weighted votes. A single required-family conflict quarantines the result.

### Sybil or common-control committee

Controls: neutral collateral, role qualification, randomized selection, operator clustering, provider/region diversity, payout-relationship analysis, and public challenge.

### Receipt copying

Controls: commit-reveal, execution-trace roots, timing analysis, custody challenges, and randomized test vectors. Copying cannot be eliminated solely cryptographically; economic and operational evidence are combined.

### Front-running a proof or bounty

Controls: claim and solution commitments, encrypted submissions, reveal deadlines, and exact artifact/salt/solver binding.

### Research-credit insolvency

Controls: backing ≥ total supply, atomic burn-and-release, no issuance against token price or expected profits, external asset reconciliation, pause circuit breaker, and invariant tests.

### Payment replay or duplicate settlement

Controls: payment identifier, nonce, expiration, network and verifying-contract binding, facilitator reconciliation, authorization state, and one-time settlement.

### Revenue fabrication

Controls: only finalized external settlement events enter gross revenue; costs, refunds, and reserves are deducted first; related-party demand is labeled; unrealized token changes are excluded.

### Dependency stuffing and royalty farming

Controls: only final proof-term dependencies, equivalence clusters, fixed upstream pool, per-result cap, graph-cycle analysis, and delayed payment based on measured use.

### Novelty cartel or popularity contest

Controls: independent corpus evidence, capped reviewer weights, calibration scoring, conflict disclosure, minority reports, challenge periods, and later replacement by observed impact.

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
- Novelty corpora are incomplete.
- Compute counterfactuals are model-dependent.
- Rights disputes can exceed what on-chain evidence resolves.
- Stable backing can face issuer, chain, bridge, custody, and regulatory risk.
- Succinct proof systems add their own trusted setup, circuit, and implementation assumptions.
