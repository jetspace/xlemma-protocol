# XLIP-014 — API and job state protocol

The HTTP API is an XLMP transport adapter. The canonical ingress is `POST /xlmp/v1/messages` using `application/x-xlmp+json;version=1`. REST convenience endpoints MUST produce or consume equivalent XLMP records and MUST NOT redefine protocol state.

Before accepting a message as authenticated, a deployment MUST verify its
signature against the configured XLMP signature profile and sender identity.
The prototype Rust ingress checks canonical content identity and append-only
storage only; it is not a production authentication boundary.

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
