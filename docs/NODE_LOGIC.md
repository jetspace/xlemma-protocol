# Node logic and consensus operations

## 1. Consensus objective

Nodes establish that independent operators reproduced the same protocol-bound observation. They do not decide formal truth by token vote.

## 2. Formal job sequence

```text
researcher commits claim and artifact
  → payment quote and backed-credit authorization
  → discover signed service advertisements
  → bind service orders to exact advertisement sequences
  → commit the eligible set and future randomness round
  → committee selected from eligible independent operators
  → ASTRA may create/repair candidate
  → Lean build node creates deterministic export
  → each checker executes without seeing other verdicts
  → each checker commits observation hash
  → commitment threshold reached
  → receipts are revealed
  → policy engine compares exact roots and checker families
  → Certified / Rejected / Divergent / Insufficient
  → challenge period
  → economic finalization or quarantine
```

## 3. Committee requirements

Committee policy is a Boolean expression over roles and diversity dimensions, not a single integer threshold. A default Gold policy is:

```text
(kernel operator A AND kernel operator B)
AND independent checker operator C
AND 3 distinct OperatorClusterIDs
AND at least 2 infrastructure providers
AND at least 2 regions
AND all required roots equal
AND all required checker families agree
```

The node selector first filters for active collateral, role, checker families,
conflict rules, software identity, and explicit minimums across formal accuracy,
availability, latency, novelty calibration, challenge quality, and
independence. Random hash ranking occurs only after the exact eligible set and
future randomness source are committed. Stake or reputation above a threshold
does not create more mathematical authority.

Every published selection includes member rank hashes and a selection root.
Any observer can reproduce the committee from the sortition request, eligible
records, revealed randomness, and selection time. One conservative
`OperatorClusterID` can fill at most one committee slot, and provider/region
minimums are enforced across the complete assignment.

## 4. Service discovery and orders

Each node advertises signed roles, endpoints, checker or implementation
families, theories/domains, hardware, capacity, observed latency, price,
reputation snapshot, bond, terms, and validity window. Discovery results are
deterministically ordered by compatible unit price, p95 latency, and
AdvertisementID. A `ServiceMatch` reserves capacity under one immutable
advertisement sequence. Matching, payment settlement, and proof certification
remain separate transitions and receipts.

## 5. Commit-reveal

```text
commitment = H(JobID || Verdict || ObservationRoot || Salt)
```

A valid reveal must match the earlier commitment. Missing reveal after a binding commitment may forfeit a timely-completion bond. A node is not required to reveal secret proprietary implementation details, but it must disclose enough deterministic trace commitments to support reproduction and dispute.

## 6. Formal outcome table

| Evidence | Outcome |
|---|---|
| All required families pass, roots match, diversity satisfied | `CERTIFIED` |
| All required families fail on the same bound artifact | `REJECTED` |
| Pass/fail mix, error, or root mismatch | `DIVERGENT` |
| Missing role, operator, provider, region, or reveal | `INSUFFICIENT` / unchecked |
| Later contradictory evidence or checker compromise | `QUARANTINED` |

A failed proof can still earn a verifier its execution fee and can become a valuable negative-result artifact.

## 7. Novelty outcome

Novelty nodes report probabilities and evidence roots. Reviewer weight is capped and based on historical calibration, domain fit, operator independence, evidence quality, and disclosed conflicts. Reviewers are not rewarded for matching the current majority. Long-term calibration is scored against later adjudication and observed research use.

## 8. Significance and impact

Significance predictions do not immediately create large token rewards. The mature graph records:

- final proof dependencies;
- independent implementations;
- verified citations;
- certified derivative results;
- paid API or artifact use;
- compute and proof-state savings;
- commercial deployment evidence;
- reproductions and challenges.

## 9. Operator clusters

The protocol assumes one key is not one operator. A cluster can be inferred from declared control, common payout addresses, key custody, cloud account, build signatures, network patterns, correlated failures, common funding, and governance relationships. False declaration is slashable only when supported by objective evidence.

## 10. Failure handling

- A checker crash is an `ERROR`, not a `FAIL`.
- Timeout is an `ERROR` and can trigger replacement selection.
- Root mismatch is always divergence.
- The network never retries against a silently changed artifact; a changed artifact receives a new ID/job.
- A quarantined proof pauses new economic activation while preserving history and attribution.
- Emergency governance may quarantine but cannot change formal observations.
