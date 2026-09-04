# Testing and verification strategy

The reference repository is not complete merely because it compiles. xLemma needs cross-layer tests that attack formal, economic, distributed-systems, privacy, and smart-contract assumptions.

## 1. Test pyramid

### Unit tests

Cover canonicalization, typed IDs, XLMP envelope integrity and message round trips, protocol lifecycle transitions, money arithmetic, contribution shares, committee eligibility, commit-reveal, formal quorum, novelty aggregation, quote construction, backing conservation, revenue allocation, dividend caps, payment-adapter encoding, and storage path safety.

### Property tests

Required properties include:

- domain-separated IDs never collide for the same serialized input across object types;
- object key order does not change canonical hashes;
- credit ledger backing never falls below outstanding supply after any valid operation sequence;
- failed operations do not partially mutate state;
- revenue allocations plus explicit rounding remainder equal net distributable revenue;
- dividends never exceed policy or downstream-revenue caps;
- committee selection never chooses two NodeIDs from the same operator cluster for one job;
- required checker-family disagreement never returns `Certified`;
- state transitions outside the transition table always fail;
- artifact repacking with identical content/environment yields the same identity;
- unsafe or symlink-escaping bundle paths always fail.
- any mutation of signed XLMP identity material invalidates its MessageID;
- every supported transport decodes to the same canonical XLMP envelope;
- no payment, finality, prover, verifier, or storage adapter can advance an XLMP state without the required protocol evidence.

### Integration tests

- `ResearchProver` candidate to default Lean adapter to independent check to PoIR certificate;
- XLMP envelope conformance over HTTP plus at least one non-HTTP transport;
- x402 and a non-x402 `PaymentAdapter` authorization, actual settlement, and refund;
- external revenue routed partly to cash and partly to vault compounding;
- bounty commitment, final certificate, release delay, and payout;
- challenge, quarantine, remediation, and revalidation;
- encrypted artifact purchase and independent local verification;
- storage expiry and replica replacement.

### Conformance tests

Publish language-neutral JSON test vectors for every XLMP message, canonical ID, receipt, signature domain, lifecycle transition, committee seed, commitment, certificate, revenue allocation, and bundle root. Independent implementations must produce byte-identical outputs.

## 2. Lean corpus

The checker test corpus must contain:

- valid constructive theorems;
- valid theorems using allowed classical axioms;
- `sorry`/placeholder attempts;
- unapproved custom axioms;
- exact-challenge mismatches;
- theorem-name and namespace confusion;
- misleading informal descriptions;
- dependency substitution;
- malicious build scripts;
- resource-exhaustion proofs;
- nondeterministic/environment-sensitive builds;
- cases accepted by one implementation and rejected by another;
- proof-object corruption;
- version and universe edge cases.

A Gold release is blocked until the corpus passes through the official Lean path and at least one independently implemented checker.

## 3. Consensus adversarial tests

Simulate:

- one operator registering many NodeIDs;
- correlated cloud/provider failures;
- committee bribery and selective denial of service;
- copied reveals;
- withheld reveals;
- mixed-job receipts;
- root mismatches;
- honest checker divergence;
- fabricated trace roots;
- randomness grinding;
- late operator-cluster reclassification;
- challenge spam;
- aggregator equivocation;
- chain reorganization during certificate submission.

Success criterion: the system fails closed without treating stake or majority count as proof validity.

## 4. Economic invariant tests

Use stateful fuzzing against both Rust and Solidity implementations.

Primary invariant:

\[
\operatorname{vaultBacking} \geq \operatorname{creditTotalSupply}
\]

Additional invariants:

- one authorization settles or cancels at most once;
- actual settlement never exceeds maximum authorization;
- settlement goes only to the pre-authorized payee;
- unused credits are returned exactly;
- compounding mints credits only after matching settlement assets arrive;
- revenue event identifiers cannot be replayed;
- waterfall shares equal 10,000 basis points;
- duplicate contributors cannot capture rounding or double allocations;
- a bounty cannot pay before a final matching claim/policy certificate;
- a quarantined certificate cannot release a bounty;
- dependency rewards never exceed downstream net revenue or policy cap;
- no protocol path books token appreciation as revenue.

