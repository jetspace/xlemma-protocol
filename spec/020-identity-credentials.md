# XLIP-020 — Participant, operator, and node credentials

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Constitutional rule

No node may contribute to xLemma consensus without a valid, non-revoked
`XLMP_OperatorCredential` ultimately controlled by a verified xLemma
participant. Multiple nodes controlled by the same participant constitute one
operator-independence domain.

Identity is therefore an eligibility and independence control, not a measure
of mathematical truth. Credentials, reputation, bonds, institutional status,
and legal identity MUST NOT override exact checker evidence or weight a formal
vote.

## 2. Identity hierarchy

XLMP distinguishes the following stable, typed identifiers:

```text
VerifiedUserID ── controls ──> OperatorID ── delegates ──> NodeID(s)
       │
       └── may link ──> ResearcherID
```

- `VerifiedUserID` is a pseudonymous public handle for a person or accountable
  organization whose uniqueness/control evidence was checked by an issuer.
- `OperatorID` identifies an operating principal and is the primary home of
  reputation and accountability.
- `NodeID` identifies one machine, service agent, or key-bound node instance.
- `ResearcherID` identifies a research persona and is a sibling role. It need
  not reveal or equal `VerifiedUserID`.
- `OperatorClusterID` is a conservative common-control domain. Undisclosed or
  suspected common control MAY merge multiple OperatorIDs for independence
  purposes; it MUST NOT split one verified participant into extra votes.

One verified participant MAY operate multiple machines and MAY rotate node
keys. That creates multiple NodeIDs but never additional committee
independence.

## 3. Credential objects

### 3.1 XLMP_UserCredential

The user credential binds `VerifiedUserID`, optional `ResearcherID`, a public
pseudonym, credential tier, issuer, uniqueness commitment, qualifications,
disclosure policy, validity interval, evidence root, and issuer signature.
Raw legal names, passport numbers, addresses, biometric templates, and the
issuer's private evidence MUST NOT appear in the public protocol object.

### 3.2 XLMP_OperatorCredential

The operator credential binds an OperatorID to its VerifiedUserID and exact
UserCredentialID. It also binds OperatorClusterID, authorized roles,
qualifications, a coarse jurisdiction class, validity interval, evidence root,
holder delegation signature, and issuer signature.

The holder delegation proves that the verified participant authorized the
operator. An issuer attestation alone MUST NOT silently transfer operational
control.

### 3.3 XLMP_NodeCredential

The node credential binds NodeID, OperatorID, OperatorCredentialID,
OperatorClusterID, node public key, authorized roles, optional hardware
attestation root, validity interval, evidence root, and operator delegation
signature. A NodeCredential MUST NOT outlive or expand the roles of its parent
OperatorCredential.

### 3.4 CredentialStatusProof and revocation

A committee candidate carries a fresh, issuer-authenticated non-revocation
proof binding all three credential IDs to an exact append-only revocation
registry root. Status proofs are short-lived. Policies MUST publish their
maximum proof age and MUST fail closed on absent, expired, invalid, or stale
proofs.

`XLMP_CREDENTIAL_REVOCATION` is append-only and binds a content-derived
RevocationID, exact credential reference, effective time, reason code,
evidence root, issuer, and issuer signature. Revoking a user credential makes
all descendant operators and nodes ineligible. Revoking an operator makes its
descendant nodes ineligible. Historical observations remain readable and are
not silently deleted; policy decides whether affected certificates require
quarantine or revalidation.

## 4. Credential tiers

| Tier | Meaning | Consensus authority |
|---|---|---|
| V0 observer | read, index, and locally verify public data | none |
| V1 verified participant | attributable participation and low-risk market activity | none |
| V2 verified operator | accountable operator with node delegation | eligible under role policy |
| V3 institutional operator | verified accountable organization | same mathematical authority as V2 |
| V4 specialized authority | additional role-specific qualification | only the role named by policy |

