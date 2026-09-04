# XLIP-005 — Compute and verification curves

Providers publish signed offers containing service, model/checker, hardware,
domain, delivery window, concrete unit scale, price, capacity, latency,
collateral, asset, and expiration. Provider-advertised completion probability
MUST NOT control routing.

For XLMP node-network routing, `NodeServiceAdvertisement` is the canonical
provider-neutral market record and `ServiceOffer` is its compute-curve
projection. Discovery and matching MUST use the exact advertisement sequence,
checked integer price arithmetic, compatible assets/decimals/units, and the
constraints in XLIP-019. A quote or service match is not a payment receipt and
does not establish proof validity.

The protocol tracks separate curves for model generation, Lean build, official
checking, independent checking, review, storage, and challenge reserve. Units
MUST remain job-specific services such as reasoning tokens, attempts, seconds,
executions, hours, or byte-months; they MUST NOT be marketed as a universal
unit of research value.

For expected quantities, a job quote is:

\[
Q_j(T)=\sum_r q_{j,r}F_r(T)+\pi_j^{risk}.
\]

Quality-Adjusted Certification Cost MUST adjust for probability of Gold
verification and novelty clearance:

\[
E[C_{service}]=\sum_r \left\lceil\frac{C_r\,10{,}000}{p_{r,bps}}\right\rceil,
\qquad
QAC=\left\lceil\frac{E[C_{service}](10{,}000+\pi_{bps})10^8}
{10{,}000\,P_{G,bps}P_{N,bps}}\right\rceil.
\]

All monetary and probability calculations on this path MUST use checked
integer arithmetic. Probabilities are encoded in basis points. The reference
rounds expected costs upward and uses deterministic offer-ID ordering to break
equal adjusted-cost ties; floating-point values MUST NOT determine a payment.

`P_G`, `P_N`, and service completion estimates MUST come from signed,
time-bounded protocol calibration records backed by audited outcome history,
not from the provider whose offer is being ranked. Estimates SHOULD stratify by
domain, task complexity, libraries, model/checker snapshot, time budget, human
assistance, selection bias, and abandoned-job censoring. Deployments MUST
authorize estimator signing keys under the referenced calibration policy; a
self-certifying signature proves key control, not that the estimator is trusted.

Compute reservations represent contractual future service, not stored compute
inventory. The launch sequence is spot quotes, usage-capped jobs, reserved
capacity, service-level forwards, and capacity options. Tradeable futures are a
deferred layer requiring standardized service profiles and settlement history.

Routing SHOULD expose spot, economy, deadline, reserved, and competitive multi-provider modes and SHOULD penalize correlated providers.
