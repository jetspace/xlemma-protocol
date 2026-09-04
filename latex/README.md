# LaTeX integration

`xlemma.sty` embeds immutable formal, provenance, rights, and revenue identifiers beside human-readable mathematics. It is compatible in spirit with leanblueprint's `\lean`, `\leanok`, and `\uses` workflow.

Changing only the prose or typesetting creates a new `PresentationID`, not necessarily a new `ClaimID`. Changing the elaborated Lean statement always creates a new `ClaimID`.

The package intentionally prints a warning that the formal Lean declaration remains authoritative. A LaTeX-to-declaration link proves only which declaration was referenced; it does not prove that the prose accurately characterizes the declaration.
