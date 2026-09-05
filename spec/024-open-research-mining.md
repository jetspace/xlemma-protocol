# XLIP-024 — Open research mining, verification, and appeals

Status: normative design requirements; an offline economic/appeal state-machine
pilot is implemented in `xlemma-economics::discovery`. It accepts unauthenticated
synthetic inputs and cannot certify or pay. A funded open discovery network and
operated appeal service are not implemented or activated. Existing funding, verification,
novelty, challenge, and escrow primitives do not by themselves enforce this
specification. Activation requires the gates below and the existing
constitutional economic-policy process. No new XLMP messages or schemas are
declared implemented by this document. Local simulation schemas, CLI, reference
vectors, and adversarial reports are documented in
[the pilot guide](../docs/DISCOVERY_PILOT.md).

## Purpose and funding

Researchers freely choose what to discover. Pooled funding rewards verified
contributions under transparent, budget-conserving rules; institutions finance
research while independent verification determines what evidence supports.

An open discovery pool MUST accept unsolicited submissions within its published
supported verification profiles without a posted bounty, institutional
affiliation, or immediate commercial demand. Domain-restricted institutional
pools and specific bounties MAY coexist, but MUST disclose their scope before
participation and MUST NOT present restricted funding as unrestricted funding.
Unsupported profiles MUST be identified explicitly; absence of a qualified
verifier is insufficient evidence, not a finding that the claim is false.

Institutions MAY raise, aggregate, and administer funds. Each intermediary MUST
publish its mandate, fees, restrictions, destination, settlement evidence, and
conflicts. A pledge or prospective grant MUST NOT count as settled funding.
Funding does not confer verification authority or rewrite attribution.

Discovery rewards are direct USDC distributions from settled external funding;
research credits are optional service prepayments, never a mining subsidy.
Restricted funds MUST remain restricted to their authorized purpose. Existing
credit backing and accrued rights MUST NOT be diverted into a discovery pool.

## Contribution and conservation invariants

1. Verification establishes evidence sufficiency under explicit assumptions;
   it MUST NOT alone create a financial entitlement.
2. Reward policies MUST separately identify new discovery, first formalization
   relative to a stated corpus cutoff, material proof improvement, and useful
   independent replication. Known results MUST NOT be sold as novel discoveries.
   Foundational work MUST NOT need commercial demand to qualify for the open
   pool. A replication reward requires evidence of added independent assurance,
   not merely another copy of an existing receipt.
3. Renaming, resubmitting, or artificially partitioning the same contribution
   MUST NOT increase its aggregate reward, including across identities and
   rounds. ClaimID inequality, additional proof steps, or a new operator key
   MUST NOT establish additional rewardable contribution.
4. Equivalent or overlapping contributions MUST have an evidence-backed
   grouping assessment and reward history. Formal equivalence assertions
   require a proof object under the relevant theory. Suspected economic
   redundancy MAY trigger a reasoned reward hold without declaring formal
   equivalence. Grouping preserves attribution and MUST be appealable.
   Independently useful intermediate lemmas MAY qualify for added rewards only
   with evidence of an additional contribution under the published category.
5. Assessment MUST NOT reward raw token consumption, elapsed time, proof length,
   submission volume, or operator count. Independently calibrated, domain-bound
   reference difficulty/cost MAY inform weights, with uncertainty, estimator
   independence, and caps disclosed. The solver's own cost claim is not an
   independent estimate. No universal scientific-value unit is implied.
6. Round rules MUST be committed before opening: scope, categories, calibration
   version, weights, grouping rules, deadlines, priority/tie rules, contributor
   splits, admission limits, fees, reserves, appeals, and expiry disposition.
   Later policy versions apply prospectively. Faster submission MUST NOT waive
   evidence, eligibility, or appeal requirements.
7. A pool MUST separately account for settled deposits, payouts, committed
   unpaid allocations, verification/appeal reserves, fees, and available funds.
   Every amount MUST be nonnegative and their accounting MUST conserve the
   deposit. Funds reserved elsewhere MUST NOT be allocated again. No positive
   minimum payout or compute-cost reimbursement is implied by submitting work.
8. Verifiers MUST be paid for authorized, honestly completed checks regardless
   of PASS, FAIL, or an evidence-supported inconclusive outcome. Bounded service
   admission and resource limits MUST prevent submission or appeal floods from
   creating unlimited verification liabilities. Access policies MUST include
   a published commons-supported route for researchers unable to prepay.

An initial candidate allocator is `floor(B * w_i / sum(w))`, where `B` is the
round's solver budget after costs/reserves and `w_i` is the finalized weight of
an eligible contribution group. This is a design candidate, not a selected or
implemented scoring oracle. The offline pilot exercises this candidate only.
Any adopted allocator MUST use checked integer
arithmetic, reject invalid weights, retain rounding dust, carry forward a
zero-weight budget, and split group awards only under the recorded allocation
policy. It MUST settle each entitlement at most once.

