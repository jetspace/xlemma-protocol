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
inventory. `ComputeProcurementInstrument` encodes the launch sequence as spot
quotes, maximum-cost authorizations, reserved capacity, domain-specific service
forwards, and nontransferable capacity options. A reservation, forward, or
option MUST bind a service profile, delivery interval, counterparty, maximum
spend, settlement policy, and collateral or reservation root. An option MUST
expire before delivery begins. XLMP/1 rejects transferable instruments;
tradeable derivatives are deferred until services have genuine
standardization, liquidity, and legally reviewed settlement history.

The protocol MUST remain useful under three compute regimes:

1. **scarce frontier compute**, where procurement pools, privacy, reservations,
   spending caps, concentration limits, and fallbacks protect access;
2. **cheap abundant compute**, where economics shifts toward question quality,
   novelty, identity, curation, empirical grounding, and maintenance rather
   than relying on persistent verification margins; and
3. **heterogeneous agent economies**, where agents read exact certificate,
   rights, dependency, and economic-policy records before composing or paying
   for a research object.

Canonical service names describe functions, not vendors. In particular,
`research_prover_generation` may be fulfilled by ASTRA, an open model, a local
model, or another adapter. Historical inputs using `astra_generation` may be
accepted only as a compatibility alias and MUST serialize canonically as the
provider-neutral service.

Routing SHOULD expose spot, economy, deadline, reserved, and competitive
multi-provider modes. A diversified route MUST enforce caps by independent
provider cluster, require multiple model families and regions, include a
fallback, honor confidential-delivery requirements, and remain under its
maximum spend. Rotating offer IDs does not create diversity.
