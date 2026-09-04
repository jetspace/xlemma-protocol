# XLIP-005 — Compute and verification curves

Providers publish signed offers containing service, model/checker, hardware, domain, delivery window, unit scale, price, capacity, completion probability, latency, collateral, asset, and expiration.

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