Timely appeals MUST be resolved before affected rewards become spendable. If
an appeal can change a shared weight denominator, the affected set includes
every dependent allocation: hold that whole allocation batch unless a tested,
fully funded isolation scheme proves conservation. Pending weights and USDC
estimates MUST be displayed as provisional, not guaranteed earnings.

## Verification and appeals

The common standard is a versioned evidence process with domain-specific
profiles (XLIP-003 and XLIP-022), not one universal verdict about reality.

1. **Submission:** bind the claim, artifact/environment/dependency roots,
   assumptions, contribution manifest, verification profile, economic policy,
   round, and signed commitment. Unsolicited claims MUST be fixed before their
   verification assignment; a solver cannot substitute a weaker statement
   after seeing the result. Publish commit/reveal and simultaneous-discovery
   rules. Commit priority is evidence of submission timing, not proof of novelty.
2. **Independent checks:** require qualified, conflict-disclosed operators and
   the diversity specified by the profile. Producers, fund administrators, and
   their controlled operators MUST NOT count as independent final verifiers
   of their own submissions. Formal checks reproduce exact objects under
   declared axioms and checker versions. Empirical physics binds data provenance,
   methods, calibration, uncertainty, analysis, and profile-required independent
   replication. A theorem within a physical model or a reproducible simulation
   MUST NOT be labeled experimental confirmation of that model.
3. **Separate decisions:** publish signed evidence-sufficiency, alignment,
   contribution-category, and reward decisions, each bound to its exact inputs
   and policy. Give reasons, prior-art corpus coverage/cutoff, grouping evidence,
   assessment uncertainty, conflicts, and the appeal deadline. Preserve FAIL,
   ERROR, missing evidence, and dissent. Required checker disagreement MUST
   cause divergence/quarantine, never majority certification.
4. **Appeal access:** authors and challengers MUST be able to dispute checking,
   statement alignment, prior art, grouping, attribution, metering, conflicts,
   procedure, or allocation. An appeal binds the decision, grounds, supporting
   evidence, requested remedy, appellant, and timestamp in a signed record.
   Publish capped fees/bonds, independently funded access, admissibility,
   response/review deadlines, and objective abuse criteria. Losing a good-faith
   appeal MUST NOT alone count as abuse. Confidential artifacts remain subject
   to access controls; publish evidence commitments and redacted reasons.
5. **Independent review:** select a fresh qualified review panel excluding the
   original decision makers and conflicted beneficial-control clusters. Formal
   disputes require independent reruns or corrected evidence; reviewers MUST
   NOT vote a failed proof into validity. Reward/procedure disputes use the
   original published economic policy and documented evidence. Every decision
   MUST state reasons, dissent, remedy, and the next available review route.
6. **Holds and remedies:** evidence disputes hold affected certification and
   payouts; purely economic disputes hold affected allocations without erasing
   validity. Outcomes MAY uphold, correct grouping or weights, reassign review,
   require revalidation, or reject eligibility. New records supersede rather
   than mutate historical decisions. A fresh review or a new policy does not
   erase the original record.
7. **Timeout and finality:** no response MUST NOT imply acceptance or forfeiture
   of the researcher's claim. Rules MUST specify independent escalation,
   service-failure remedies, maximum review duration, permitted further appeals,
   and disposition of held funds on expiry. Repeated unsupported filings MUST
   NOT restart deadlines automatically. Unresolved cases stay unpaid and
   explicitly unresolved; expiry cannot turn them into a scientific rejection.
   Corrections after settlement append evidence and may quarantine future use.
   Monetary remedies require a previously funded reserve or enforceable
   pre-agreed recovery mechanism; automatic recovery of spent USDC MUST NOT
   be promised.

## Activation gates and unresolved trust assumptions

Novelty, difficulty, and economic grouping are contestable assessments, not
perfect oracles. Formal identity alone does not detect every equivalent or
trivially extended claim. Institutional independence and prior-art coverage
also depend on evidence outside the checker. Deployments MUST publish these
limitations and evaluate false-positive exclusion of legitimate foundational
work as well as false-negative reward farming.

Before opening funded discovery rounds, implement and independently review:

- signed round/assessment/appeal records, schemas, replay-safe lifecycle, APIs,
  institutional-fund restrictions, and atomic funded settlement;
- domain calibration, evidence-backed grouping and cross-round reward history,
  assessor qualification, conflict handling, and accessible appeal operations;
- property tests for conservation, nonnegative balances, zero weights, dust,
  replay, and shared-denominator appeal recalculation;
- adversarial tests for arithmetic-identity floods, renamed/partitioned proofs,
  identity rotation, cross-round duplicates, inflated compute, fabricated
  replication, institution/assessor collusion, and submission front-running;
- end-to-end tests for a foundational result without a buyer, a first
  formalization, a genuinely useful intermediate lemma, independent replication,
  a successful grouping appeal, a checker disagreement that cannot be voted
  away, review timeout, fee assistance, and post-settlement correction.
