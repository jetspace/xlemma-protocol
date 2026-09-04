# Deployment architecture

This is a production-oriented decomposition, not a claim that the reference implementation is production-ready.

## 1. Recommended launch topology

Launch on an established settlement chain rather than building a new L1. Keep formal verification and evidence production off chain; anchor compact certificate, policy, artifact, payment, and revenue roots on chain.

XLMP/1 is the service-to-service contract. Core services and adapters:

- XLMP message router with HTTP, libp2p, or WebSocket transport adapters;
- append-only participant/operator/node credential and revocation registry with issuer/key-resolution adapters;
- optional x402 and other payment adapters;
- researcher identity/manifest service;
- quote and compute-curve service;
- durable job orchestrator;
- `ResearchProver` workers, with ASTRA as the reference adapter;
- `VerifierAdapter` build workers, with Lean as the default;
- official-kernel checker network;
- independent-checker network;
- committee selection and receipt aggregator;
- novelty/review network;
- watcher/challenge service;
- content-addressed storage and availability service;
- finality-adapter chain writer/indexer;
- vault/revenue reconciler;
- public explorer;
- metrics, security monitoring, and incident tooling.

## 2. Trust-zone separation

### Public edge

Terminates TLS, applies DDoS controls, request size limits, rate limits, authentication, and request identifiers. It must not hold long-lived model, chain, or vault keys.

### Control plane

Stores job metadata, policy selection, committee assignments, deadlines, and idempotency state. Use durable queues and transactional outboxes. The control plane cannot fabricate checker receipts because node signatures and artifact roots are independently verified.

### Identity plane

Publishes pseudonymous UserCredential, OperatorCredential, NodeCredential, and
revocation records. Issuer adapters verify private identity/uniqueness evidence
off protocol; key-resolution adapters authenticate issuer and delegation
signatures. The public service retains only commitments, coarse qualifications,
validity windows, and short-lived status proofs. Separate issuers and threshold
registry operation reduce identity-provider capture.

### Prover plane

May access approved model APIs and private research context. It must be isolated from final checker credentials and should use per-job secrets and egress policies.

### Builder/checker plane

Runs untrusted artifacts in ephemeral no-network sandboxes. Builder and checker pools must use separate credentials, images, and operators. Checker images are content-addressed and reproducible.

### Economic plane

Holds chain signing, facilitator, vault, and reconciliation components. Use hardware-backed threshold signing, withdrawal limits, allowlisted contracts, and independent accounting alerts.

### Storage plane

Stores encrypted/public bundles and serves availability challenges. Storage receipts must identify operator cluster, provider, region, custody root, and expiry.

## 3. Data stores

- relational database for mutable indexes and workflow state;
- durable message bus for jobs and observations;
- append-only event store for signed protocol events;
- object/content-addressed storage for proof bundles;
- graph index for dependencies, contribution, equivalence, and revenue routes;
- time-series store for compute curves and operations;
- secret manager/HSM for operational credentials.

No mutable store substitutes for the signed protocol object.

## 4. Sandbox baseline

Production Lean/checker sandboxes should have:

- no network by default;
- read-only base image;
- per-job ephemeral filesystem;
- fixed CPU, memory, process, file, and wall-clock limits;
- seccomp or equivalent syscall filtering;
- unprivileged UID and no host mounts;
- dependency inputs copied by hash;
- output allowlist;
- deterministic environment variables and locale;
- kill-on-timeout and guaranteed cleanup;
- signed image and binary digests;
- separate export/replay checker outside any build-script trust boundary.

## 5. High availability

At minimum:

- multiple stateless API replicas;
- replicated job queue and database with point-in-time recovery;
- geographically and administratively diverse checker nodes;
- multiple payment/facilitator routes where supported;
- several content-storage providers and regions;
- redundant chain RPC providers and a locally verified index;
- primary/shadow aggregators with epoch fencing;
- independent watcher infrastructure.

A loss of one prover adapter should degrade only the work routed to that provider, not existing proof verification or XLMP interoperability. A payment-adapter outage should not alter certificate state. A finality-adapter outage should delay anchoring without rewriting signed observations.

## 6. Key hierarchy

```text
offline governance root
  ├── contract administration threshold
  ├── policy-registry threshold
  ├── emergency pause/quarantine threshold
  └── release-signing threshold

researcher identity root
  ├── contribution signing key
  ├── publication key
  ├── vault operations key
  └── short-lived agent/session keys

node operator root
  ├── verified participant / operator delegation key
  ├── node credential and receipt signing key
  ├── infrastructure attestation key
  └── rotation certificates
```

Separate keys by role and environment. Publish credential rotations and revocations as signed append-only events without changing the participant's independence domain.

## 7. Chain deployment order

1. settlement-asset allowlist and oracle policy, if any;
2. node bond registry;
3. PoIR certificate registry;
4. proof registry;
5. researcher vault implementation and factory;
6. revenue router;
7. bounty escrow;
8. optional ERC-1155 capsule/license registry;
9. indexer and public explorer;
10. role transfer to threshold-controlled governance.

Contracts must be independently audited before real funds. Use staged caps and emergency exits.

## 8. Environments

- **local:** mocked settlement and model providers; deterministic fixtures;
- **devnet:** public test chain, nonvaluable credits, adversarial test nodes;
- **canary:** limited researchers, strict value caps, mandatory manual review;
- **beta:** audited contracts, broader nodes, published SLOs, still capped;
- **production:** security, legal, economic, incident, and governance gates completed.

Never move directly from local reference code to uncapped production.

## 9. Suggested service-level objectives

These are starting targets, not guarantees:

- API availability: 99.9% initially;
- no acknowledged payment without durable idempotency record;
- no certificate submission without complete signed observation set;
- 100% daily vault solvency reconciliation;
- p99 committee assignment under 60 seconds when sufficient capacity exists;
- no unacknowledged checker divergence;
- artifact availability policy satisfaction above 99.99% for active Gold capsules;
- recovery point objective under five minutes for mutable workflow state;
- cryptographic artifacts and chain state recoverable independently of the primary database.

## 10. Supply-chain security

- pin compiler, Lean, mathlib, checker, Rust, Solidity, and container versions;
- generate SBOMs and provenance attestations;
- verify dependencies and lockfiles;
- sign releases and container images;
- require two-person review for consensus/economic changes;
- run static analysis, fuzzing, sanitizer, and dependency scans;
- reproduce release builds in two environments;
- maintain checker test corpora containing valid, invalid, malicious, and divergent cases.
