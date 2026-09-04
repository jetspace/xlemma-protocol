# XLIP-001 — Identifiers and canonicalization

## TheoryID

TheoryID MUST bind protocol version, Lean toolchain, dependency root, trust/axiom policy, checker policy, and canonical encoding version.

## ClaimID

ClaimID MUST be derived from the canonical elaborated Lean expression under TheoryID. Source strings, theorem names, filenames, comments, and LaTeX MUST NOT serve as the sole identity input.

## ProofID

ProofID MUST bind ClaimID and a canonical checker-consumable proof object.

## ArtifactID

ArtifactID MUST bind a sorted file manifest, byte hashes, toolchain, dependency lock, and relevant build metadata. Symlinks and path traversal MUST be rejected by bundle builders.

## ReceiptID

ReceiptID MUST be domain-separated by receipt kind and bind all consensus-critical fields.

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

JSON manifests SHOULD use RFC 8785 or an independently tested equivalent. Lean expressions require a dedicated versioned canonical encoding that resolves names, universes, binders, implicit arguments, and type information.

## Equivalence

Similarity systems MAY propose clusters. A protocol `FORMALLY_EQUIVALENT` edge MUST point to an explicit Lean proof under a declared equivalence relation.
