# Lean and LaTeX integration guide

## Lean workflow

1. Pin `lean-toolchain`, package lockfiles, imported libraries, and trust policy.
2. Write the exact theorem statement in a trusted challenge file.
3. Mark exportable declarations with `@[xlemma]`.
4. Build the candidate inside a no-network sandbox.
5. Export the elaborated type and proof object.
6. Enumerate axioms and reject disallowed trust paths.
7. Replay using official Lean checking and at least one independent checker family.
8. Bind all outputs into signed receipts.

## Identity warning

Do not hash source text as a claim identity. Notation, whitespace, implicit arguments, binder names, and imported syntax can differ while elaborating to the same expression. The production exporter must canonicalize the elaborated expression under `TheoryID`.

## LaTeX workflow

Use `latex/xlemma.sty` alongside or in a leanblueprint-style document:

```latex
\lean{Namespace.theoremName}
\leanok
\uses{dependency-labels}
\xtheoryid{xlt:...}
\xclaimid{xlc:...}
\xproofid{xlp:...}
\xartifactid{xla:...}
\xverification{xlemma-gold-v1}
\xverificationreceipt{xlr:...}
\xrightsmanifest{...}
\xcontributionmanifest{...}
\xrevenue{...}
```

The PDF should display formal status, exact declaration name, IDs, toolchain/theory, axiom policy, verification grade, receipt, license, and rights caveat.

## Presentation rule

A human-readable statement is not the formal source of truth. A presentation update gets a new `PresentationID`; a formal statement update gets a new `ClaimID`. The protocol should automatically render the raw formal type beside the human explanation for high-assurance work.
