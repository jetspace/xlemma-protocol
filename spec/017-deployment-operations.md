# XLIP-017 — Deployment and operations

XLMP/1 SHOULD use an existing chain through a `FinalityAdapter` and off-chain role-specific node committees. A chain orders or anchors state and settles value; it does not define formal validity, attribution, novelty, or rights. Production deployment requires:

- audited smart contracts and cryptography;
- hardened sandboxing;
- independent Lean checker integration;
- HSM-backed node signing and rotation;
- manipulation-resistant randomness;
- stable-backing reconciliation;
- monitoring, incident response, pause and recovery;
- artifact retention and disaster recovery;
- privacy, sanctions, tax, payments, consumer, securities, commodities, IP, and jurisdictional review;
- deterministic test vectors and cross-implementation conformance tests.

The public x402 development facilitator MUST NOT be assumed to be the production mainnet path without current verification of its intended use.
