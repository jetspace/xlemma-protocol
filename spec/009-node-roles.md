# XLIP-009 — Node roles, selection, and incentives

## Roles

Researcher, ASTRA prover, Lean builder, official checker, independent checker, novelty reviewer, significance reviewer, challenger, storage provider, indexer, payment facilitator, and certificate finalizer are distinct logical roles.

## Separation

A researcher MUST NOT satisfy independent verification for their own job. A proof producer MUST NOT be the sole final checker. A challenged operator MUST NOT adjudicate its own challenge.

## Selection

Nodes first satisfy collateral, reliability, qualification, active-status, software, and conflict criteria. A manipulation-resistant random seed then selects eligible nodes. Stake above the eligibility threshold SHOULD NOT create unbounded selection or voting power.

## Operator clusters

Policies MUST count conservative operator clusters rather than public keys. High assurance SHOULD require multiple operators, checker implementations, providers, and regions.

## Compensation

Verifier base compensation MUST depend on complete reproducible execution, not `PASS`. Accepted-proof bonuses MAY apply to prover nodes. Delayed calibration rewards MAY apply to reviewers.

## Slashing

Only objectively provable misconduct SHOULD be slashable. Honest dissent or checker divergence MUST NOT be automatically slashed.
