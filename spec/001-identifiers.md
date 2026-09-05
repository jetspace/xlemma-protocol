# XLIP-001 — Identifiers and canonicalization

## TheoryID

TheoryID MUST bind protocol version, Lean toolchain, dependency root, trust/axiom policy, checker policy, and canonical encoding version.

## ClaimID

ClaimID MUST be derived from the canonical elaborated Lean expression under TheoryID. Source strings, theorem names, filenames, comments, and LaTeX MUST NOT serve as the sole identity input.

## ProofID

ProofID MUST bind ClaimID and a canonical checker-consumable proof object.

## ArtifactID

ArtifactID MUST bind a sorted file manifest, byte hashes, toolchain, dependency lock, and relevant build metadata. Symlinks and path traversal MUST be rejected by bundle builders.

`created_at` and source-control labels are preserved as provenance but excluded
from ArtifactID. The canonical bundle builder accepts an explicit timestamp so
the complete manifest can also be reproduced byte-for-byte. The published
vector is in `examples/deterministic-bundle/`.

## PolicyID and registry roots

Trust policies and axiom profiles use the typed `PolicyID` domain and exclude
only their identifier field from identity. Trust-registry roots use the
`trust-policy-registry-v1` domain over a strictly ID-sorted snapshot. See
XLIP-023.

## ReceiptID

ReceiptID MUST be domain-separated by receipt kind and bind all consensus-critical fields.

## Offline discovery pilot identifiers

The local XLIP-024 simulation uses `DiscoveryRoundID` (`xlround:blake3:`,
`discovery-round-v1`) for the entire immutable round policy and
`ContributionGroupID` (`xlgroup:blake3:`, `contribution-group-v1`) for economic
grouping assertions. Group identity is not a proof of formal equivalence.
Simulation event ReceiptIDs bind an explicit simulation domain, RoundID, the
previous receipt and the entire event. The initial receipt also commits
declared funding and prior settlement/contribution histories. They are local
audit records, not authenticated evidence or payment receipts. Existing formal
ClaimID/ProofID derivation is unchanged.

## Participant, operator, node, and credential identity

`VerifiedUserID`, `OperatorID`, and `NodeID` are separate typed domains.
`UserCredentialID`, `OperatorCredentialID`, `NodeCredentialID`, and
`CredentialRevocationID` are content-derived from their exact public assertion
excluding identifier and signature fields. A different signature encoding does
not create a new asserted subject, while any change to subject, delegation,
role, qualification, time window, evidence root, or revocation content does.

`ResearcherID` is a sibling research persona and MAY be linked by a
UserCredential; it is not silently equated with VerifiedUserID. See XLIP-020.

## Canonicalization

JSON manifests used for identifiers or signatures MUST use RFC 8785 JCS. Integer values outside the exact IEEE-754 range MUST be encoded as decimal strings or rejected. Lean expressions require a dedicated versioned canonical encoding that resolves names, universes, binders, implicit arguments, and type information.

### `xlemma-lean-expr-v1`

The reference Lean environment exporter serializes an elaborated expression as
compact UTF-8 JSON with no insignificant whitespace. Every node is a tagged
array. Names retain their exact recursive `anonymous`, `str`, or numeric
structure; universes retain `zero`, `succ`, `max`, `imax`, and named
parameters; expressions retain bound-variable indices, sorts, constants and
universe instantiations, applications, binder mode, lambda/forall/let shape,
literals, and projections. Natural values are base-10 strings without leading
zeroes.

Binder names and expression metadata MUST be omitted because they do not affect
kernel identity. Bound occurrences remain de Bruijn indices. A free variable,
expression metavariable, or universe metavariable MUST fail export. Unsafe,
partial, or valueless declarations MUST NOT cross the default export boundary.
Direct constant names and transitive axioms are emitted separately as evidence.

`canonical_elaborated_type` is exactly the compact structural string hashed
under `TheoryID` for `ClaimID`. `canonical_proof_object` is exactly the compact
structural proof string hashed under the resulting `ClaimID` for `ProofID`.
The outer export record, declaration name, source spelling, dependency summary,
and axiom summary are evidence and MUST NOT be substituted for either identity
input. See `lean/XLemma/Export.lean`, the export schema, and
`examples/lean-export/expected-add-zero.json`. The paired
`expected-ids.json` vector fixes the resulting Rust reference `ClaimID` and
`ProofID` under a declared `TheoryID`.

## Equivalence

Similarity systems MAY propose clusters. A protocol `FORMALLY_EQUIVALENT` edge MUST point to an explicit Lean proof under a declared equivalence relation.
