# Project direction: expand humanity's capacity for discovery

Decision date: 2026-09-07. This is the current engineering and research strategy,
subject to revision against evidence. It changes no historical round policy,
reward formula, certificate semantics or researcher's freedom to choose a problem.

## Purpose and working hypothesis

xLemma's long-term purpose is to expand humanity's capacity to understand nature
and turn discovery into new possibilities. Improving lives names that broad
ambition: advances may change what is possible in energy, materials, measurement,
computation and other fields in ways that cannot yet be predicted.

The operational objective is to increase the rate at which reliable knowledge
accumulates and enables further discovery. The working hypothesis is that making
results easier to reproduce, challenge, connect and reuse will reduce the cost of
subsequent research. The protocol must test that hypothesis with independent users.

Landau's organizing perspective remains central: explicit assumptions, variational
structure, symmetry, conservation, controlled limits and testable consequences.
The research web lets people approach those questions from any direction.
Unification is an open research ambition; the near-term milestone is a demonstrated
improvement in a research workflow.

Keep three horizons open together: foundational structures that deepen understanding,
general-purpose methods that enlarge research capability, and applications that
turn knowledge into practical advances. No fixed sequence or immediate application
is required. A foundational result can matter before anyone knows where it leads.

When a project claims a practical benefit, record the evidence connecting its
result to that outcome. Potential applications, scientific validity, observed
reuse and delivered benefits are separate claims. This reporting discipline does
not require each contribution to predict a social benefit or justify itself by
commercial demand. The pilot below tests the means of acceleration; it does not
set the ceiling of the mission.

## Why this is the next priority

The repository has reference identity, journal, verification, funding, assessment,
appeal and settlement infrastructure. The checked-in Lean example
[`add_zero_verified`](../lean/XLemma/Example.lean) exercises the exporter with
arithmetic; it is not a physics corpus. Local settlement tests establish plumbing,
not faster research or improved lives.

The next uncertainty to resolve is whether researchers can use this infrastructure
to produce and reuse scientifically meaningful work more effectively. Prioritize
a complete, bounded example and researcher feedback over further generalization
of the marketplace. Continue security fixes and required verification hardening.

