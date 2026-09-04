# XLIP-015 — Storage and availability

IPFS, content-addressed object stores, archival nodes, and local stores are implementations of the XLMP `StorageAdapter`. Storage transport does not define research identity, validity, provenance, or rights.

Artifact manifests MUST contain sorted safe relative paths, media types, byte hashes, lengths, and encryption flags. Bundle builders MUST reject absolute paths, parent traversal, symlinks, and non-regular files unless a future policy explicitly defines them.

Availability receipts MUST bind artifact, storage node, operator cluster, provider, region, custody challenge root, retention horizon, time, and signature.

Policies SHOULD require independent replicas across operators, providers, and regions. Periodic proof-of-custody challenges SHOULD be used for long-lived certificates.

The certificate root MUST point to underlying receipts; it MUST NOT replace them.
