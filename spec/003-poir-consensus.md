# XLIP-003 — Proof of Independent Reproduction

## Observation

A PoIR observation MUST bind job, NodeID, VerifiedUserID, OperatorID,
OperatorClusterID, UserCredentialID, OperatorCredentialID, NodeCredentialID,
credential-chain root, checker family, artifact root, environment root,
dependency root, axiom root, observation root, verdict, timestamps, and
signature. The chain MUST have passed XLIP-020 admission before assignment.

## Commit-reveal

Nodes SHOULD commit `H(JobID || Verdict || ObservationRoot || Salt)` before revealing. A reveal MUST match the commitment.

## Generalized quorum

A policy MUST express required checker families and minimum independent
VerifiedUserIDs, OperatorIDs, and operator clusters. High-assurance policies
SHOULD additionally require infrastructure-provider and region diversity.

## Exact comparison

Where a policy requires exact root equality, any root mismatch MUST produce `DIVERGENT`.

## Verdict aggregation

- all required evidence present and all required families pass: `CERTIFIED`;
- all required families fail against the same roots: `REJECTED`;
- pass/fail mix, checker error, or root mismatch: `DIVERGENT`;
- missing role/diversity/reveal: insufficient evidence.

A majority MUST NOT override a required-family disagreement.

## Two-stage finality

PoIR creates an epistemic certificate. A base chain or BFT state machine later orders certificate and economic transitions. The economic layer MUST NOT alter underlying receipts.
