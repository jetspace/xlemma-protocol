# XLIP-014 — API and job state protocol

The HTTP API is an XLMP transport adapter. The canonical ingress is `POST /xlmp/v1/messages` using `application/x-xlmp+json;version=1`. REST convenience endpoints MUST produce or consume equivalent XLMP records and MUST NOT redefine protocol state.

Before accepting a message as authenticated, a deployment MUST verify its
signature against the configured XLMP signature profile and sender identity.
The Rust reference ingress requires an API bearer token, verifies the baseline
Ed25519 XLMP profile against an explicit signer allowlist, binds observation
signers one-to-one to NodeIDs and committed committee identities, rejects
unknown or non-canonical typed XLMP fields, and requires a prior signed commit.
Committee reveals must match the exact stored job roster and server-side trust
policy. The reference API persists complete mutations in a versioned,
domain-separated BLAKE3 hash chain, fsyncs each entry before acknowledgement,
and reconstructs messages, commitments, and job state on restart. Replay fails
closed on a changed record, gap, duplicate, missing predecessor, or invalid job
update. The reference journal is intentionally single-writer; production HA
deployments must replicate or migrate it to a transactional log/outbox without
weakening exact replay, and must implement key rotation and backup restoration.

Payment offers MUST be derived from authorized server-side job and quote state.
The reference HTTP server does not expose a payment-offer constructor; the
provider-neutral `xlemma-x402` codec remains available to a payment adapter.

The protocol-level lifecycle is `CLAIM → COMMIT → FORMALIZE → PROVE → REPRODUCE → CERTIFY → CHALLENGE → FINALIZE → PUBLISH → REUSE → REWARD → REVALIDATE`. The implementation-level verification sequence below refines part of that lifecycle:

The canonical verification state sequence is:

```text
DRAFT → CLAIM_COMMITTED → QUOTED → FUNDED → ASSIGNED
→ FORMALIZING → CANDIDATE_READY → BUILDING
→ CHECKERS_COMMITTED → CHECKERS_REVEALED
→ PASSED | FAILED | DIVERGENT
→ CHALLENGED | FINALIZED | REJECTED | QUARANTINED
→ PUBLISHED → REVALIDATED | SUPERSEDED | QUARANTINED
```

A changed formal claim, proof object, artifact root, theory, or trust policy MUST create a new object/job rather than mutate a running job.

API responses SHOULD include stable IDs, state, policy, timestamps, and receipt links. Error responses MUST distinguish mathematical failure, checker error, insufficient quorum, payment failure, and divergence.
