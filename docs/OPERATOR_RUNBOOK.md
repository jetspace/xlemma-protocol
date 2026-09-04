# Operator runbook

This runbook applies to reference-service, prover, builder, checker, storage, aggregator, watcher, and economic operators.

## 1. Before joining the network

- review the protocol invariants and role-conflict matrix;
- establish a legal operating entity and service terms where required;
- generate offline root and online receipt-signing keys;
- obtain a pseudonymous UserCredential and V2-or-higher OperatorCredential, then delegate one NodeCredential per node key and role lineage;
- keep raw legal/uniqueness evidence with the issuer rather than publishing it in protocol records;
- declare the conservative OperatorClusterID covering common control;
- publish role, checker/model version, binary/image digest, provider, region, capacity, price, privacy, retention, and contact metadata; provider success claims are not routing authority;
- bond neutral collateral where policy requires it;
- pass conformance and adversarial qualification suites;
- configure monitoring and incident contacts;
- demonstrate artifact retrieval and receipt reproduction.

Never create several nominal NodeIDs to evade operator-diversity requirements.

## 2. Daily preflight

1. Verify time synchronization and chain head agreement.
2. Check signing-key availability and rotation status.
3. Verify the current issuer policy, complete credential chain, status-proof freshness, and revocation-registry root.
4. Recompute checker/model image and binary digests.
5. Confirm sandbox no-network and resource controls.
6. Confirm settlement balances, bond state, and vault solvency where applicable.
7. Confirm storage replication and expiry headroom.
8. Check job queue, assignment, commit, reveal, and challenge backlogs.
9. Confirm telemetry export does not contain private proof content or secrets.
10. Run a known-answer test against each active checker family.
11. Verify protocol success-calibration signatures/expiry and impact-pool authorization state where those services are enabled.

## 3. Job handling

### Accept assignment

Validate:

- assignment signature and randomness reference;
- role eligibility;
- exact VerifiedUserID → OperatorID → NodeID delegation, minimum tier, role qualification, and non-revocation proof;
- no same-job conflict of interest;
- exact JobID, ClaimID, ArtifactID, TheoryID, and PolicyID;
- payment/fee terms;
- deadline and resource limits;
- artifact availability and content hash.

Reject rather than silently substituting a dependency, model, checker, or environment.

For statement-alignment assignments, bind the exact ClaimID, informal-claim
hash, presentation hash, assumptions, definitions, reviewer credential and
conflicts. Never infer alignment from a successful Lean run.

### Execute

- fetch all inputs by content hash;
- create an ephemeral sandbox;
- verify image and binary digests;
- disable network unless the policy explicitly requires a build-fetch stage separated from final checking;
- execute under deterministic limits;
- capture exit status, normalized outputs, axiom inventory, roots, resource usage, and trace commitment;
- destroy scratch data according to retention policy.

### Commit and reveal

- calculate the observation root over the complete normalized receipt;
- generate a fresh random salt;
- submit the commitment before observing peer reveals;
- keep the receipt and salt durable until the reveal deadline;
- reveal exactly the committed verdict and observation root;
- sign the final receipt with the registered operational key.

Do not copy another node's outcome or change a result to align with a majority.

## 4. Checker divergence procedure

1. Stop automatic finalization for the affected job.
2. Mark it `DIVERGENT` and preserve every receipt.
3. Compare artifact, environment, dependency, axiom, checker binary, and execution trace roots.
4. Re-run in clean environments operated by additional independent clusters.
5. Determine whether divergence comes from input mismatch, nondeterminism, implementation defect, malicious artifact, or dishonest evidence.
6. Quarantine the affected certificate/checker digest class.
7. Publish a scoped evidence root and remediation plan.
8. Revalidate affected historical certificates after a checker or environment defect.
9. Slash only when objective misconduct is reproducible.

Never resolve divergence by counting votes.

## 5. ASTRA/model outage

