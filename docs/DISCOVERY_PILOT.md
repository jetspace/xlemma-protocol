# Open discovery pilot

The local pilot implements reproducible economic state transitions before any
funded network activation. `xlemma-economics::discovery` powers the CLI and the
adversarial runner. It does **not** authenticate funding or evidence, execute
checkers or experiments, issue certificates, or authorize/execute USDC
transfers. Reports carry these limits as machine-readable flags. Amounts use
six-decimal simulated USDC units; all fixture identities/evidence are synthetic.

## Run and review

```sh
cargo run --locked -p xlemma-cli -- simulate-discovery examples/discovery/pilot.json
python3 scripts/simulate_discovery.py --check
cargo test --locked -p xlemma-economics --test discovery
```

The runner builds the CLI, exercises the same Rust implementation, compares
`examples/discovery/expected-report.json`, and checks the committed
`reports/discovery-simulation.json`. Run without `--check` to regenerate the
attack report after reviewing changes. It contains no second reward allocator.
Input/output schemas are `schemas/discovery-simulation*.schema.json`; these
are local formats, not new XLMP messages or API settlement endpoints.

## Seven improvements and implementation boundaries

| Improvement | Executable pilot | Required for deployment |
|---|---|---|
| Separate rewards | Fixed discovery, formalization, proof-improvement, replication, research-tools, and negative-result budgets; restricted categories cannot subsidize each other | Protected foundational/domain allocations and independently calibrated assessment policies |
| Physics evidence progression | Existing verification profiles; empirical profile requires methods, instruments, provenance, uncertainty, robustness and replication commitments | Qualified laboratories, instrument/calibration verification, authenticated data and independent reproductions |
| Correction and null results | Informative registered null/counterexample category; registered replication earns the same modeled reward for positive and null outcomes; verification costs are verdict-neutral | Outcome-neutral registered-method contracts, meaningful-result assessment, actual timestamps and data provenance |
| Collaboration | Contributor shares, manifest commitments, explicit split dust, attribution appeals; duplicates cannot replace payees | Contributor signatures/consent, secure commit/reveal, contribution and simultaneous-discovery policies |
| Verification capacity | Admission caps, category review reserves, pool-funded appeal access and deadlines | Fair distributed queues, commons-assisted intake, operator service orders and cost/wait forecasts |
| Funding transparency | Donor/admin declarations, mandate roots, category restrictions, fees, settlement replay history, ignored pledges and donor concentration | Authenticated funding/mandates, reconciled treasuries, multi-round runway reporting and institutional commitments |
| Adversarial evaluation | Generated conservation/partition properties and measured synthetic leakage/exclusion cases | Independent hostile corpora, empirical detection/error rates and economic/security audits |

There is no buyer, affiliation or posted-bounty requirement in the discovery
fixture. Weight is an assessor input, not a universal scientific-value metric
or an implemented difficulty oracle. Reference cost may inform a future
calibration policy; raw tokens and reported compute never enter allocation.

## Economic state machine

The policy is fixed during replay and content-derived into a `DiscoveryRoundID`.
Opening requires declared category funding net of intermediary fees, funded
reviews, separate qualified assessor/appeal rosters, bounded capacity, and the
full profile challenge window after submissions close. Donors and administrators
are excluded from reviewing work funded by this pilot. Their declared control
identities are not independently authenticated here.

Events are `submit`, `appeal`, `resolve`, `finalize`, and `expire`, with explicit
nondecreasing times. Closed rounds accept no new events. Event receipts commit
the ordered history, policy, funding disclosures, and declared prior settlement,
group and claim histories. Corrections replay into private state while retaining
the old events; the private state is not a deserializable payment authority.

An admitted submission models a completed assessment, so its fixed fee is
charged for supported, rejected, divergent and inconclusive evidence alike.
An appeal reserves its review fee on admission and spends it only on resolution.
All pilot reviews are pool-funded. One appellant cannot repeatedly appeal the
same submission; other appellants may file, subject to global capacity/reserves.
Production still needs fair intake and escalation when these finite limits are
reached. Pool-funded access is not a promise to serve unlimited demand.

Appeal panels exclude original assessors, contributors, the appellant and
disclosed funder/admin control clusters. Remedies uphold, correct reward/grouping,
correct contributors with a new manifest, or require revalidation. Economic
correction cannot change evidence status; revalidation leaves it inconclusive
and unpaid. A subsequent verified packet needs a subsequent reviewed submission.

Finalization waits for the appeal window and all admitted appeals, then
recalculates the entire batch. Expiry retains unallocated funds and unresolved
cases without declaring their research false. Refunds and post-settlement
recovery are not implemented. Each category calculates:

```text
group award = floor(solver budget * group weight / total eligible group weight)
deposits = administrator fees + completed verification + completed appeal work
         + contributor allocations + retained funds (reserves and dust included)
```

Zero-weight categories retain their solver budget. Contributor splits round
down and retain dust. Prior-settlement and reward histories prevent replay only
if complete and independently maintained; they are caller declarations here.
The retained balance is not a multi-round sustainability forecast.

## Duplicate and exclusion measurements

Repeated economic groups cannot increase weight or replace the first eligible
assessment's contributors. Exact ClaimIDs also prevent repeated discovery and
first-formalization awards within their respective categories. Formalization
of known mathematics remains separately eligible. Improvements, replications,
tools and informative negative results may refer to known claims; independent
assessment must establish their additional contribution.

First eligible assessment order is a deterministic simulation rule, not a
secure discovery-priority protocol. Production must use signed commit/reveal,
published simultaneous-arrival rules and contributor agreements. A copied
reveal does not establish prior authorship.

The attack report deliberately measures both failure directions:

- recognized partition/identity flooding adds zero reward but uses bounded
  review resources;
- an unrecognized semantic duplicate with a fresh claim/group receives **150
  simulated USDC**, showing that independent grouping remains an activation gate;
- overbroad grouping excludes one legitimate contribution, which a modeled
  independent appeal restores;
- inflated compute leaves payouts unchanged; pledged funding, repeated
  settlements, conflicted panels, capacity overflow and premature finalization
  are rejected or held as appropriate.

These are constructed cases, not real-world error rates. A successful report
means the expected behaviors, including assessor failure, were reproduced.
It does not prove perfect novelty, equivalence or difficulty detection.

## Experimental physics profile

`examples/discovery/physics-profile.json` uses the existing profile schema with
stricter empirical evidence. The proposed first laboratory workflow compares
two preregistered predictive models against independent instrument measurements:

1. Commit hypotheses, assumptions, protocol, calibration plan, uncertainty
   treatment, exclusions, analysis and stopping criteria before confirmatory
   collection. Exploration remains a distinct route; retrospective registration
   does not make exploratory findings confirmatory.
2. Preserve raw data, instrument/calibration records, code, environment and
   provenance under immutable roots and appropriate access controls. A reviewed
   method contract commits its work budget before the outcome is known.
3. Independent operators reproduce the analysis; independent laboratories
   replicate the experiment. Shared instruments, datasets or controllers must
   be disclosed rather than counted as independent evidence.
4. Publish separate findings for model derivation, computational reproduction,
   measurement, replication and limitations. A rigorous informative null result
   or counterexample is eligible; missing data or a failed method is not useful
   simply because its verdict is negative.
5. Preserve dissent, allow evidence/reward appeals, and revalidate after
   corrections or instrument problems. No panel votes a law of nature into truth.

The pilot checks evidence commitments and declared registration ordering, not
their authenticity or scientific content. No laboratory is qualified and no
new physics is claimed by this fixture. The existing formal/research verifier
adapters remain the future authenticated evidence boundary.
