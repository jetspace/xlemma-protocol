# XLIP-015 — Storage and availability

IPFS, content-addressed object stores, archival nodes, and local stores are implementations of the XLMP `StorageAdapter`. Storage transport does not define research identity, validity, provenance, or rights.

Artifact manifests MUST contain sorted safe relative paths, media types, byte hashes, lengths, and encryption flags. The artifact root binds every one of those fields. Bundle builders MUST reject absolute or platform-ambiguous paths, parent traversal, duplicate paths, symlinks, non-regular files, root mismatches, and bounded-resource violations unless a future policy explicitly defines them.

Builders MUST accept an explicit creation timestamp for deterministic
conformance runs. Given identical bytes, metadata, build parameters, and that
timestamp, implementations MUST emit the same manifest and ArtifactID. The
XLMP/1 test vector is published in `examples/deterministic-bundle/`.

Availability receipts MUST bind artifact, storage node, operator cluster, provider, region, custody challenge root, retention horizon, time, and signature.

Policies SHOULD require independent replicas across operators, providers, and regions. Periodic proof-of-custody challenges SHOULD be used for long-lived certificates.

The certificate root MUST point to underlying receipts; it MUST NOT replace them.
