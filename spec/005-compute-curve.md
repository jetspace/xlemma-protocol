# XLIP-005 — Compute and verification curves

Providers publish signed offers containing service, model/checker, hardware, domain, delivery window, unit scale, price, capacity, completion probability, latency, collateral, asset, and expiration.

For XLMP node-network routing, `NodeServiceAdvertisement` is the canonical
provider-neutral market record and `ServiceOffer` is its compute-curve
projection. Discovery and matching MUST use the exact advertisement sequence,
checked integer price arithmetic, compatible assets/decimals/units, and the
constraints in XLIP-019. A quote or service match is not a payment receipt and
does not establish proof validity.

The protocol tracks separate curves for model generation, Lean build, official checking, independent checking, review, storage, and challenge reserve.

For expected quantities, a job quote is:

\[
Q_j(T)=\sum_r q_{j,r}F_r(T)+\pi_j^{risk}.
\]

Verified Proof Cost MUST adjust for probability of Gold verification and novelty clearance:

\[
VPC=\frac{generation+verification+review}{P_G P_N}.
\]

Compute reservations represent contractual future service, not stored compute inventory.

Routing SHOULD expose spot, economy, deadline, reserved, and competitive multi-provider modes and SHOULD penalize correlated providers.
