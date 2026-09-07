# Research compounding pilot

Status: proposed evaluation and implementation work, 2026-09-07. No experiment,
independent participant result or acceleration rate is claimed by this document.
It implements the planning part of the [project direction](PROJECT_DIRECTION.md);
it is distinct from the [synthetic economic pilot](DISCOVERY_PILOT.md).

## Questions and scope

Test whether one generation of research improves what independent researchers can
achieve in the next, and whether the resulting capabilities are accessible outside
the producing team. Report scientific performance and access as separate outcomes.

Start with the proposed oscillator, energy-balance and controlled-approximation
collection in the [physics program](FOUNDATIONAL_PHYSICS.md). Select exact tasks
after inventorying upstream mathematics, qualified reviewers and resource limits.
This corpus tests a method of research; it does not restrict the wider network.

## Two generations and comparison conditions

Generation one produces a frozen, versioned collection of original artifacts,
dependencies, readable explanations, reproduction instructions, rights and
correction history. Identify which elements were already available upstream.
Record the full effort to create, check, package and maintain the collection.

Generation two uses researchers independent of the producing team to tackle
previously unseen tasks. These must require an additional result; merely renaming
a theorem or repeating an already packaged solution is insufficient.

| Condition | Resources available |
|---|---|
| Ordinary workflow | The declared public literature, pinned upstream libraries, Git, Lean and ordinary research tools, under the registered resource limits. |
| Artifact reuse | The same baseline resources plus the frozen generation-one collection, delivered through an ordinary repository. |
| xLemma workflow | The same artifacts and resources, accessed through xLemma's evidence, retrieval, correction and collaboration workflow. |

Artifact reuse versus the ordinary workflow estimates the value of the new
collection. xLemma versus artifact reuse estimates the protocol's additional
benefit or overhead. Compare all three before attributing ordinary library reuse
to the protocol. Required scientific assurance is equivalent across conditions;
account for equivalent review effort in the baseline too.

Assign comparable tasks and balance participant expertise and task order where
feasible. Prevent participants from carrying a solution from one condition into a
repeat of the same task. Disclose unavoidable spillovers and prior familiarity.
Pin model versions, tools, inputs and permitted assistance, and record changes.
Producer help is measured assistance, not invisible independence.

## Registration before the experiment

Publish the following before artifact production and evaluation begin:

- Exact scope, accepted evidence profiles and upstream dependency inventory.
- Task-selection method, comparison design, participant/conflict criteria and
  recruitment plan. The initial minimum is two downstream researchers from
  distinct disclosed control domains, neither under the producer's control;
  this is a feasibility floor, not statistical power or checker qualification.
- Resource limits and either a fixed quality target or a fixed resource budget,
  with success, failure, timeout and withdrawal rules applied consistently.
- Scientific and access decision thresholds, sample-size rationale, stopping rules
  and the treatment of missing data. Do not invent thresholds after seeing results.
- All cost categories, measurement methods and the allocation of shared setup costs.
- Rights needed for independent reuse, publication scope, consent and privacy rules.

An independent evaluator selects and commits to held-out tasks and judging criteria
before releasing them to downstream researchers. Keep solution-bearing task details
from producers until the collection is frozen. Salt commitments to low-entropy
private task descriptions; publish openings with the permitted evaluation record.
Record suspected contamination or earlier task exposure. Hidden tasks assess
generalization only; they cannot silently change a live funding round's criteria.

Register amendments prospectively with reasons. Retain original plans and results;
separate exploratory follow-up from the registered comparison.

## Required evidence and accounting

