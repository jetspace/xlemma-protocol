# XLIP-006 — Revenue and bounded impact allocation

## Net revenue

Only settled external gross revenue less delivery/service cost, compute cost, refunds, and reserves is distributable.

## Waterfall

Every waterfall MUST total 10,000 basis points. Every creator sub-allocation MUST total 10,000 basis points within the creator pool.

## Auto-compounding

A researcher MAY direct a portion of their settled creator allocation into their vault. Backing MUST enter before or atomically with credit issuance.

## Separate evidence and economic graphs

`FORMALLY_DEPENDS_ON` is an evidence edge and MUST NOT be interpreted as
`OWES_PAYMENT_TO`. A payment requires qualifying settled revenue, an active
economic policy, an eligible economic edge, an unspent bounded pool, and
non-recursive treatment of the revenue event. No formal dependency may by
itself block publication or reuse.

## Upstream pool

Automatic upstream rewards MUST draw from a fixed pool or explicit budget
declared before monetization. Unbounded transitive royalties are prohibited.
Open Commons capsules MUST default this pool to zero and MAY instead receive
retrospective impact-pool allocations.

## Compute-savings impact allocation

A compute-savings signal requires the upstream lemma to appear in the final
proof dependency graph. Savings MUST use a conservative lower-confidence
estimate and MUST be capped by both a configured fraction of downstream net
revenue and an explicit `ImpactPoolAuthorization`. Equivalent duplicate
clusters MUST share or prevent duplicate claims. The estimate is evidence for
an impact allocation, not a precise invoice or independent payment trigger.
Settlement amounts, uncertainty multipliers, and caps MUST use checked
fixed-point integer arithmetic; floating-point values MUST NOT determine a
payment. The authorization MUST bind the content-derived compute-savings
evidence identifier.
The authorization MUST bind the exact `RevenueEventID`; settlement MUST consume
it atomically so the same authorization or revenue-event pool cannot be replayed.