V2 is the minimum for PoIR, challenge, finalization, and other consensus
committee roles. Higher tiers do not create extra votes or make observations
more mathematically valid.

## 5. Admission validation

Before admitting a node to an eligible set, an implementation MUST:

1. validate every typed and content-derived credential identifier;
2. verify issuer signatures under the active issuer policy;
3. verify participant-to-operator and operator-to-node delegation signatures;
4. verify exact parent links and nested validity intervals;
5. verify role authorization and required qualifications;
6. verify the status proof against the current revocation registry root;
7. reject any effective user, operator, or node revocation;
8. bind the validated chain and status proof into the eligible-set commitment.

The reference Rust registry is append-only and requires a
`CredentialProofVerifier` adapter. Its structural checks do not substitute for
the production signature/key-resolution adapter.

## 6. Committee independence

A committee member record publishes VerifiedUserID, OperatorID,
OperatorClusterID, all three credential IDs, credential tier, and credential
chain root. A single committee MUST contain at most one member from each:

- VerifiedUserID;
- OperatorID; and
- OperatorClusterID.

The strictest collision controls. If two apparently distinct users are later
shown to share control, an append-only cluster/conflict record may make them
jointly ineligible and may trigger revalidation; prior records are preserved.

The Gold profile requires at least three NodeIDs, three OperatorIDs, three
verified participants or organizations, two checker families, two
infrastructure providers, and two regions. Provider and region diversity do
not substitute for operator independence.

## 7. Conflicts

An operator MUST disclose material control relationships and job-specific
conflicts. The following pairs are incompatible on the same research job:

- researcher or proof producer with final formal verifier;
- build producer with official or independent checker;
- payment facilitator with certificate finalizer;
- active challenger with certificate finalizer; and
- multiple nodes under one verified participant, OperatorID, or
  OperatorClusterID occupying multiple committee slots.

Concealment, forged delegation, fabricated uniqueness evidence, or use of a
revoked key may be slashable under a published policy. Honest `FAIL` results,
checker divergence, and unpopular novelty judgments are not by themselves
misconduct.

## 8. Operator-primary reputation

Reputation snapshots bind both OperatorID and NodeID. The OperatorID is the
primary accountable record; NodeIDs are machine/key subrecords that preserve
latency, availability, and hardware-specific evidence across rotations.

XLMP/1 keeps separate evidence-backed dimensions for formal accuracy,
availability, latency, novelty calibration, challenge quality, independence,
storage quality, and integrity. It defines no universal scalar score. A new
operator or node MUST NOT inherit reputation merely through asset, credential,
or token transfer.

## 9. Privacy and issuer boundaries

Public protocol data is pseudonymous. The issuer retains private identity and
uniqueness evidence under its disclosed legal, retention, access, and breach
policies. Selective-disclosure or zero-knowledge credentials MAY replace
ordinary attestations when they preserve stable uniqueness, accountability,
revocation, and delegation checks.

No global issuer is built into XLMP. Networks MUST publish accepted issuer and
key-resolution policies, avoid one-provider capture, and preserve a path to
credential portability. Issuer approval qualifies an accountable participant;
it does not certify a proof or decide a research dispute.

## 10. Required wire messages

```text
XLMP_USER_CREDENTIAL
XLMP_OPERATOR_CREDENTIAL
XLMP_NODE_CREDENTIAL
XLMP_CREDENTIAL_REVOCATION
```

These messages use the standard XLMP/1 envelope. Credential identifiers omit
signature fields from their canonical content identity so a signature-format
migration does not silently change the asserted subject; signed envelopes and
credential signatures still authenticate the exact content-derived ID.

## 11. Conformance

A conforming implementation MUST test malformed links, stale status proofs,
effective revocations, role expansion, tier insufficiency, duplicate users,
duplicate operators, duplicate clusters, pseudonymous public objects, and
append-only behavior. It MUST demonstrate that running more machines under one
participant never produces additional PoIR independence.
