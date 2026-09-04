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

The reference server also exposes read-only projections for researchers,
theories, claims, contribution and rights manifests, proofs, PoIR certificates,
compute receipts, research credits, latest vault snapshots, lemma capsules,
publications, licenses, challenges, quarantine records, revenue events,
dependency dividends, and availability receipts. Canonical ingress rejects duplicate native objects and enforces
cross-object prerequisites for claim-bound contribution/rights manifests,
challenges/finalization, vault ownership, capsule assembly, publication,
revenue, dividends, and artifact availability. These projections are indexes
over accepted XLMP messages; they are not a second source of protocol truth.

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

## Reference ingress and recovery requirements

The API and journal replay use the same `ProtocolProjection` validator. A claim
requires its accepted theory; proofs require their claim; certificates require
their proof; capsules bind the exact claim, theory, artifact, rights and
contribution roots. Publication requires matching licenses and finalization,
and cannot precede finalization or bypass an unresolved challenge.

Formal certificate ingress MUST resolve the immutable verification job and
check the complete accepted receipt set against its committee roster and formal
policy. Receipts accepted through either observation endpoint participate.
Omitting dissent, substituting roots or a policy, inventing operator clusters,
or shortening the policy challenge period MUST fail. Caller timestamps MUST
NOT advance live finalization or publication into the future.

On Unix, the local journal uses a nonblocking exclusive advisory file lock,
refuses symlink/non-regular journal files, and creates files with mode 0600.
An incomplete final line fails recovery. Any write or fsync failure disables
further appends for that writer; recovery is an operator action. This does not
provide cross-host consensus, rollback protection, or a distributed outbox.
Historical records failing the strengthened checks MUST NOT be silently
rewritten: preserve the original journal and rebuild a reviewed valid history.
