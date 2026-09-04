# XLIP-014 — API and job state protocol

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
