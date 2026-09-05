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

### Open discovery reward farming and appeal capture

Attack: generate unlimited true identities, rename or split a proof, rotate
identities or rounds, fabricate costly compute or independent replication,
front-run a reveal, or collude with institutional fund administrators to claim
the shared research budget. Alternatively, overbroad grouping can exclude a
genuine foundational result; expensive or captured appeals can entrench that
exclusion. Flooded verification or repeated appeals can exhaust reserves and
prevent settlement. A successful weight appeal can also invalidate every
previously calculated reward sharing its denominator.

Required controls: separate evidence and reward decisions; published prospective
round policies; evidence-backed grouping and cross-round reward history;
independent calibration rather than self-reported compute; commit/reveal with
explicit priority rules; disclosed fund restrictions and control relationships;
bounded service admission; funded verification regardless of verdict; accessible
independent appeals; append-only reasons and remedies; and atomic conservation,
replay protection, and holds covering the full affected allocation set.

The offline `xlemma-economics::discovery` model exercises restricted budgets,
contributor splits, declared control exclusions, review capacity, outcome-neutral
fees, append-only appeal replay and whole-batch holds. Its CLI accepts synthetic
facts and cannot certify or pay. All funding, grouping, evidence verdicts,
control identities and histories remain unauthenticated inputs. Production
authentication and atomic settlement remain **unimplemented activation gates**.
Novelty, difficulty, grouping, institutional independence, and prior-art
coverage remain uncertain external assessments. Formal identity is not a
complete equivalence detector. Publish uncertainty and assess both farming and
wrongful exclusion; governance cannot invent a formal equivalence proof or vote
a failed proof into validity. Physics profiles must distinguish model proofs
and simulations from empirical support. See
[XLIP-024](../spec/024-open-research-mining.md) and its activation tests.

The committed discovery attack report deliberately records 150 simulated USDC
of leakage for an unrecognized semantic duplicate, and one legitimate exclusion
corrected by appeal. These are constructed failure cases, not measured field
error rates. Exact identity/group replay checks do not solve semantic grouping.
Pool-funded appeal capacity can still be exhausted; bounded costs cannot
replace fair admission, authenticated control mapping and independent review.

### False proof through source or environment substitution

Controls: TheoryID, exact challenge, pinned toolchain, dependency lock, artifact root, environment root, proof export, multiple checker families, and append-only receipts.

### Forged or ambiguous Lean environment export

Attack: a producer hashes pretty-printed source, injects unresolved variables,
changes binder spelling to manufacture a new identity, supplies noncanonical
JSON, hides an axiom, or pairs an export with a different toolchain/theory.

Controls: `#xlemma_export` reads the checked environment, uses a versioned
structural expression encoding, removes binder names and metadata, rejects
free variables, metavariables, unsafe/partial declarations and valueless
objects, and separately emits sorted direct constants and transitive axioms.
The Rust ingress parser rejects unknown outer fields, malformed structural
tags, noncanonical embedded JSON, duplicate evidence names, theory protocol,
encoding or toolchain mismatch, and axioms not permitted by that theory before
deriving domain-separated ClaimID and ProofID. The export still cannot certify
itself: exact artifact/dependency binding and independent checker receipts
remain mandatory.

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
commit lookup, job-specific committee-roster matching, and rejection of formal
or generalized certificates whose observations did not previously pass
authenticated XLMP ingress. Body/concurrency limits and append-only MessageID
storage bound the HTTP surface. The reference API journal fsyncs each complete
mutation before acknowledgement and links entries with canonical,
domain-separated hashes; restart replay fails closed on tampering, gaps,
duplicates, or invalid job-update predecessors. It remains a single-writer
local journal, so multi-process writes, filesystem loss, rollback to an older
whole-file snapshot, backup compromise, and HA replication require external
controls.

### Binary transport framing confusion

Controls: the reference non-HTTP frame carries exactly one length-delimited
canonical XLMP envelope, caps payloads at one MiB, rejects trailing or
truncated bytes and non-canonical JSON, and revalidates the content-derived
MessageID after decoding. Framing bytes never acquire research-consensus or
payment meaning.

### Outbound transport downgrade, redirect, or credential leakage

Controls: the reference HTTP transport accepts only explicitly allowlisted
HTTPS hosts, rejects URL-embedded credentials and fragments, disables
redirects, bounds time and response size, sends canonical XLMP media bytes,
requires an HTTP 202 response containing the exact valid envelope and
MessageID, and signs a domain-separated content-derived transport receipt.
Bearer tokens are never copied into receipts or adapter errors. Production
operators must additionally control DNS resolution, egress policy, certificate
roots, key rotation, and endpoint ownership; a hostname allowlist alone does
not eliminate DNS or infrastructure compromise.

