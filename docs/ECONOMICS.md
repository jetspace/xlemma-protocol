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

## Compute savings

\[
\Delta C_{k\to j}=LCB_\alpha[\widehat C_j^{(-k)}-C_j^{(+k)}].
\]

\[
D_{k\to j}=\min[\rho\max(0,\Delta C_{k\to j}),\kappa N_j].
\]

This creates an economic reward for open reusable lemmas without requiring exclusive ownership of the underlying truth.

## Anti-reflexivity rules

- Research credits cannot be backed by their own price.
- Credit spending cannot set verifier voting weight.
- A lemma's token price cannot count as protocol revenue.
- Related-party purchases are identified separately from arm's-length demand.
- Revenue routes use stable settlement accounting.
- Public profit-linked tokens are outside the V1 protocol core.
