# Deployment reference

The compose file illustrates the target control-plane dependencies—API, PostgreSQL, and authenticated NATS JetStream—but the current Rust API uses in-memory state and does not yet connect to those services. A content-storage service is intentionally not bundled: the formerly referenced MinIO community image is not available for its latest security release. Configure a separately maintained, digest-pinned storage adapter instead.

Formal verification workers should not run inside the API container. Deploy separate isolated worker pools for ASTRA generation, Lean builds, official kernel checks, independent checks, novelty review, and storage. High-assurance jobs require distinct operator clusters and should not share a privileged orchestrator host.

Before production:

- keep the committed image digests current and pin every separately deployed adapter by immutable digest;
- replace local secrets with a secret manager/HSM;
- add mTLS and workload identity;
- enforce egress-denied build sandboxes;
- use a production x402 facilitator or self-facilitation path appropriate to the network;
- add database migrations, durable outbox, event replay, metrics, tracing, and backups;
- perform contract, sandbox, cryptographic, and economic audits.