## 5. Smart-contract testing

Run:

- Foundry unit and invariant tests;
- fuzz tests over amounts, deadlines, role changes, and recipient arrays;
- Slither and other static analyzers;
- symbolic execution for authorization and settlement paths;
- storage-layout checks for any upgradeable component;
- ERC compatibility tests;
- malicious/rebasing/fee-on-transfer token tests;
- reentrancy and callback tests;
- chain-fork tests for target networks;
- independent manual audit and public contest before uncapped deployment.

The included contracts are reference implementations and must not be used with real value before these gates pass.

## 6. Payment-adapter testing

- common authorization and settlement contract across x402, backed research credit, grant/escrow, and invoicing adapters;
- x402 exact, upto, and batch-settlement flows;
- malformed and oversized headers;
- network/asset/payee mismatch;
- expired quote or authorization;
- duplicate payment identifier;
- facilitator timeout and retry;
- settlement succeeds but response is lost;
- refund and reconciliation;
- payment succeeds while proof generation fails;
- proof succeeds while payment settlement is delayed;
- multiple facilitator disagreement;
- private metadata leakage in extensions.

Payment success must never set formal status by itself.

## 7. XLMP transport and downgrade testing

- all twelve XLMP/1 message discriminators validate against the canonical schema;
- unsupported protocol names and major versions fail closed;
- unknown required fields fail schema validation;
- MessageID mutation, sender substitution, correlation substitution, and replay are rejected;
- observation reveal fields match their prior commit;
- HTTP, WebSocket, libp2p, and chain adapters preserve canonical message identity;
- x402 extension payloads bind the exact XLMP MessageID;
- adapter-specific metadata cannot be interpreted as proof validity or finalization evidence.

## 8. Research-prover evaluation

Evaluate ASTRA and alternative prover adapters by domain and point in time:

- formal target acceptance by a human researcher;
- Lean build rate;
- Gold verification rate;
- cost per Gold result;
- novelty clearance;
- context and tool use;
- repair-cycle count;
- hallucinated definitions/imports;
- private-data handling;
- reproducibility under a pinned model snapshot when available;
- calibration of success and cost estimates.

Do not optimize only for pass rate; include statement correctness, novelty, compute cost, and human oversight.

## 9. Privacy/security testing

- prompt and proof contents absent from routine logs;
- per-job encryption key isolation;
- unauthorized node cannot retrieve private bundle;
- expired grants cease access;
- ciphertext/root swap is detected;
- access-pattern leakage assessed;
- secrets scanner passes;
- dependency and container supply chain scanned;
- sandbox escape attempts blocked;
- filesystem traversal and symlink attacks blocked;
- denial-of-service limits enforced.

## 10. Chaos and recovery testing

Inject:

- model-provider outage;
- checker node loss after commit;
- database failover;
- queue duplication;
- storage-region loss;
- chain RPC disagreement;
- facilitator outage;
- key rotation during a job;
- certificate challenge at the finalization boundary;
- clock skew;
- partial network partitions.

Verify exact-once economic effects, at-least-once safe workflow handling, and preservation of signed evidence.

## 11. Performance testing

Measure independently:

- API request latency;
- queue and committee assignment latency;
- ASTRA generation latency and cost;
- Lean build/check latency by artifact size;
- commit/reveal completion;
- chain finality and challenge delays;
- storage retrieval;
- index graph queries;
- reconnect/retry storms;
- maximum safe concurrent sandbox count.

No latency target should cause the protocol to weaken checker diversity or skip challenge windows.

## 12. Release gates

A release candidate requires:

- formatting, lint, and unit tests;
- property/fuzz suites;
- schema and OpenAPI validation;
- deterministic test vectors;
- cross-checker corpus pass;
- reproducible source archive and checksums;
- dependency/SBOM scan;
- threat-model and traceability updates;
- migration/recovery test;
- documented unresolved risks.

A production economic release additionally requires independent audits, legal review, staged value caps, incident drills, and live monitoring.
