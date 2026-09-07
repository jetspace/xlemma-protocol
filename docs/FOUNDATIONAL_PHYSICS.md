# Foundational physics research program

xLemma should help researchers uncover the organizing principles beneath physical
phenomena: start from explicit assumptions, identify the relevant symmetries and
variational structure, derive consequences, and determine where they survive
comparison with nature. This is our Landau-inspired research framing.

The ambition is an evolving web of independently checked relationships between
results and theories. Researchers enter wherever a question leads them; teams can
work across mechanics, fields, mathematics and experiments simultaneously. Pooled
funding can reward making that web more reliable, explanatory and reusable before
commercial applications are known.

The [project direction](PROJECT_DIRECTION.md) makes the near-term objective
concrete: test whether a small connected corpus helps independent researchers
reproduce and extend reliable work more efficiently. This tests one mechanism
toward the broader ambition of expanding humanity's capacity for discovery;
individual foundational contributions need no immediate application.

## Starting references

Landau and Lifshitz provide seed questions and organizing perspectives:

- [Volume 1: Mechanics, third edition](https://shop.elsevier.com/books/mechanics/landau/978-0-08-050347-9)
  covers equations of motion, conservation laws, collisions, oscillations, rigid
  bodies and canonical equations.
- [Volume 2: The Classical Theory of Fields, fourth edition](https://shop.elsevier.com/books/the-classical-theory-of-fields/landau/978-0-08-050349-3)
  covers relativity, electromagnetic fields and radiation, gravitation and
  relativistic cosmology.

These are publisher access listings. This program is based on their published
scope; it does not claim a completed chapter-by-chapter review or formalization.
Contributions should cite the edition and section they use and supply original,
appropriately licensed artifacts. The repository does not redistribute the books.

The books offer routes through the research web. A textbook citation cannot replace checker
reproduction or experimental evidence. Other approaches, foundational mathematics
and questions outside these volumes remain eligible under published round rules.

## Connections that carry scientific meaning

A proposed unification should identify precisely what it connects and under which
conditions. Shared notation or analogy can motivate a question; verification must
establish the claimed relationship.

| Proposed relationship | What a contribution must make explicit |
|---|---|
| Derivation | Assumptions and a reproducible argument establishing the conclusion. |
| Symmetry and conservation | The system, transformation, action or equations, and hypotheses under which the claimed invariant follows. |
| Equivalence | Maps between formulations, their domains, and which solutions or observables are preserved; whether the relation holds in both directions. |
| Limiting case | The parameter, regime, approximation and sense of convergence; error bounds where claimed. |
| Shared mathematical structure | The structure and maps actually established, with the physical interpretation of each model kept explicit. |
| Empirical disagreement | A specified observable and conditions under which predictions differ, with uncertainty and a feasible independent test. |

These are proposed research acceptance profiles, not newly implemented XLMP edge
types. They require explicit artifact/schema mapping, reference vectors and
independent review before protocol integration. Existing evidence and economic
edges retain their current meanings.

Unification remains a research question. A verified bridge between two models can
be useful without establishing a universal theory. Counterexamples and failed
equivalences are valuable when they locate the limits of a proposed connection.

## Research objects and evidence

Each proposed result should identify its assumptions, mathematical statement,
units and conventions, dependencies, derivation or computation, approximation
regime, and evidence limitations. Predictions should specify observables and
conditions under which they could be tested. Experimental records additionally
need methods, provenance, calibration, uncertainty and replication evidence.

| Contribution | Evidence required for its stated claim |
|---|---|
| Formal derivation | Exact reproduction of the proof under explicit assumptions and a pinned checker policy. |
| Reproduced calculation | Independent execution with pinned code, inputs, environment and declared numerical tolerances. |
| Proposed extension | Explicit differences from the baseline theory, consistency checks and distinguishable predictions where available. Derivation alone does not establish physical validity. |
| Measurement or replication | Qualified execution of a documented method, with uncertainty and independently assessable observations. |

Keep these records linked and distinct. A checked theorem establishes a
consequence of assumptions; it does not establish that nature satisfies them.
A replicated observation supports a claim within its tested conditions; it does
not prove a theory universally. Negative findings and counterexamples can narrow
or invalidate a claim without erasing the historical evidence.

## Seed questions and parallel research paths

- **Variational structure:** for a specified action and admissible variations,
  which equations follow, and which regularity and boundary assumptions are needed?
- **Symmetry and invariants:** under stated hypotheses, which transformations
  preserve the dynamics and what conserved quantities can be derived?
- **Alternative descriptions:** when can two formulations represent the same
  dynamics, and where do constraints or singularities obstruct that equivalence?
- **Relations between regimes:** under what controlled limit does one description
  recover another, and which observable differences remain outside that regime?
- **Predictions and counterexamples:** what would distinguish competing models,
  invalidate a claimed relation, or reveal a missing assumption?

An initial pilot should select a small, connected set of these questions according
to available mathematics, checker support and independent review capacity. It can
include mechanics and field-theory contributions together. Formal-library gaps,
conventions and acceptance criteria must be published before promising coverage.

Publish stable artifacts, reproduce them independently, and exercise corrections
and appeals. A funded round can follow once deployment gates are satisfied.
Empirical paths additionally require qualified laboratories, a measurement target,
uncertainty budget and outcome-neutral terms before confirmatory work begins.
Exploratory research remains welcome with its status visible.

The initial completion target is a small, connected set of independently
reproduced results and justified relationships with functioning funding and
appeals. Whole-volume coverage and a unified physical theory are not promised
deliverables.

## Reward policy

Existing results may qualify for first-formalization or proof-improvement rewards
without being labeled discoveries. Independently useful intermediate results,
software, replication and informative negative results can qualify in their
published categories. Assess a contribution's additional value before dividing
its award among collaborators.

A demonstrated connection between existing results may qualify under an existing
category when it adds a supported contribution. Any new eligibility rules must be
published prospectively. A shorter formulation, fewer stated axioms or more graph
links do not establish additional value by themselves: hidden assumptions,
duplicate descriptions and lost predictive content must be examined. No automatic
bonus is introduced for claiming unification or linking already rewarded work.

Page count, equation count, proof splitting, citations, raw tokens and elapsed
compute do not create reward entitlements. Reference difficulty and cost remain
contestable inputs to a domain-specific assessment. Separate fixed service fees
pay completed verification and review regardless of a positive verdict, while
total spending remains bounded by settled funding and reserved commitments.

Institutions can fund this program, compute or laboratory access with disclosed
restrictions and conflicts. Funding does not grant authority to certify results
or decide appeals involving the funder's own research.

## Status and next deliverables

The [discovery service](DISCOVERY_SERVICE.md) supplies reference funding,
assessment, appeal and settlement infrastructure. This document adds a research
direction; it adds no Lean proofs, relation schemas, empirical qualification,
reward-logic changes or live funding.

- [ ] Publish an initial connected statement inventory with source references,
  assumptions, dependency gaps and bounded acceptance criteria for each relation.
- [ ] Specify and review mappings from proposed relation profiles to protocol artifacts, with reference vectors before implementation claims.
- [ ] Produce original artifacts across selected research paths and obtain independent reproduction.
- [ ] Qualify reviewers and calibrate reference tasks before opening a funded round.
- [ ] Exercise correction, rejection and appeal cases on the selected corpus.
- [ ] Qualify further mechanics, field-theory and empirical paths as their dependencies and review capacity become available.

The [roadmap](../ROADMAP.md) retains the independent audit, operator qualification,
funded deployment and recovery gates required before public activation.
