# Lean pseudo self-test report

As-of date: 2026-09-04.

## Result

**PASS — author-operated pseudo/self-test only.**

The repository-pinned toolchain was installed locally through `elan` and the
checked-in Lean package was built and replayed successfully:

- `elan-init 4.2.4` installed through Homebrew;
- `leanprover/lean4:v4.33.1` installed and selected by `lean/lean-toolchain`;
- Lean reported commit `819816b2e0a3bf405af45ae5c7af2491d8f5bee6` for
  `arm64-apple-darwin24.6.0`;
- Lake `5.0.0-src+819816b` built all seven package jobs;
- `lake env lean -DwarningAsError=true XLemma/Example.lean` emitted one
  machine-readable environment record for `add_zero_verified`, exactly matched
  the checked-in canonical type/proof/dependency/axiom vector, and reported no
  axiom dependencies;
- negative fixtures confirmed that unsafe and valueless declarations fail the
  default export boundary;
- the Rust protocol parser validated the record against the resolved theory
  and reproduced the checked-in domain-separated `ClaimID` and `ProofID`;
- verifier-adapter tests rejected missing, multiple, malformed,
  identity-mismatched, environment-mismatched, and prohibited-axiom exports;
- `lake env leanchecker --fresh XLemma.Example` completed with exit status zero.

The repeatable local command is:

```bash
cd lean
./self-test.sh
```

## What this establishes

This is evidence that the pinned Lean release can elaborate the checked-in
example, that warnings fail the example run, that `#xlemma_export` serializes
the checked environment to the exact `expected-add-zero.json` vector, that its
type/proof/name strings are parseable compact JSON, that `#print axioms`
reports an empty axiom inventory for that theorem, and that the bundled fresh
Lean checker can replay the produced module on this machine. Build-time guards
also confirm binder-name alpha-invariance and fail-closed free-variable and
metavariable cases. Separate negative files confirm command-level rejection of
unsafe and valueless declarations.

## Explicit limitations

This result is deliberately labeled a pseudo/self-test because the repository
author/operator selected the inputs, installed the toolchain, ran the checks,
and recorded the result. It is not independent reproduction and does not close
the clean-room or independent-checker roadmap gates.

In particular:

- the reference exporter and one deterministic vector are now implemented,
  but no clean-room implementation has reproduced the encoding and the current
  vector does not constitute a hostile proof corpus;
- direct Lean constants are enumerated, but mapping them to xLemma `ClaimID`
  dependency edges and binding the dependency lock/artifact roots still belongs
  to the external evidence packer;
- the replay used the checker bundled with the same Lean distribution, not an
  independently implemented checker family such as nanoda;
- no hostile proof corpus, comparator challenge workflow, sandbox, independent
  operator, signed observation, PoIR committee, or production certificate was
  exercised;
- a local exit status of zero is not a claim of theorem novelty, informal
  statement alignment, production safety, or independent mathematical
  certification.
