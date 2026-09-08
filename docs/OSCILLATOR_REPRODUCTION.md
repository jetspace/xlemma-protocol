# Oscillator reproduction and independent review handoff

Status: author-operated discrete-model rehearsal, 2026-09-08. No independent
participants or auditors are engaged yet. This is not a discovery, a first
formalization claim, empirical confirmation, or evidence of research acceleration.
Source and accompanying documentation are available under the repository's
Apache-2.0 license.

## Exact statements

The nondimensional model has unit mass, rational displacement `x`, velocity `v`,
step `h`, stiffness `k` and damping `d`. A midpoint step to `(y,w)` satisfies

```
2(y-x) = h(v+w)
2(w-v) = -h[k(x+y)+d(v+w)]
E2(x,v) = k*x*x + v*v
```

[Oscillator.lean](../lean/XLemma/Physics/Oscillator.lean) proves the exact
conditional identity `2(E2(y,w)-E2(x,v)) = -h*d*(v+w)^2` for rational inputs.
[Reuse.lean](../lean/XLemma/Physics/Reuse.lean) imports that result and derives
conservation for zero damping. The general identity permits negative parameters;
the numerical dissipation checks restrict `h,k,d` to nonnegative values.

The same base module proves that forward Euler at unit stiffness multiplies
`E2` by `1+h*h`. The negative fixture attempts the false conservation equality
at `(x,v,h)=(1,0,1)` and must fail with the expected Lean proof error.
This distinguishes a useful negative result from a missing compiler or import.

The Python reference solves the midpoint equations with exact fractions.
It checks 243 initial parameter combinations over ten steps, the balance,
dissipation and zero-damping conservation, plus reversibility and the Euler
counterexample. These checks exercise an explicit solver independently of the
Lean tactic implementation, but both artifacts were produced by the same team.

## Reproduce

Use the pinned Rust and Lean toolchains in a fresh checkout. Follow
[local development](LOCAL_DEVELOPMENT.md) for prerequisites, then run:

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
make lean-test
cargo test --locked -p xlemma-api event_store::tests
python3 scripts/simulate_discovery.py --check
```

`make lean-test` builds the connected modules, rejects the false assertion,
and runs `leanchecker --fresh XLemma.Physics.Reuse` in addition to the existing
export and checker tests. This checker invocation is author-operated and shares
the Lean distribution; it does not establish independent operator or checker-family
qualification. CI also retains its configured axiom audit and nanoda checks;
a local run does not establish their remote outcome.

The three new theorems report the axiom set `propext`, `Classical.choice`,
`Quot.sound`, matching the existing CI allowlist. They do not claim an empty
axiom inventory.

The formal artifact uses only the pinned Lean distribution's `Std` rational
arithmetic, with no Mathlib or Physlib dependency. A broader upstream coverage
and novelty review remains open. These identities do not prove convergence,
an error bound, a continuous real-valued oscillator solution, or correspondence
with measurements. The new physics modules are not yet packaged as a qualified
discovery submission, policy-bound ClaimID bundle or payable certificate.

## Outside review and experiment still required

1. An independent physics/formal-methods reviewer checks the assumptions, upstream
   coverage, exact statements and proposed extension before the pilot is frozen.
2. Independent checker operators reproduce the frozen revision on their own
   environments, record toolchain/dependency/axiom inventories and failures,
   and qualify under the published policy. A disagreement quarantines the object.
3. Independent downstream researchers receive preregistered held-out tasks in
   the three conditions in the [compounding pilot](RESEARCH_COMPOUNDING_PILOT.md).
   Record creation and maintenance cost as well as downstream effort; disclose
   that the checked-in reuse example is exposed and cannot serve as a held-out task.
4. Outside participants attempt retrieval, reproduction, correction, appeal and
   export within prospectively specified cost and time limits. Record accessibility
   failures and assistance without converting a scripted journey into a human result.
5. An external security reviewer assesses payment, signing, journal rollback,
   RPC/finality, verifier independence and appeal boundaries before live funds.
   Review the retained Slither warnings, not just the passing test counts.

The recovery test covers termination after a durable write, replay, duplicate
retry rejection and restoring a closed journal copy. It does not simulate a
physical power failure, prove storage hardware durability, or detect rollback to
an older internally consistent backup without an external trusted checkpoint.
Production restore drills must reconcile that checkpoint and all settlement
receipts before resuming writes or payments.