### Artifact traversal, substitution, or overwrite

Controls: the local storage adapter rejects absolute paths, parent traversal,
duplicates, symlinks, non-regular files, hidden manifest collision, manifest
or ArtifactID mismatch, per-file and aggregate over-size payloads, and any
retrieved byte mismatch. It writes new files with create-new semantics, fsyncs
payloads and metadata, publishes through a directory rename, serializes writes
within one adapter, and refuses an existing artifact directory. This is a
single-process reference store, not proof of independent availability;
production storage needs a transactional cross-process writer, authenticated
custody challenges, replication, backups, and independent provider receipts.

### x402 authorization mutation or local replay-state loss

Controls: the reference x402 adapter content-binds the job, quote, payer,
payee, scheme, network, maximum amount, payment terms, expiry, payer
attestation, and facilitator-verified x402 envelope into one authorization ID.
Settlement checks every bound field, enforces exact/upto amount semantics,
rejects duplicate consumption, and accepts only a content-derived facilitator
receipt with the same parties, job, asset, amounts, scheme, network, and
payment identifier. The reference replay set is deliberately process-local;
real settlement requires durable transactional idempotency and reconciliation
against the facilitator or chain before restart/failover.

### Projection reordering or orphaned protocol objects

Controls: the native projection first validates every signed envelope and
content-derived object, rejects duplicate objects, then enforces accepted
theory/claim/proof/certificate, rights/contribution, finalization, publication,
revenue/dividend, artifact, and supersession prerequisites. Its deterministic
root commits to the exact accepted MessageID set. Ordering and completeness
still depend on the underlying authenticated journal or replicated log; the
projection is an index and does not replace signed source messages.

### Revenue fabrication

Controls: only finalized external settlement events enter gross revenue; costs, refunds, and reserves are deducted first; related-party demand is labeled; unrealized token changes are excluded.

### Trust-policy substitution or axiom laundering

Attack: a producer substitutes a permissive policy, mutates a registry entry,
omits an observed axiom, uses `sorry`, unsafe declarations, or compiler-trusted
evaluation, or supplies only one checker family while presenting the result as
Gold assurance.

Controls: TheoryID binds the trust policy; content-derived PolicyIDs bind every
semantic policy field; the registry root binds strictly sorted immutable
profiles and policies; proof and checker axiom inventories must be identical;
exact challenge, pinned toolchain, dependency lock, and independent
checker-family requirements fail closed. Production governance must separately
authenticate the accepted registry root—content integrity does not grant
publisher authority.

### Semantic-gap laundering

Attack: a valid proof is marketed as supporting a stronger, different, or
real-world claim by weakening the theorem, hiding assumptions, or redefining
terms.

Controls: `StatementAlignmentReceipt` binds the exact ClaimID to the informal
claim and presentation hashes, disclosed assumptions, reviewed definitions,
credentialed domain reviewers, conflicts, limitations, and signatures. Formal
and alignment statuses remain separate and challengeable.

### Dependency stuffing and royalty farming

Controls: evidence and economic graphs are separate; a final proof-term
dependency is necessary evidence but never sufficient authorization. Payment
also requires settled external revenue, an active economic policy, an eligible
economic edge, a fixed pool, a per-result cap, equivalence clustering,
graph-cycle analysis, and non-recursive event treatment. Commons has no
mandatory dependency pool. Impact authorization binds the exact revenue event
and content-derived impact evidence; settlement atomically consumes it to
prevent replay. Fixed-point checked arithmetic prevents floating-point drift or
overflow from changing a payment.

### Novelty cartel or popularity contest

Controls: cryptographically verified review and calibration evidence, unique reviewer nodes and operator clusters, independent corpus evidence, capped reviewer weights, conflict disclosure, minority reports, challenge periods, and later replacement by observed impact.

### Rights laundering

Controls: signed clearance, source-agreement roots, employer/university/grant fields, legal wrappers, dispute notices, and no implication that tokenization itself creates rights.

### Private research leakage

Controls: client-side encryption, minimal public commitments, wrapped key delivery after authorization, provider data classification, redacted receipts, and separate public/private indexes.

### Governance capture

Controls: immutable historical objects, policy-version IDs, timelocks, narrowly scoped emergency quarantine, no governance power to alter checker evidence, and forkable open specifications.

### Company or frontend disappearance

Controls: content-derived researcher portability manifests, open event-log
checkpoints, independent reconstruction clients, direct-custody vaults, and at
least two independent storage locations for every exported artifact. Origin,
contribution, verification, rights, and economic-policy records do not depend
on one proprietary database.

### Cross-layer capture hidden by aggregate branding

