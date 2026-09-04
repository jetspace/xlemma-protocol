# Research economics

## Research-credit conservation

For researcher `i`:

\[
A_i \ge B_i,
\]

where `A_i` is independently valued backing and `B_i` is outstanding Research Credit supply. Locked credits are a subset of outstanding credits and do not create new supply.

Usage settlement burns actual consumed credits and releases the same amount of backing. `upto` authorization locks a maximum but settles only actual usage.

## Net revenue

\[
N_j=G_j-C^{serve}_j-C^{compute}_j-C^{refund}_j-C^{reserve}_j.
\]

If deductions exceed gross revenue, no distribution occurs. Losses cannot be hidden through credit issuance.

## Creator allocation

The creator pool is split according to an append-only signed contribution manifest. The protocol supports multiple roles and does not assume the person who stated the conjecture also discovered, formalized, reviewed, and explained the proof.

## Auto-compounding

\[
R^{new}_{i,j}=\alpha_i s_iN_j,
\qquad
Cash_{i,j}=(1-\alpha_i)s_iN_j.
\]

Backing enters the Research Vault before or atomically with credit issuance.

## Evidence graph is not the economic graph

`FORMALLY_DEPENDS_ON` records actual use in a final proof. It does not mean
`OWES_PAYMENT_TO`, establish commercial causation, or let an upstream
participant block publication. Payment requires a separately signed economic
policy, an eligible economic edge, a settled external revenue event, and a
bounded pool.

`RevenueEventID` is content-derived from the exact claim, source,
related-party disclosure, settlement receipt, gross amount, deductions, time,
and evidence root. Changing any settlement fact creates a different event.
Related-party events remain auditable but are excluded from the reference
upstream allocator so wash activity cannot manufacture apparent external
demand.

Commons is the default: its mandatory upstream pool is zero. Reciprocal,
Commercial Artifact, and Sponsored Challenge may define a bounded upstream
pool before monetization. The constitution also declares a settlement-asset
minor-unit payout floor. Allocations below it stay in the explicit unallocated
remainder, preserving conservation without creating dust transfers.

## Market, commons, and assurance rails

Every settled `FundingReceipt` names one rail, one purpose, an external-value
evidence root, a settlement receipt, an economic policy, and a destination
vault. A purpose has exactly one rail:

- **market** funds bounties, formalization contracts, proof APIs, commercial
  licenses, certified implementations, reserved compute, and maintenance;
- **commons** funds foundational work, formal libraries, negative results,
  benchmarks, tactics, open data, exposition, and retrospective impact; and
- **assurance** funds verifier-bond, challenge, revalidation, warranty, and
  reliance-insurance reserves.

Protocol fees MUST conserve the settled amount and allocate a nonzero share to
both commons and assurance. A self-issued credit, token-price increase, or
unsettled promise is not funding.

## Compute-savings impact signal

\[
\Delta C_{k\to j}=LCB_\alpha[\widehat C_j^{(-k)}-C_j^{(+k)}].
\]

\[
D_{k\to j}=\min[\rho\max(0,\Delta C_{k\to j}),\kappa N_j].
\]

The estimate is an uncertain impact signal, not a counterfactual fact or
invoice. It can allocate only from the smaller of the conservative estimate,
the revenue cap, and a separately authorized impact-pool budget. The same
revenue event cannot be recursively charged. Authorization binds the exact
revenue event, and settlement must atomically consume the authorization and
remaining pool budget to prevent replay.
The evidence record is content-addressed and names its settlement asset and
decimals. Confidence multipliers use basis points; allocation math is checked
integer arithmetic with conservative upward rounding of uncertainty.

Impact policies should combine independently observed downstream use,
conservative compute contribution, adoption across independent operators,
maintenance/revalidation, and external economic use. They should apply
equivalence clustering, depth decay, concentration caps, and anti-collusion
review.

## Anti-reflexivity rules

- Research credits cannot be backed by their own price.
- Credit spending cannot set verifier voting weight.
- A lemma's token price cannot count as protocol revenue.
- Related-party purchases are identified separately from arm's-length demand.
- Revenue routes use stable settlement accounting.
- Public profit-linked tokens are outside the V1 protocol core.
- Compute markets price concrete services, not universal scientific worth.
