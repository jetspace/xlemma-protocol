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
enables further discovery while widening access to the resulting capabilities.
The working hypothesis is that making results easier to reproduce, challenge,
connect and reuse will improve what subsequent researchers can accomplish with
their resources. Test scientific gains and access separately with independent
users; growth in one does not establish progress in the other.

The Culture provides a literary reference for abundance, curiosity and freedom.
Our chosen lesson is that powerful technology should support agency and shared
capability. Fiction supplies an aspiration, not a demonstrated technological path
or authority over participants' values. See [Banks's own account](https://redsails.org/a-few-notes-on-the-culture/).

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

## Research navigation decision — 2026-09-11

Develop an opt-in [local retrieval layer](RESEARCH_RETRIEVAL.md) that helps
researchers find useful existing artifacts and declared dependencies. The first
prototype compares exact tokens, sparse lexical vectors, explicit graph navigation
and their combination. Its development cases show both useful matches and
assumption-confusion failures. It introduces no scientific or payment authority.
Independent evaluations of verified reuse, cost and access remain necessary
before adopting it as a default workflow or claiming research acceleration.

## What should compound

Research should create durable capabilities: checked mathematical libraries,
algorithms, numerical methods, measurement techniques and reusable evidence.
Those assets should make further questions tractable, improve reliability, or
reduce the resources needed for subsequent work. Preserve failed approaches and
corrections so later researchers can understand and avoid the same mistakes.

The intended loop is research, independent verification, usable shared artifacts,
and further research that benefits from those artifacts. Count maintenance,
revalidation and onboarding as part of keeping the loop working. A growing corpus,
more model tokens or greater computing expenditure does not establish compounding.
No constant or exponential rate of scientific return is assumed.

Assess gains under declared quality and resource constraints. Record human time,
compute, energy where measurable, and laboratory needs separately. More affordable
capability is useful even when total demand rises; neither resource savings nor
universal access can be inferred from a faster benchmark alone.

Research capability compounding is distinct from the optional reinvestment of
settled revenue in [the economics model](ECONOMICS.md#auto-compounding). Reuse does
not create a payment debt, mint backing, or establish a claim on future discoveries.

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
| Downstream extension | Independent teams tackle held-out tasks under a preregistered comparison, recording artifact reuse, failures, repair and full costs. |

Finalize exact statements only after checking existing library coverage and
reviewer availability. Do not require a generic formalization of all variational
calculus before a useful restricted statement can qualify. Narrow or replace a
pilot item if its dependencies exceed available review capacity.

A later empirical branch can compare a model with measured data using a qualified
method and an uncertainty budget. It requires independent experimental capability;
the core pilot can test formal and computational reuse before that branch opens.
Field-theory research can proceed in parallel when its own dependencies and
verification requirements are met.

The [research compounding pilot](RESEARCH_COMPOUNDING_PILOT.md) specifies the two
generations, baseline conditions, access checks and evidence required for a
decision to expand. Its experimental protocol is planned work to implement and
operate; it is separate from the existing synthetic economic simulator.

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
| Does accumulated work improve future capability? | Held-out task results under fixed resource limits or costs to reach a fixed quality target, including asset creation, maintenance and verification. |
| Are failures caught? | Detection and correction of declared adversarial cases, later reversals and unresolved disputes. |
| Can people participate fairly? | Queue delays, admission failures, assisted access, appeal outcomes and review of legitimate work excluded. |
| Are the gains broadly accessible? | Usable artifact rights, fresh-environment reproduction, independent export/reuse, access costs and concentration of funding, compute and review control. |
| Is the system economically viable? | Actual cost per completed workflow, funded verification capacity and exact spending conservation. |
| Does research reach practice? | Documented use by an independent practitioner and the resulting measured benefit or failure, with uncertainty and limitations. |

These are evaluation measures, not automatic payout formulas. Do not optimize
for theorem count, token volume, treasury size, citation count or self-reported
social impact. Reward rules remain prospective, budget-conserving and appealable.

## Shared capability and accountable power

Keep the existing commons, portable-exit and independent-appeal commitments central
to implementation. For the pilot, contributors must explicitly agree to usable
artifact rights before inclusion; historical rights are not rewritten. Test that
another participant can retrieve, understand and reuse an artifact without the
original producer's private service or a discretionary downstream permission.
Any such dependency must be disclosed and limits the access claim.

Measure who controls funding, compute, assessment and review using disclosed
control relationships, with uncertainty visible. Protect assisted access and
foundational exploration within published budgets. Preserve multiple research
paths and provider choices rather than ranking all questions by one model's
predicted social value. Treat model competence, participant consent and governance
authority as distinct. AI proposals still require the applicable independent
evidence; no provider gains authority by supplying more compute.

Software alone cannot establish a benevolent society. Energy, manufacturing,
institutions and accountable technology deployment remain additional work. xLemma
should demonstrate its contribution through reliable research and accessible
capabilities, without claiming to settle that wider social design.

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
  artifact licenses and accepted evidence profiles under the research compounding pilot.
- [ ] Preregister held-out tasks, baseline conditions, resource/quality limits,
  accounting and access criteria before generating the pilot's reusable assets.
- [ ] Produce the connected artifacts and minimum researcher workflow; keep all
  scientific and empirical claims scoped to their actual evidence.
- [ ] Obtain independent reproduction and downstream task comparisons, including
  a correction/appeal exercise and fresh-environment export/reuse.
- [ ] Publish scientific gains and access results separately with full costs and
  control-concentration disclosures; revise if overhead or access barriers
  undermine the claimed benefit. Report inconclusive or negative outcomes.
- [ ] Open a bounded funded pilot only after the existing activation gates pass.
- [ ] Expand when independent use identifies a concrete next bottleneck and the
  required reviewers or laboratories are available.

This strategy is not evidence that any pilot, partnership, funding round or human
benefit has already occurred. Success should earn the next expansion through
observable research progress.
