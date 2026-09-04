# XLIP-021 — Statement alignment, rights modes, and impact funding

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative. This
specification defines protocol semantics, not legal advice or a representation
that any particular right is enforceable in a jurisdiction.

## 1. Five conservation laws

1. **Truth conservation.** Stake, token balances, votes, popularity, and
   reputation MUST NOT establish formal validity. A formal status describes
   exact checking of a formal claim under pinned definitions, imports, axioms,
   proof objects, and checker policy.
2. **Value conservation.** Every withdrawable reward MUST trace to settled
   value from an external payer. Token issuance, expected revenue, and asset
   appreciation MUST NOT be represented as value creation.
3. **Rights conservation.** Capsule registration MUST NOT create copyright,
   patent rights, ownership of mathematical truth, or rights not controlled by
   the signers. It MAY record priority, attribution, artifacts, contracts, and
   economic participation actually supported by evidence.
4. **Independence conservation.** Multiple machines under common beneficial
   control count as one independence domain. Credential issuance and
   revocation SHOULD use multiple qualified issuers. Operator independence is
   an evidence-backed confidence assessment, not mathematical certainty.
5. **Causality conservation.** A formal dependency proves use in a formal
   artifact; it does not prove commercial causation or create a debt.

These rules refine, and do not replace, the invariants in XLIP-000.

## 2. Formal validity and statement alignment

A formal certificate means that an exact formal statement follows under its
declared theory and evidence policy. It does not mean that the formal statement
faithfully represents an informal claim, scientific model, empirical
interpretation, or commercial assertion.

XLMP defines `StatementAlignmentReceipt` as a separate human/domain-review
record containing:

- the exact `ClaimID`;
- hashes of the informal claim and LaTeX presentation;
- disclosed assumptions and reviewed definitions;
- identified, credential-referenced domain reviewers and conflicts;
- an `aligned`, `partially_aligned`, `misaligned`, or `inconclusive` verdict;
- limitations, evidence root, review time, and signatures.

Its `ReceiptID` MUST be content-derived. Reviewer signatures MUST cover the
same receipt identity. Reviewers MUST be unique under the applicable
independence policy. An interface MUST display formal status and alignment
status separately and MUST NOT collapse them into one undifferentiated
“verified” badge.

## 3. Three distinct rights objects

XLMP distinguishes:

- **origin/provenance:** nontransferable, append-only evidence that a researcher
  committed a claim or artifact at a time and ordering reference;
- **artifact and legal rights:** rights in manuscripts, code, diagrams,
  datasets, experimental records, commercial implementations, eligible patent
  interests, or signed contracts, only to the extent actually controlled;
- **economic participation:** a contractual or policy entitlement to a defined
  revenue source.

Economic participation terms MUST identify the revenue source, payer,
calculation base, exclusions, share, duration, payout cap, transfer rules,
economic policy root, and dispute process. A token handle MUST NOT replace
these terms or imply ownership of future royalties from a theorem.

## 4. Evidence graph and economic graph

The evidence graph is descriptive. Edges such as `FORMALLY_DEPENDS_ON`,
`EXTENDS`, `USES_DATASET`, and `USES_LIBRARY` record what the final certified
artifact used.

The economic graph is prescriptive. Edges such as `PAYS_TO`,
`CONTRIBUTES_TO_IMPACT_POOL`, and `ALLOCATES_BOUNTY_TO` exist only under an
explicit policy, license, sponsorship, or other agreement.

The protocol invariant is:

```text
FORMALLY_DEPENDS_ON != OWES_PAYMENT_TO
```

An upstream allocation requires all of:

```text
qualifying settled revenue
AND active economic policy
AND eligible economic edge
AND final-artifact use where the policy requires it
AND an unspent bounded pool
AND non-recursive treatment of the revenue event
```

No upstream participant may block publication or use merely because its claim
appears in the evidence graph. Generic runtime or language dependencies SHOULD
be excluded from commercial allocation. Equivalent claims SHOULD be clustered,
depth decay and concentration caps SHOULD apply, and every revenue event MUST
be charged at most once.

## 5. Capsule economic modes

Every `LemmaCapsule` MUST select exactly one mode:

### Commons

- public formal artifacts are usable without mandatory per-use protocol fees;
- attribution remains visible;
- grants, donations, sponsorship, and capped impact-pool allocations MAY fund
  continued work;
- a license in this mode MUST NOT contain hidden revenue-participation terms.

This is the default mode for foundational and open research.

### Reciprocal

- qualifying monetized xLemma descendants contribute one bounded upstream
  pool;
- bare citations and independently created external claims do not qualify;
- depth decay, ancestor caps, equivalent-claim clustering, and one charge per
  revenue event prevent an anticommons;
- upstream participants receive no veto over publication, use, or licensing.

### Commercial Artifact

- the formal claim MAY remain public;
- controlled artifacts or services MAY be licensed under explicit scope;
- any upstream pool MUST be fixed before monetization, bounded, non-recursive,
  non-blocking, and supported by the economic graph.

### Sponsored Challenge

- the sponsor escrows or otherwise proves the reward source;
- acceptance, contribution allocation, upstream allocation, result rights,
  deadline, and dispute procedure are declared before work begins;
- payout requires the exact final artifact and certificate specified by the
  challenge.

## 6. Impact signals and compute pricing

Compute savings are an uncertain counterfactual signal, not a precise invoice.
They MAY contribute to a capped impact score alongside independently observed
reuse, adoption across independent operators, maintenance/revalidation, and
external economic use. They MUST NOT create an allocation without an
`ImpactPoolAuthorization` that binds settled revenue, the economic policy,
exact revenue event, eligible edge, pool budget, and non-recursion rule. The
authorization MUST have
a content-derived identifier and a cryptographic signature from an authorized
economic-policy principal. Deployments MUST resolve and authorize that signer
under the referenced policy; key possession alone does not grant spending
authority. Settlement MUST atomically consume the authorization and its
revenue-event pool budget; the calculation object alone does not settle funds.

Compute is priced as heterogeneous, time-bound service. XLMP MUST use
job-specific units such as reasoning tokens, proof-search attempts, Lean build
seconds, checker executions, expert-review hours, or byte-months; it MUST NOT
represent them as a universal unit of scientific worth.

Provider-advertised success rates MUST NOT control routing. Quality-adjusted
certification cost MUST use signed, time-bounded protocol estimates backed by
audited outcome history. Wire probabilities and all monetary calculations MUST
use checked fixed-point integer arithmetic. Initial forward markets SHOULD progress from spot
quotes to usage caps, reserved capacity, service-level forwards, and capacity
options. Tradeable futures remain outside the core until service profiles and
settlement history are sufficiently standardized.

## 7. Launch profile

The recommended first market is a sponsor-backed marketplace for
ASTRA-assisted Lean formalization and independently reproduced certification
in one domain with identifiable external buyers. Suitable profiles include
cryptographic protocols, smart-contract properties, verified algorithms, and
selected optimization results.

Per-lemma speculative tokens, researcher profit tokens, universal mandatory
royalties, universal research-value units, tradeable compute futures, and
token-weighted research governance are outside the core launch profile.
