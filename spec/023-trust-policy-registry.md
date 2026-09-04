# XLIP-023 — Trust-policy registry and axiom profiles

## Scope

The XLMP trust-policy registry declares which formal environments are eligible
for certification. It does not establish mathematical truth and cannot replace
exact checker execution. A certificate binds both the selected policy and the
underlying observations.

## Native objects

An `AxiomProfile` is content-derived and declares:

- the exact permitted and explicitly forbidden axiom names;
- whether unlisted axioms are allowed;
- whether `sorry`/`admit`, unsafe declarations, or compiler-trusted
  `native_decide` execution are allowed; and
- an optional superseded profile.

XLMP/1 certification profiles MUST set all four permissive flags to `false`.
The fields remain explicit so omission, ambiguous defaults, and policy
substitution cannot hide a weaker trust path.

A certification-eligible `TrustPolicy` binds an axiom profile, permitted
checker families, the minimum number of independent checker families,
permitted canonical encodings, and mandatory exact-challenge, pinned-toolchain,
and dependency-lock requirements.

A `TrustPolicyRegistrySnapshot` contains strictly ID-sorted immutable profiles
and policies. Its `registry_root` is:

```text
blake3("trust-policy-registry-v1" || RFC8785(registry content without root))
```

Every `PolicyID` is derived from policy semantics. The identifier field itself
is excluded. Mutation therefore creates a different ID and registry root.

## Evaluation

Evaluation MUST fail closed unless all of the following hold:

1. the theory references a policy present in the selected registry snapshot;
2. the policy references a profile in the same snapshot;
3. the formal canonical encoding is permitted;
4. the exact challenge, pinned toolchain, and dependency lock are verified;
5. the required number of independently implemented checker families is met;
6. the proof manifest and checker evidence report the identical axiom set;
7. every observed axiom is allowed by both the theory and the axiom profile;
8. no explicitly forbidden axiom or prohibited trust path is observed.

An absent profile, unknown policy, malformed root, noncanonical ordering,
unlisted axiom, evidence mismatch, or insufficient checker diversity is a
rejection. It is never resolved by node count or token vote.

## Publication and supersession

Registries are append-only snapshots. A new policy may identify the policy it
supersedes, but clients retain both. Changing a policy in place is forbidden.
Deployment governance MUST authenticate registry roots through its configured
key-resolution and constitutional process before using them for committee or
certificate eligibility. Content integrity alone does not prove that a
registry publisher is authorized.

The reference implementation supplies content-derived objects, canonical
snapshot validation, fail-closed evaluation, JSON Schemas, a CLI verifier, and
an example vector. Production still requires audited registry publication,
key resolution, independent policy review, and a hostile Lean corpus.