| Record | Required contents |
|---|---|
| Task manifest | Version, assumptions, inputs, expected evidence, resource limits, criteria and held-out commitment/opening as applicable. |
| Artifact manifest | Original versus upstream work, exact versions, dependencies, permissions, producer attribution and qualified verification bindings. |
| Attempt record | Condition, task, participant control disclosure, tool versions, start/end, human effort, compute usage, assistance, outcome and failure reason. |
| Assurance record | Exact independent reproduction, statement alignment, dissent, quarantine, corrections and appeal history appropriate to the claim. |
| Resource ledger | Generation-one creation/review, generation-two work, onboarding, retrieval, maintenance and revalidation; include unsuccessful attempts. |
| Access record | Availability, artifact rights, operating requirements, assisted access, queue/admission outcomes, export/reuse and control-concentration disclosures. |
| Evaluation report | Per-condition results, comparisons, uncertainty, missing data, contamination, deviations and the resulting decision. |

These are proposed record requirements, not implemented schemas or new XLMP
messages. First map fields to existing manifests and receipts; specify missing
evaluation records with reference vectors before writing collection tooling.
Do not put private keys, personal data or unpublished proof contents in logs.

Record human effort, wall-clock delay, machine time, monetary expenditure and
measured energy separately. Token counts are provider-specific usage records,
not a universal compute measure. Disclose unavailable energy data; estimates need
their assumptions and uncertainty. Neither estimated savings nor future reuse
counts as settled money.

Report both marginal downstream cost and total cost including generation one.
Avoid counting the same creation cost once per consumer or claiming the same
saved work multiple times. Publish any amortization assumptions. Revalidation
under the declared trust policy still applies when artifacts are reused.

## Access and correction exercises

Require a participant outside the producing control domain to retrieve, reproduce
and reuse the published collection in a fresh environment. Demonstrate an export
that preserves attribution, evidence and usable rights, and document dependence
on any centrally controlled service. A successful file download alone is not
portable research capability.

Exercise a deliberately invalid assumption or artifact substitution and a reasoned
correction/appeal path. Preserve the original record and measure reviewer effort
and resolution time. Qualified independent checks establish the scientific outcome;
an appeal cannot turn a failed check into a pass.

Report access costs and queue outcomes, including participation without an upfront
payment through reserved assisted capacity when available. Publish aggregate
concentration of funding, compute and reviewer control with missing disclosures
visible. Agree thresholds and remedies before the evaluation, without weakening
verification requirements for any participant.

## Decision rules

- Advance scientific scope when declared criteria support useful additional
  capability or lower full costs at the same required quality. Report the exact
  task population supported by the evidence.
- Advance access claims only when independent use, rights, portability and
  registered access criteria are demonstrated. Scientific performance cannot
  compensate for a failed access criterion in an aggregate score.
- If the library helps but the protocol adds excessive overhead, simplify the
  workflow and retain the useful artifacts. If neither helps, revise the chosen
  tasks or method rather than scaling compute automatically.
- If evidence is insufficient, report an inconclusive result and justify a further
  experiment. Two generations cannot establish indefinite or exponential returns.

## Implementation order and completion gates

| Order | Deliverable | Completion evidence |
|---|---|---|
| 1 | Scope and upstream inventory | Exact tasks, missing dependencies, licenses and available review capacity recorded. |
| 2 | Registered evaluation and record definitions | Reviewed comparison plan, role/conflict mapping, prospective thresholds and machine-readable reference vectors. |
| 3 | Connected artifacts and minimal workflow | Reproducible collection, readable assumptions, retrieval/export and correction paths exercised locally. |
| 4 | Independent second-generation evaluation | All three conditions run on held-out tasks with full attempt and cost records. |
| 5 | Access and control review | Independent portability/reuse, assisted-access results, appeals and concentration disclosures. |
| 6 | Public decision report | Scientific and access outcomes separately assessed, limitations stated, and next work justified. |

Local rehearsals use labeled test identities and cannot satisfy independent-user
gates. No new reward formula or automatic reuse royalty is introduced. Any actual
USDC-funded operation still requires the existing [activation gates](../ROADMAP.md#open-research-mining--activation-gates),
settled funding, qualified operators and independent appeals. Honest completed
review remains outcome-neutral; a negative pilot result does not invalidate work
properly performed under its agreed terms.
