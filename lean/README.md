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

`#xlemma_export` reads the elaborated declaration from Lean's environment and
emits one `XLMP_LEAN_EXPORT` JSON record. The record contains the closed
canonical type and proof term, structural universe/name encodings, sorted
direct constants, transitive axiom inventory, declaration kind and exact Lean
version/commit. It rejects unsafe, partial, valueless, free-variable, and
metavariable-bearing exports.

The two identity-bearing strings use `xlemma-lean-expr-v1`, a tagged structural
encoding rather than pretty-printed source. Binder names and elaborator
metadata are removed; de Bruijn indices, binder modes, universes, constants,
literals and projections remain. Decimal strings carry natural values so JSON
number limits cannot change identity. `canonical_elaborated_type` is the input
to `ClaimID` derivation under a `TheoryID`; `canonical_proof_object` is the
input to `ProofID` derivation under that `ClaimID`. The declaration name and
source text are not ClaimID material.

`../examples/lean-export/expected-add-zero.json` is the deterministic reference
vector and `validate-export.py` compares actual output to it. The Rust
`LeanEnvironmentExport` parser rejects unknown fields, noncanonical embedded
JSON, unsafe/partial declarations, duplicate evidence names, and mismatched
theory protocol/encoding/toolchain/axioms before deriving the paired IDs in
`expected-ids.json`. This establishes the repository's reference serialization
boundary, not cross-implementation agreement; the encoding remains unfrozen
until a clean-room implementation reproduces it.

Run `./self-test.sh` for the pinned, author-operated package build,
warning-as-error deterministic export-vector/axiom check, fresh bundled-checker
replay, and focused Rust ingestion/ID tests. The result is a pseudo/self-test,
not an independent checker or clean-room reproduction; see
`../docs/LEAN_SELF_TEST_REPORT.md`.

The self-test also expects `tests/RejectUnsafe.lean` and
`tests/RejectValueless.lean` to fail at the export boundary. These negative
fixtures are tests, not proof artifacts suitable for publication.

For high-value or hostile submissions, build through a hardened sandbox and use Lean's comparator workflow plus independently implemented checkers. The Rust service boundary is in `crates/xlemma-lean`.
