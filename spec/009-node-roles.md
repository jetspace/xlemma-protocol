# XLIP-009 — Node roles, selection, and incentives

## Roles

Researcher, research prover, Lean builder, official checker, independent checker, novelty reviewer, significance reviewer, challenger, storage provider, indexer, payment facilitator, and certificate finalizer are distinct logical roles. ASTRA is one implementation of the generic research-prover role.

## Separation

A researcher MUST NOT satisfy independent verification for their own job. A proof producer MUST NOT be the sole final checker. A challenged operator MUST NOT adjudicate its own challenge.

## Selection

Nodes first satisfy collateral, role, checker-family, active-status, software, conflict, and every required reputation-dimension criterion. A committed future manipulation-resistant random seed then hash-ranks the committed eligible set. The selection and rank hashes MUST be exactly reproducible. Stake above the eligibility threshold MUST NOT create selection weight or formal voting power.

Committee selection MUST enforce unique conservative operator clusters and the policy's minimum infrastructure-provider and region diversity. It MUST fail closed if no independent assignment exists. XLIP-019 defines the native sortition request and committee-selection records.

## Operator clusters

Policies MUST count conservative operator clusters rather than public keys. High assurance SHOULD require multiple operators, checker implementations, providers, and regions.

## Discovery and service market

Nodes publish signed, expiring, append-only `NodeServiceAdvertisement` records. Discovery filters explicit service, role, checker family, theory, domain, capacity, latency, price, bond, reputation, and operator-exclusion constraints. A `ServiceMatch` binds an order to one exact advertisement sequence; a later price or capacity update cannot silently rewrite it. See XLIP-019.

Reputation is a six-dimensional evidence vector: formal accuracy, availability, latency, novelty calibration, challenge quality, and independence. No scalar score is authoritative, and strength in one dimension MUST NOT compensate for failure in a required dimension.

## Compensation

Verifier base compensation MUST depend on complete reproducible execution, not `PASS`. Accepted-proof bonuses MAY apply to prover nodes. Delayed calibration rewards MAY apply to reviewers.

## Slashing

Only objectively provable misconduct SHOULD be slashable. Honest dissent or checker divergence MUST NOT be automatically slashed.