- stop accepting deadlines that require the unavailable provider;
- route to allowed alternative prover adapters when the researcher's policy permits;
- preserve already-generated candidates and receipts;
- do not change verification policy;
- refund unused `upto` authorization;
- update compute curves with the lost capacity and observed failure rate;
- avoid sending private context to a substitute provider without explicit authorization.

## 6. Payment/facilitator outage

- preserve unpaid job state without marking verification complete economically;
- do not re-charge an existing payment identifier;
- prevent service delivery beyond the authorized policy;
- reconcile pending chain transactions against a locally verified ledger;
- fail over only to a facilitator/network allowed by the original quote;
- keep formal verification records independent from payment availability.

## 7. Vault solvency alarm

If `backing < credit total supply` or reconciliation is uncertain:

1. pause new deposits, authorizations, compounding, and redemption according to contract capability;
2. preserve chain, asset, and accounting evidence;
3. determine whether the cause is token behavior, transfer fee/rebase, compromised role, accounting mismatch, or external asset impairment;
4. notify affected researchers and governance;
5. do not mint replacement credits without new backing;
6. execute the audited recovery/exit procedure;
7. publish post-incident reconciliation.

Settlement assets with transfer fees, rebasing, blacklisting, or unusual callbacks require explicit compatibility analysis.

## 8. Checker compromise

- revoke or disable the binary/image digest for new work;
- quarantine certificates whose policy relied on the affected checker;
- identify the first vulnerable version and complete affected range;
- run independent clean replay;
- rotate credentials if signing integrity may be affected;
- publish CVE/evidence details when safe;
- compensate honest challengers according to policy;
- do not erase old receipts.

## 9. Key compromise

1. Append the exact user, operator, or node credential revocation and distribute the new registry root.
2. Stop the compromised role at gateways and chain registries.
3. Rotate to a new operational key signed by the offline root/threshold.
4. identify every receipt or transaction signed during the exposure interval;
5. quarantine or revalidate affected objects;
6. preserve forensic evidence and notify counterparties;
7. delegate a new NodeCredential while preserving the same VerifiedUserID, OperatorID, and OperatorClusterID rather than pretending a new key is a new operator.

## 10. Storage failure

- test actual retrieval, not only provider API status;
- challenge custody using random content ranges or policy-defined proofs;
- replace failed replicas across distinct operators/providers/regions;
- renew receipts before expiry;
- quarantine availability status if threshold is not met;
- do not claim formal invalidity merely because an artifact is temporarily unavailable;
- retain enough local evidence to reconstruct public roots.

## 11. Maintenance windows

- publish start/end times and affected roles;
- stop accepting jobs whose deadlines overlap capacity loss;
- allow assigned jobs to complete or explicitly reassign before commitment;
- preserve committee randomness and conflict constraints during reassignment;
- run known-answer tests after restart;
- issue new receipts when binary or environment digests change.

## 12. Metrics and alerts

Critical alerts:

- vault backing ratio below 1.0 or unknown;
- duplicate payment identifier settlement;
- required checker-family divergence;
- missing/late reveal;
- operator/provider/region diversity below policy;
- certificate finalized during open challenge;
- artifact root mismatch;
- unsupported axiom detected;
- storage threshold failure;
- unauthorized contract role change;
- sudden node/operator concentration;
- expired credential/status proof, issuer failure, revocation-root mismatch, or unexplained user/operator linkage change;
- anomalous proof pass-rate or cost shift;
- private content in logs.

## 13. Backup and recovery

Back up:

- signed protocol events;
- mutable job database and transaction outbox;
- content-addressed manifests and storage maps;
- chain index and reconciliation checkpoints;
- encrypted key material under documented recovery controls;
- policy registry snapshots;
- checker images/binaries and lockfiles;
- incident and challenge evidence.

Quarterly, restore into a clean environment and verify that a historical certificate can be reproduced without the primary operator's database.

## 14. Decommissioning

- stop new offers and assignments;
- finish or reassign existing jobs;
- honor reveal and storage obligations;
- request unbond only after the policy's exposure period;
- publish signed retirement metadata;
- preserve receipts and public artifacts;
- rotate or destroy operational keys;
- do not delete historical operator-cluster links.
