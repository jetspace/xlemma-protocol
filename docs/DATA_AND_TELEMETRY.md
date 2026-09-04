# Data, telemetry, and learning architecture

xLemma requires enough telemetry to price proof work, reproduce certificates, reward useful dependencies, detect manipulation, and improve prover systems without leaking private research.

## 1. Data classes

| Class | Examples | Default treatment |
|---|---|---|
| Public protocol metadata | IDs, policy roots, public receipts, status | append-only public index |
| Formal artifacts | Lean source, proof object, dependency graph | content-addressed; public or encrypted |
| Private research content | unpublished formula, manuscript, data | encrypted; least-privilege access |
| Compute telemetry | units, latency, retries, model/checker digest | signed and minimally disclosed |
| Economic records | quote, authorization, settlement, revenue route | ledger-bound; privacy policy applies |
| Security evidence | traces, sandbox logs, challenge bundles | restricted until disclosure is safe |
| Identity/compliance data | optional KYC, tax, sanctions records | off-chain segregated controller |
| Training trajectories | proof states, actions, failures, feedback | explicit consent/license and redaction |

## 2. Event envelope

Every durable event should include:

```text
event_id
protocol_version
event_type
aggregate_id
aggregate_version
occurred_at
observed_at
actor_id
verified_user_id when consensus-relevant
operator_id when consensus-relevant
operator_cluster_id when applicable
credential_chain_root when consensus-relevant
policy_id
payload_hash
previous_event_hash
signature
privacy_class
retention_class
```

A database row is an index, not the authoritative artifact. The signed event and content-addressed payload remain independently exportable.

## 3. Point-in-time integrity

Training, pricing, novelty review, and performance evaluation must preserve what was known at each time. Store:

- source publication and retrieval timestamps;
- model/checker version and binary digest;
- prompt, context, tool, and dependency roots;
- policy version;
- market quote and capacity window;
- challenge status at evaluation time;
- later corrections separately.

Never overwrite prior-art corpora or relabel historical outcomes without retaining the original state.

## 4. Compute-curve observations

For each service execution, record:

```text
service_type
domain
difficulty_features
provider_offer_id
model_or_checker_id
hardware_or_service_class
queue_time
execution_time
input_units
cached_input_units
output_units
tool_calls
failed_attempts
memory_peak
actual_settlement_cost
artifact_accepted
Gold outcome
novelty outcome
revalidation outcome
```

Prices and success probabilities must be estimated separately. Providers may
publish prices and capacity, but an independent protocol estimator derives
success probability from audited history. A high per-token ASTRA price may
still yield a lower Quality-Adjusted Certification Cost if it materially
improves acceptance probability.

## 5. Difficulty and proof-state features

Candidate features include:

- elaborated type size and universe complexity;
- number and type of hypotheses;
- dependency graph depth;
- available relevant lemmas;
- tactic-state token length;
- branching factor and failed branches;
- number of compiler diagnostics;
- repair cycles;
- axiom restrictions;
- domain taxonomy;
- expected independent checker complexity;
- novelty corpus density;
- privacy and sandbox requirements.

Model features are advisory. They cannot replace deterministic final checking.

## 6. Proof-yield metrics

Track at least:

- formalization acceptance rate;
- candidate build rate;
- official-kernel pass rate;
- independent-checker agreement rate;
- Gold certification rate;
- novelty clearance rate;
- cost per accepted artifact;
- latency distribution;
- revalidation survival;
- challenge and reversal rates;
- downstream reuse;
- measured compute savings;
- calibration of quoted success probabilities.

Report denominators, abstentions, policy versions, and confidence intervals. Do not market cherry-picked success rates.

## 7. Compute-impact measurement

The preferred hierarchy is:

1. randomized lemma-withholding re-proving, where ethically and economically feasible;
2. matched repeated tasks under pinned environments;
3. causal adjustment using pre-registered features;
4. conservative lower confidence bounds;
5. capped observational estimates when experiments are impossible.

Required safeguards:

- count only final proof dependencies;
- cluster formally equivalent or near-duplicate lemmas;
- detect cycles and self-citation rings;
- treat savings as one uncertain impact signal, not a precise invoice;
- require a separately authorized eligible economic edge and bounded pool;
- cap allocations by realized downstream net revenue and remaining pool budget;
- prevent recursive treatment of one revenue event;
- publish methodology and uncertainty;
- preserve negative estimates;
- require minimum sample and effect thresholds.

## 8. ASTRA learning loop

With contributor authorization, collect:

```text
formal target
retrieved context root
proof-state trajectory
candidate actions
Lean diagnostics
repair actions
accepted proof term
rejected alternatives
human edits and selection
cost and latency
verification outcome
novelty outcome
```

Training exports must remove secrets, personal data, and restricted artifacts; preserve license and consent metadata; and include negative trajectories rather than only successful proofs.

## 9. Privacy controls

- encrypt unpublished bundles before remote storage;
- use per-job keys and short-lived access grants;
- keep prompts and proof contents out of ordinary logs;
- separate identity/compliance data from research graph data;
- keep legal names, identity documents, addresses, biometrics, and uniqueness evidence with the credential issuer; public telemetry carries only pseudonymous typed IDs and commitments;
- support private proofs with public commitments and selectively disclosed verification evidence;
- publish only aggregated compute metrics where exact values leak research direction;
- permit researchers to choose provider and jurisdiction constraints;
- document unavoidable leakage from access patterns, prices, dependencies, and timing.

Deletion requests can remove mutable indexes or encryption keys where legally appropriate, but cannot erase immutable public commitments already finalized on a chain. The interface must disclose this before publication.

## 10. Observability

Operational metrics:

- job queue depth and age;
- quote acceptance and expiry;
- vault backing ratio;
- authorization lock totals;
- proof-build success rate;
- committee assignment latency;
- commit/reveal completion;
- checker-family divergence;
- challenge rate;
- certificate finalization latency;
- artifact replication and expiry;
- payment settlement/reconciliation failures;
- node/operator concentration;
- model/provider concentration;
- API error and latency distributions.

Security logs must be tamper-evident, access-controlled, retention-limited, and linked to public evidence only when disclosure is safe.

## 11. Data retention

Suggested classes:

- permanent: public manifests, receipts, certificates, supersession and challenge roots;
- long-term: reproducible build inputs, checker binaries/images, dependency locks;
- policy-defined: metering records and economic reconciliation;
- consent-defined: ASTRA trajectories and private research data;
- minimal: raw secrets, transient tokens, unredacted prompts, sandbox scratch space.

A retention policy is part of each service offer and must be known before x402 authorization.