Use established mathematical libraries where suitable. The community's
[Mathlib](https://github.com/leanprover-community/mathlib4) provides Lean mathematics,
and [Physlib](https://github.com/leanprover-community/physlib) develops formalized
physics, with distinct core and exploratory review paths. Inventory existing work
before claiming a first formalization. Pin compatible revisions, inspect axioms
and review status, and reproduce artifacts under xLemma's declared trust policy.
Upstream inclusion does not substitute for that reproduction. Contribute reusable
work upstream where appropriate and under the project's contribution rules.

Research on automated laboratories likewise emphasizes that access and research
acceleration depend on reproducibility and coordination as well as automation.
That motivates testing these properties here; it does not establish xLemma's
effectiveness. See [Canty et al., 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12022019/).

## First pilot: oscillation, energy balance and controlled approximations

Choose a small connected collection of classical models whose assumptions,
derivations and numerical behavior can be inspected independently. This is an
engineering judgment about a tractable starting point, not a claim that this
subject has the highest ultimate scientific value.

The proposed core is an ideal oscillator, coupled linear oscillators, and a
declared dissipative extension. Together they allow exploration of shared
structure and limits of a model. Potential later applications in measurement,
vibration and control are hypotheses to investigate with practitioners.

| Pilot object | Required result |
|---|---|
| Model and derivation | Explicit parameters, dimensions, admissible functions, regularity and boundary conditions; checked consequences of the declared equations or action. |
| Energy balance | A conditional conservation result for the ideal model and a separately stated balance for the dissipative model. Assumptions must distinguish them. |
| Connection | A precisely scoped relation between the coupled system and a modal description, including conditions under which the transformation is valid. |
| Reproduced computation | Pinned solver, inputs and tolerances; convergence and conservation/balance diagnostics assessed against declared criteria. Numerical agreement alone is not a proof. |
| Adversarial case | A counterexample or deliberately invalid assumption that the review process correctly detects and preserves in history. |
| Downstream extension | Another team uses an artifact in a different, declared task and records what it reused, what required repair, and the effort involved. |

Finalize exact statements only after checking existing library coverage and
reviewer availability. Do not require a generic formalization of all variational
calculus before a useful restricted statement can qualify. Narrow or replace a
pilot item if its dependencies exceed available review capacity.

A later empirical branch can compare a model with measured data using a qualified
method and an uncertainty budget. It requires independent experimental capability;
the core pilot can test formal and computational reuse before that branch opens.
Field-theory research can proceed in parallel when its own dependencies and
verification requirements are met.

## Researcher workflow to build

1. State a question and locate existing results, assumptions, contrary evidence
   and reusable artifacts.
2. Produce an original contribution or justified relationship, with readable
   scientific explanation alongside machine-checkable artifacts.
3. Reproduce it independently; inspect statement alignment and report dissent.
4. Correct or appeal a decision without overwriting its history.
5. Let another researcher retrieve, reproduce and extend the result.

Build the minimum interface needed to complete this workflow: clear evidence
status, assumptions, dependencies, reproduction instructions and correction paths.
Evaluate it with researchers who did not develop the protocol. A graph display is
useful when it helps them answer a question or reuse work.

AI can propose connections, locate missing dependencies, draft proofs and suggest
tests within explicit resource budgets. Its proposals remain hypotheses until
the relevant independent checks pass. Model agreement cannot certify a result;
the producer cannot verify its own output. Stop or revise searches that consume
their budget without producing inspectable progress.

## Measure acceleration without rewarding activity counts

Publish task definitions, baseline workflows and measurement procedures before
the pilot. Include failed attempts, onboarding effort, reviewer time and operating
costs. Where feasible, use matched tasks and counterbalanced task order to reduce
learning and selection effects. A small pilot provides preliminary evidence;
report the sample size, variation and confounders.

| Question | Evidence to collect |
|---|---|
| Is reproduction easier? | Elapsed time, human effort, compute cost and success rate for another team reproducing the same bounded task. |
| Does work accumulate usefully? | Demonstrated reuse in a subsequent task with attributable artifacts and a clear additional result. Downloads and graph links are insufficient. |
| Are failures caught? | Detection and correction of declared adversarial cases, later reversals and unresolved disputes. |
| Can people participate fairly? | Queue delays, admission failures, assisted access, appeal outcomes and review of legitimate work excluded. |
| Is the system economically viable? | Actual cost per completed workflow, funded verification capacity and exact spending conservation. |
| Does research reach practice? | Documented use by an independent practitioner and the resulting measured benefit or failure, with uncertainty and limitations. |

These are evaluation measures, not automatic payout formulas. Do not optimize
for theorem count, token volume, treasury size, citation count or self-reported
social impact. Reward rules remain prospective, budget-conserving and appealable.

## Funding and focus

Protect foundational exploration while maintaining separate support for
verification, replication, tools and negative results. Choose engineering
priorities by the bottleneck relieved, plausible benefit, tractability under
available capacity, reuse potential and evidence of additional contribution.
Publish reasons and uncertainties; no model-generated universal impact score
controls eligibility.

Before live settlement, satisfy the existing audit, independent verification,
operator qualification, funding and appeal gates. Until then, evaluate the
workflow without promising USDC. Institutions can fund shared capabilities and
research access under disclosed terms; their contribution does not determine
scientific verdicts.

Defer additional token products, compute futures, broad marketplace expansion and
large new domain commitments until evidence identifies a need. Keep existing
security and conservation work active. The remaining architectural phases in the
roadmap remain available; their numbering does not require finishing every
financial feature before testing research usefulness.

## Next milestones and decision points

- [ ] Inventory upstream coverage and select exact pilot statements, dependencies,
  artifact licenses and accepted evidence profiles.
- [ ] Produce the connected artifacts and minimum researcher workflow; keep all
  scientific and empirical claims scoped to their actual evidence.
- [ ] Obtain independent reproduction and a downstream reuse case, including one
  correction or appeal exercise.
- [ ] Publish baseline comparisons and complete costs; revise the workflow if
  overhead dominates the observed benefit or independent teams cannot reuse it.
- [ ] Open a bounded funded pilot only after the existing activation gates pass.
- [ ] Expand when independent use identifies a concrete next bottleneck and the
  required reviewers or laboratories are available.

This strategy is not evidence that any pilot, partnership, funding round or human
benefit has already occurred. Success should earn the next expansion through
observable research progress.
