# Foundational physics research program

xLemma should help researchers build a shared, independently checked account of
physical theories: what they assume, what follows, where approximations apply,
and which predictions survive measurement. Pooled funding can reward building,
testing, correcting and extending that account before commercial applications
are known. Researchers retain freedom to choose their questions.

## Starting references

Landau and Lifshitz provide a proposed initial scope:

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

The books organize a starting corpus. A textbook citation cannot replace checker
reproduction or experimental evidence. Other approaches, foundational mathematics
and questions outside these volumes remain eligible under published round rules.

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

## Proposed sequence

1. **Establish a small mechanics corpus.** Choose a bounded set of original
   statements around variational mechanics, conservation laws and oscillations.
   Start with explicitly restricted mathematical settings; make regularity,
   boundary conditions and approximation assumptions visible. Inventory missing
   formal-library dependencies before committing to coverage or delivery dates.
2. **Qualify reproduction.** Publish stable artifacts and reference calculations;
   obtain independent checker reproduction, adversarial review and working
   evidence/eligibility appeals. Record failures as well as successful cases.
3. **Run a bounded funded round.** Reward useful first formalizations, proof
   improvements, reusable tools and independent checking under prospective
   category budgets. Use the existing service only after its deployment gates
   are satisfied. Measure both duplicate-payment leakage and legitimate exclusion.
4. **Extend into classical fields.** Begin with selected special-relativistic and
   electromagnetic results and explicit conventions. Expand toward gravitation
   as mathematical dependencies and qualified review capacity permit.
5. **Connect predictions to tests.** Qualify one feasible empirical profile with
   independent laboratories. Publish the measurement target, method, uncertainty
   budget and outcome-neutral service terms before confirmatory work begins.
   Welcome exploratory results with their exploratory status visible.

The initial completion target should be a small set of independently reproduced,
reusable results with functioning funding and appeals. Whole-volume coverage and
discovery of new physical laws are not promised deliverables.

## Reward policy

Existing results may qualify for first-formalization or proof-improvement rewards
without being labeled discoveries. Independently useful intermediate results,
software, replication and informative negative results can qualify in their
published categories. Assess a contribution's additional value before dividing
its award among collaborators.

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
direction; it adds no Lean proofs, empirical qualification or live funding.

- [ ] Publish an initial statement inventory with source references, assumptions,
  dependency gaps, supported evidence profiles and bounded acceptance criteria.
- [ ] Produce original mechanics artifacts and obtain independent reproduction.
- [ ] Qualify reviewers and calibrate reference tasks before opening a funded round.
- [ ] Exercise correction, rejection and appeal cases on the selected corpus.
- [ ] Publish a separately qualified field-theory and empirical extension plan.

The [roadmap](../ROADMAP.md) retains the independent audit, operator qualification,
funded deployment and recovery gates required before public activation.
