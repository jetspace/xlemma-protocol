# XLIP-013 — Governance, challenges, and disputes

Governance MAY version policies, manage emergency security roles, fund public goods, and adjudicate process violations. It MUST NOT rewrite a checker observation or vote a failed proof into validity.

Challenges MUST identify a certificate, evidence root, bond, and objective challenge type. Successful objective challenges MAY receive a capped slashing reward. Abusive challenges MAY forfeit their bond.

Emergency quarantine MUST be append-only, preserve attribution and prior receipts, pause new economic activation, and provide a revalidation path.

Policy upgrades create new PolicyIDs. Historical certificates retain their original policy, checker, dependency, and software roots.

## Open discovery decision appeals

Certificate challenges and appeals of a rejected, grouped, or misallocated
contribution are distinct processes. An economic appeal need not allege an
invalid proof or identify an already issued certificate. The challenge bond
requirement above does not impose an uncapped bond on access to decision review.

Evidence and reward decisions MUST offer independent, reasoned, append-only
review with published deadlines, capped costs or funded assistance, conflict
exclusions, and remedies. Good-faith disagreement alone is not abuse. Economic
appeals preserve formal validity while holding affected allocations; a shared
allocation denominator makes all dependent allocations affected unless safe
isolation is demonstrated. Timeouts MUST NOT silently approve a claim or
convert missing review into scientific rejection. Formal appeals still require
reproduction and cannot override checker divergence by vote.

[XLIP-024](024-open-research-mining.md#verification-and-appeals) specifies the
full process and activation gates. An offline replay model exercises review
independence, costs, remedies and holds; it does not authenticate decisions or
operate an appeal service. See [the pilot guide](../docs/DISCOVERY_PILOT.md).
