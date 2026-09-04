# xLemma governance constitution

This constitution defines powers that protocol governance may exercise and boundaries it must not cross.

## 1. Constitutional principles

1. Mathematical validity is established by deterministic verification under a declared theory, not by token-weighted governance.
2. Governance cannot convert `DIVERGENT`, `ERROR`, or missing evidence into `CERTIFIED`.
3. ASTRA and other proof producers cannot serve as their own independent final verifier.
4. Research-credit supply cannot exceed independently valued backing.
5. Attribution history, dissent, corrections, revocations, and supersession are append-only.
6. Rights manifests cannot create rights that contributors did not possess.
7. Verifier compensation is based on reproducible work, not agreement with a desired result.
8. Emergency actions are fail-closed, scoped, reviewable, and time-limited.
9. Protocol revenue cannot be manufactured from unrealized token valuation.
10. Public profit-linked instruments remain outside the core until separately reviewed and legally wrapped.
11. No node may contribute to xLemma consensus without a valid, non-revoked OperatorCredential ultimately controlled by a verified xLemma participant; multiple nodes under the same participant form one operator-independence domain.
12. Formal validity and statement alignment are separate evidence classes; governance cannot infer either from the other.
13. A formal dependency cannot create a payment obligation, compulsory license, or publication veto without an explicit economic agreement.
14. Open Commons artifacts carry no mandatory per-use protocol fee.
15. Compute markets price defined services, not universal scientific value; provider success claims do not control routing.

These principles should be encoded as immutable contract constraints where practical and as social/governance constraints where not.

## 2. Governance domains

Governance may manage:

- supported chains and settlement assets;
- approved contract implementations;
- minimum neutral collateral;
- challenge periods;
- checker-family qualification criteria;
- accepted credential issuers, credential tiers, revocation profiles, and role qualifications;
- permitted trust-policy templates;
- treasury and public-goods budgets;
- software release processes;
- security councils and incident procedures;
- registry metadata and deprecation schedules.

Governance may not decide:

- that a failed proof is valid;
- that one formal claim is equivalent to another without a proof object;
- that an originator loses historical attribution because a token was transferred;
- that a vault is solvent when backing is insufficient;
- that a payment receipt substitutes for a verification receipt;
- that a novelty vote changes the result of formal checking.
- that additional NodeIDs under one verified participant create additional independence.
- that a formal dependency creates an economic debt or that an economic payment proves scientific importance.
- that a token or capsule registration creates rights in mathematical truth.

## 3. Roles

### Protocol maintainers

Maintain reference software, specifications, schemas, and release artifacts. They do not have unilateral authority over independently deployed networks.

### Security council

May pause vulnerable adapters, quarantine affected certificate classes, freeze new economic actions, and publish emergency policy roots. It cannot certify a theorem.

### Policy registry stewards

Publish versioned policy templates. Existing certificates remain bound to the policy version under which they were issued.

### Node qualification councils

Maintain qualification criteria and evidence, not case-by-case theorem outcomes. Selection remains randomized among eligible operators.

### Treasury stewards

Fund audits, public goods, negative results, storage, and incident reserves. Treasury allocations must be disclosed and should not weight formal consensus.

### Researchers and contributors

Control their manifests, rights assertions, revenue routes, and correction proposals subject to cryptographic and legal constraints.

### Watchers and challengers

Submit counterevidence and monitor execution, availability, solvency, conflicts, and implementation vulnerabilities.

## 4. Proposal classes

| Class | Examples | Minimum process |
|---|---|---|
| Editorial | wording, non-normative examples | maintainer review |
| Compatible protocol | optional fields, adapters | public review + conformance tests |
| Consensus policy | checker families, quorum diversity | extended review + simulation + staged activation |
| Economic policy | fees, pools, collateral | audit + scenario analysis + delayed activation |
| Contract migration | vault/registry changes | independent audit + user exit path |
| Constitutional | core invariants | supermajority across stakeholder chambers; cannot override formal evidence |
| Emergency | active exploit/checker compromise | scoped pause, public evidence, expiry, retrospective review |

## 5. Multi-chamber governance

A mature network should avoid a single fungible-token electorate. Material changes should require concurrence from independent chambers such as:

- verified researcher identities;
- qualified node operators, capped independently per VerifiedUserID, OperatorID, and conservative OperatorClusterID;
- maintainers and security experts;
- public-goods or user representatives;
- optional capital providers, without unilateral epistemic power.

A chamber vote can approve policies. It still cannot change the observed output of a checker run.

## 6. Upgrade rules

1. Every policy and implementation has a content-addressed version.
2. Certificates bind exact policy and binary/environment digests.
3. Upgrades apply prospectively unless a certificate is explicitly revalidated.
4. Users receive a migration and exit window for economic contracts.
5. Historical records remain readable after deprecation.
6. Emergency upgrades expire unless ratified through the normal process.
7. Upgrade keys should be threshold-controlled, hardware-protected, geographically distributed, and eventually minimized or removed from immutable core registries.

## 7. Dispute classes

- formal checker divergence;
- compromised checker implementation or environment;
- provenance or contribution dispute;
- novelty/prior-art dispute;
- rights or license dispute;
- payment or metering dispute;
- vault solvency dispute;
- storage availability failure;
- operator-cluster or conflict-of-interest concealment;
- statement-alignment dispute;
- compute-impact measurement or economic-edge dispute.

Each class has a distinct evidence type and remedy. A rights dispute may pause commercial revenue without changing formal validity. A checker compromise may quarantine validity without erasing authorship.

## 8. Emergency powers

Permitted emergency actions:

- pause new authorizations or minting;
- quarantine affected certificates;
- disable a compromised checker digest;
- stop a vulnerable facilitator or adapter;
- increase storage replication;
- extend challenge windows;
- preserve evidence and publish incident roots.

Forbidden emergency actions:

- rewrite old receipts;
- seize attribution;
- finalize a failed or divergent proof;
- mint unbacked credits;
- delete dissent or challenge evidence;
- transfer researcher rights without the governing agreement.

## 9. Fork and exit rights

Specifications, identifiers, and proof artifacts should be portable. Researchers and nodes must be able to:

- verify artifacts locally;
- export manifests and receipts;
- deploy compatible registries;
- move future service activity to another facilitator or chain;
- preserve content-addressed identity across forks;
- redeem or exit economic positions according to the governing vault terms.

Protocol legitimacy should come from reproducibility and credible exit, not dependency on one operator.