Controls: an eight-layer `CaptureResistanceDashboard` reports largest operator
and beneficial-owner shares, control-domain and coalition sizes, and relevant
provider, region, software, issuer, and frontend concentration. Effective
decentralization is the minimum layer score. Evidence roots and append-only
dashboard IDs prevent a coordinator from silently revising the measurement.

### Cooperative independence inflation

Controls: every research compute cooperative counts as one operator cluster
for a job. Member-share overlap between cooperatives reduces independence
credit, and undisclosed shared beneficial control is objectively challengeable
and slashable under the declared policy.

### Inflation-funded idle nodes

Controls: `NodeWorkReceipt` requires completed-work evidence, external settled
value, and a settlement receipt. Eligible revenue kinds are execution,
reserved capacity, availability, specialization, successful challenge, and
maintenance. Bond coverage caps certificate exposure; bond size is never
mathematical voting power.

## Slashing standard

Slashing requires objectively provable misconduct: equivocation, false artifact binding, fabricated evidence, unauthorized key use, false custody claim, hidden common control where disclosure was required, or failure to reveal after a binding commitment. Honest dissent and checker incompatibility are not automatically slashable.

## Residual risks

- A trusted challenge may itself be misleading or wrong.
- All checker implementations may share a logic flaw.
- The reference Lean encoder and Rust structural validator may share a
  specification defect until a clean-room encoder reproduces the vectors.
- Sandboxing may fail.
- Operator clustering can be evaded.
- A credential issuer can mis-verify uniqueness, be compromised, censor applicants, or leak private evidence.
- Novelty corpora are incomplete.
- Compute counterfactuals are model-dependent.
- Statement alignment remains a human/domain judgment and can be mistaken or captured.
- Beneficial-control confidence and credential-issuer independence cannot be proven perfectly.
- Rights disputes can exceed what on-chain evidence resolves.
- Stable backing can face issuer, chain, bridge, custody, and regulatory risk.
- Succinct proof systems add their own trusted setup, circuit, and implementation assumptions.
- The prototype HTTP ingress uses one baseline Ed25519 profile; production key rotation, threshold keys, replicated durable state, journal rollback detection anchored outside the host, and issuer-backed key resolution remain deployment work.
- Public beacon/VRF authentication and decentralized eligible-set publication are not yet integrated into the prototype service.
- Capacity, latency, hardware, reputation evidence, and operator clustering still require independent measurement and challenge infrastructure.
- The reference credential registry supplies deterministic structure and an adapter boundary; production issuer, delegation-signature, key-resolution, privacy, and accumulator-proof implementations remain deployment work.
- The concrete HTTPS, filesystem-storage, and x402 implementations are bounded
  reference adapters. They do not supply distributed finality, replicated
  availability, live facilitator assurance, or production key custody.

## September 2026 audit follow-up

The repository audit and remaining integration gaps are recorded in
[`REPOSITORY_AUDIT.md`](REPOSITORY_AUDIT.md). Additional enforced controls:

- Formal certificate IDs bind all non-signature content. API issuance validates
  the complete authenticated job evidence, including dissent, committee identity,
  exact roots, policy and challenge duration; policies cannot disable exact-root
  agreement. The single shared projection also validates restart replay.
- x402 settlement requires an exact locally issued authorization and consumes
  the underlying network/payment identifier. Recomputed wrapper IDs and renewed
  wrappers cannot bypass replay protection. A failed external call remains
  consumed because the payment may already have settled. Durable reconciliation
  is still required before retrying or replacing a process.
- Unix journal writers take an exclusive advisory lock, reject symlinks and
  non-regular files, require complete bounded newline-terminated records, and
  stop acknowledging writes after any uncertain I/O. Hash chains alone cannot
  resist an attacker who can replace the entire journal and recompute its hashes;
  externally anchored checkpoints and protected backups remain necessary.
- Storage retrieval verifies manifest identity and aggregate declared size
  before payload I/O, limits reads to declared sizes, and rejects non-regular
  descriptors. Host filesystem ownership remains trusted; this is not a complete
  defense against a concurrent attacker replacing ancestor directories.
- ASTRA native compute receipts receive new content IDs and signatures over the
  native structure. Impossible cached-token counts and overflowing charges fail.
  Signer adapters must implement `sign_compute_receipt` separately from the
  provider-specific `sign_receipt` operation.
- Revenue compounding skips allocations rounded to zero and checks the exact
  amount consumed by a vault and the final router balance. This verifies asset
  movement, not the honesty of a caller-selected vault's credit program.
- Release archives and manifests share an inventory that excludes common local
  secrets, key files, environment files except `.env.example`, runtime artifacts,
  and agent configuration. Arbitrarily named unpublished material still requires
  an explicit release review; filename filters are not a data-classification system.
