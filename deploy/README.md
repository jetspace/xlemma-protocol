# Deployment reference

The compose file illustrates the target control-plane dependencies—API, PostgreSQL, and authenticated NATS JetStream. The Rust API now writes protocol mutations to a persistent, hash-chained single-writer journal at `XLEMMA_EVENT_LOG_PATH`; it does not yet use PostgreSQL or NATS for replicated HA state. A content-storage service is intentionally not bundled: the formerly referenced MinIO community image is not available for its latest security release. Configure a separately maintained, digest-pinned storage adapter instead.

Formal verification workers should not run inside the API container. Deploy separate isolated worker pools for ASTRA generation, Lean builds, official kernel checks, independent checks, novelty review, and storage. High-assurance jobs require distinct operator clusters and should not share a privileged orchestrator host.

Before production:

- keep the committed image digests current and pin every separately deployed adapter by immutable digest;
- replace local secrets with a secret manager/HSM;
- add mTLS and workload identity;
- enforce egress-denied build sandboxes;
- use a production x402 facilitator or self-facilitation path appropriate to the network;
- migrate or replicate the local event journal into a transactional HA log/outbox, retain exact replay semantics, and add metrics, tracing, encrypted backups, and restoration drills;
- perform contract, sandbox, cryptographic, and economic audits.
