# xLemma Lean package

This minimal package establishes stable annotations and protocol metadata without changing Lean's trusted kernel.

```lean
import XLemma

@[xlemma]
theorem noArbitrage ... := by
  ...

#xlemma_export noArbitrage
#print axioms noArbitrage
```

`#xlemma_export` deliberately expands only to a human-inspectable declaration print in this prototype. The production exporter must use Lean's environment APIs to serialize the elaborated theorem type, proof object, universes, direct dependencies and axiom inventory into deterministic protocol encodings. Source text alone is not a valid `ClaimID` input.

For high-value or hostile submissions, build through a hardened sandbox and use Lean's comparator workflow plus independently implemented checkers. The Rust service boundary is in `crates/xlemma-lean`.
