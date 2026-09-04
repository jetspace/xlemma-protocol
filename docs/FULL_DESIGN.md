# xLemma XLMP/1 — Complete Protocol Design

## Executive definition

xLemma is an open decentralized protocol for **proof-carrying research**. Its primary user is a sovereign decentralized researcher—an individual, pseudonymous contributor, laboratory, or collective—who wants to identify, finance, produce, independently reproduce, certify, attribute, publish, license, reuse, and economically reward verifiable research objects. ASTRA and Lean are the reference prover and verifier adapters, not protocol dependencies.

The protocol's research flywheel is:

\[
\text{funding}
\rightarrow
\text{prover-adapter compute}
\rightarrow
\text{formal proof candidate}
\rightarrow
\text{independent reproduction}
\rightarrow
\text{certified capsule}
\rightarrow
\text{paid use}
\rightarrow
\text{settled revenue}
\rightarrow
\text{new backed research credits}
\rightarrow
\text{more research}.
\]

The protocol does **not** attempt to privatize mathematical truth. It creates a programmable market around the work required to discover, formalize, explain, verify, maintain, apply, and distribute research.

---

## 1. Target user: the Decentralized Researcher Node

A `DecentralizedResearcherNode` is the protocol's first-class principal:

```text
DecentralizedResearcherNode
├── ResearcherID
├── optional VerifiedUserID link
├── identity and signing keys
├── Research Credit Rᵢ
├── Research Vault Vᵢ
├── contribution identity
├── reputation and calibration record
├── Lemma Capsules[]
├── compute reservations[]
├── revenue accounts[]
├── licenses[]
└── governance and recovery policy
```

A researcher may remain pseudonymous while still having a stable cryptographic identity. Legal rights, regulated activity, commercial licensing, or fiat access may require additional identity at specific boundaries, but public theorem participation does not require one universal identity regime. Consensus node operation does require the privacy-preserving `VerifiedUserID → OperatorID → NodeID` credential chain defined by XLIP-020; issuer-retained legal or uniqueness evidence is not published on protocol.

One researcher or laboratory has one primary research-credit economy and many proof capsules. A separate freely traded token for every lemma is not the default because it would fragment liquidity, reward spam, make dependency accounting unmanageable, and turn scientific objects into speculative micro-assets.

---

## 2. The separations that protect the system

The system must preserve the following distinctions:

\[
\text{formal validity}
\neq
\text{authorship}
\neq
\text{legal rights}
\neq
\text{token ownership}
\neq
\text{license rights}
\neq
\text{novelty}
\neq
\text{significance}
\neq
\text{statement alignment}
\neq
\text{LaTeX interpretation}
\neq
\text{payment settlement}.
\]

A Lean proof can be valid but unoriginal. A result can be novel but trivial. A
formally valid statement can be vacuous, weaker than advertised, or misaligned
with its informal interpretation. A token can point to a proof without
conveying any copyright or patent right. A payment can settle for a failed
verification job. A researcher can establish priority without proving
exclusive legal ownership.

### 2.1 What Lean establishes

Under a pinned theory, dependency graph, trust policy, toolchain, exact claim, and exact proof object, Lean establishes whether the proof checks. Nodes reproduce that fact; they do not vote it into existence.

### 2.2 What the chain establishes

The economic ledger establishes ordering, escrow, settlement, challenge deadlines, certificate anchoring, credit issuance, revenue routing, and optional token state. It does not establish mathematical validity by itself.

### 2.3 What reviewers establish

Novelty, usefulness, interpretation, significance, and prior-art coverage are evidence-weighted judgments. These judgments must remain separately labeled, calibrated, challengeable, and reversible when new evidence appears.

### 2.4 What statement alignment establishes

Domain reviewers may attest that a specific informal claim and LaTeX
presentation correspond to the exact formal ClaimID, with disclosed
assumptions, reviewed definitions, conflicts, and limitations. This
`StatementAlignmentReceipt` remains separate from formal certification.

---

## 3. Modular architecture

| Module | Responsibility | Does not do |
|---|---|---|
| `xlemma-core` | IDs, manifests, capsules, receipts, append-only graph | Run models or chains |
| `xlemma-xlmp` | Canonical XLMP/1 messages, lifecycle, and provider-neutral adapter contracts | Execute a provider, checker, payment, chain, transport, or store |
| `xlemma-astra` | Reference `ResearchProver` adapter for formalization, proof search, repair, explanation | Define XLMP or certify its own output |
| `xlemma-lean` | Default `VerifierAdapter` for pinned builds, proof export, axiom inventory, checker replay | Define XLMP, assess novelty, or determine rights |
| `xlemma-consensus` | PoIR, generalized quorums, commit-reveal, divergence | Token-weighted truth voting |
| `xlemma-x402` | Optional x402 payment and paid-HTTP adapter | Define research state or consensus |
| `xlemma-economics` | Backed credits, revenue conservation, dividends | Create unbacked value |
| `xlemma-compute-curve` | Spot/forward service offers and proof-cost estimates | Treat compute as storable inventory |
| `xlemma-storage` | `StorageAdapter` implementation, content-addressed bundles, availability receipts | Decide truth or ownership |
| Registry contracts | `FinalityAdapter` anchors, escrow value, revenue routing, optional rights tokens | Interpret formal statements or define XLMP |
| Presentation layer | LaTeX, web, citations, explanations | Override the formal claim |

This dependency direction is strict: adapters depend on XLMP types; XLMP never depends on ASTRA, Lean, x402, a chain, or a storage network. It allows xLemma to use an existing blockchain rather than creating a bespoke consensus chain at launch.

---

## 4. Immutable research object graph

```text
TheoryID
  └── ClaimID
        ├── ProofID A
        │     └── ArtifactID A
        │           ├── ASTRA Generation Receipt
        │           ├── Lean Build Receipt
        │           ├── Checker Observations
        │           └── Formal Consensus Certificate
        ├── ProofID B
        ├── PresentationID A
        ├── PresentationID B
        ├── Rights Manifest
        ├── Contribution Manifest
        ├── Novelty Receipts
        ├── Statement Alignment Receipts
        ├── Availability Receipts
        └── Revenue Route
```

Nothing is silently modified. Corrections create new nodes and explicit edges such as:

```text
SUPERSEDES
AMENDS
REVOKES
EQUIVALENT_TO_BY_PROOF
DEPENDS_ON
PRESENTS
LICENSES
CHALLENGES
REVALIDATES
```

### 4.1 Identifier construction

A conceptual construction is:

```text
TheoryID = H(
  protocol version
  || Lean toolchain
  || dependency Merkle root
  || trust policy
  || checker policy
  || canonical encoding version
)

ClaimID = H(
  "xlemma-claim-v1"
  || TheoryID
  || canonical elaborated Lean type
)

ProofID = H(
  "xlemma-proof-v1"
  || ClaimID
  || canonical Lean proof object
)

ArtifactID = H(
  sorted file manifest
  || toolchain
  || lockfile
  || build metadata
)

ReceiptID = H(
  receipt kind
  || exact bound identifiers
  || observation
  || signer
  || policy
)
```

Domain separation prevents the same bytes from becoming interchangeable claim, proof, theory, artifact, or receipt IDs.

### 4.2 Claim identity uses elaborated Lean expressions

A claim is not identified by filename, title, whitespace, binder spelling, notation, or LaTeX prose. The canonical exporter must resolve constants and namespaces, include universes and implicit arguments, alpha-normalize local binder names, preserve type information, bind the expression to its theory environment, and use a versioned deterministic encoding.

### 4.3 Equivalence is itself a proof object

Language-model similarity, text embeddings, or reviewer opinions may propose duplicate clusters, but the protocol must not automatically merge claims. A formal equivalence edge requires a theorem such as:

```lean
theorem claim_A_equiv_claim_B : ClaimA ↔ ClaimB := by
  ...
```

For non-propositional objects, the equivalence relation must be explicitly defined and proved.

---

## 5. The Proof Rights Capsule

Each result receives a capsule that binds formal, human, economic, and rights records without conflating them:

```text
ProofRightsCapsule
├── LemmaID
├── TheoryID
├── ClaimID
├── ProofID(s)
├── Lean artifact
├── LaTeX presentation(s)
├── Origin Certificate
├── Contribution Graph
├── Machine-Contribution Record
├── Rights Manifest
├── Compute Ledger
├── Verification Receipts
├── Novelty and significance evidence
├── Statement Alignment Receipts
├── Evidence/dependency graph
├── Economic mode and economic-policy graph
├── Revenue Route
└── optional token handle
```

### 5.1 Origin and attribution

The Origin Certificate is non-transferable. It records a signed priority claim and an ordering reference. It does not prove that no earlier work exists and does not erase competing priority claims.

### 5.2 Contribution graph

The contribution manifest can distinguish:

- formula or conjecture originator;
- proof discoverer;
- Lean formalizer;
- tactic and library author;
- dataset or experimental contributor;
- expert reviewer;
- exposition author;
- sponsor;
- compute provider.

Contributor shares are signed and must sum to 10,000 basis points within the creator pool. Amendments are append-only and retain the prior record.

### 5.3 Machine contribution record

ASTRA or another model receives an explicit machine-contribution record containing provider, model, request hash, context root, output roots, and the human selection/edit record. This prevents AI assistance from being hidden or confused with human contribution.

### 5.4 Rights manifest

The rights manifest states what the contributor actually claims to control:

- attribution;
- manuscript expression;
- source code;
- dataset rights;
- patent or application rights;
- trade secrets;
- contractual licenses;
- commercial implementation rights;
- access rights;
- or no exclusive right in the underlying mathematical proposition.

It must include employment, university, grant, collaborator, and sponsor clearance. A token cannot create rights the minter never owned.

The capsule keeps three objects distinct: nontransferable origin/provenance;
rights actually held in artifacts or contracts; and economic participation in
a defined revenue source. The last requires payer, calculation base,
exclusions, duration, share, cap, transfer rule, and dispute procedure.

### 5.5 Economic modes

- **Open Commons** permits public reuse without mandatory per-use protocol
  fees and may receive impact-pool, grant, donation, or sponsor funding.
- **Commercial Research** licenses controlled artifacts or services with
  explicit bounded terms.
- **Sponsored Challenge** fixes funded acceptance, contribution/upstream
  allocation, result rights, and disputes before work.

Formal dependency edges remain descriptive evidence. Only a separate economic
policy graph can authorize payment.

---

## 6. Research Credit and Research Vault

The decentralized researcher pays for their own work with a researcher-specific token, but that token must not create circular validator economics.

### 6.1 Recommended V1 asset

`Rᵢ` is a fully backed, restricted-transfer service credit:

```text
stable settlement assets in Research Vault Vᵢ ≥ outstanding Rᵢ
```

A one-to-one initial unit makes accounting transparent:

```text
1,000 USDC deposited
  → at most 1,000 Rᵢ minted

125 Rᵢ authorized for a proof job
40 Rᵢ actually consumed
  → 40 Rᵢ burned
  → 40 USDC paid to independent nodes
  → 85 Rᵢ authorization returned
```

### 6.2 Permitted backing sources

New credits may be issued only against:

1. stable assets deposited into the vault;
2. settled external research revenue;
3. grants or bounty funds;
4. prepaid compute capacity valued with a conservative haircut;
5. funded protocol research awards.

They may not be issued against:

- the token's own market price;
- an unverified lemma;
- future expected profits;
- another circularly issued researcher token;
- unverifiable impact claims.

### 6.3 Early researchers without capital

They can receive grants, public-goods allocations, sponsor-funded bounties, donated compute credits, fellowships, or voluntary node sponsorship. Independent nodes are never forced to accept an unbacked personal token.

### 6.4 The researcher's token cannot affect consensus

`Rᵢ` may pay for execution, higher assurance, storage, challenges, or revalidation. It may not weight formal votes, select friendly verifiers, suppress dissent, alter node reputation, or shorten mandatory challenges.

Node bonds and payouts use a neutral independently valuable asset.

---

## 7. Adding profits without manufacturing value

Protocol profit means settled net external research revenue:

\[
N_j = G_j - C^{serve}_j - C^{compute}_j - C^{refund}_j - C^{reserve}_j.
\]

Only `N_j` is distributable.

For researcher share `s_i N_j` and auto-compound rate `α_i`:

\[
\text{credits added}_i = \alpha_i s_i N_j,
\]

\[
\text{cash payout}_i = (1-\alpha_i)s_i N_j.
\]

If a lemma produces 1,000 USDC of net revenue, the researcher receives 650 USDC from a 65% creator pool, and chooses 60% compounding:

```text
390 USDC remains in Vᵢ and backs 390 newly minted Rᵢ
260 USDC is paid out as cash
```

An increase in the quoted market price of `Rᵢ` is not revenue.

### 7.1 Illustrative net-revenue waterfall

| Destination | Share |
|---|---:|
| Creator and formalizer pool | 65% |
| Mandatory upstream lemma/library pool | 0% |
| Reverification and security | 8% |
| Open research, impact, and unsuccessful-work fund | 17% |
| Dispute and insurance reserve | 5% |
| Protocol operations | 5% |

This is configurable but must always conserve 100% of net distributable revenue.

---

## 8. ASTRA proof-production layer

ASTRA is the primary configured proof-production model, but the interface is provider-neutral.

### 8.1 ASTRA functions

ASTRA may:

- translate prose and LaTeX into candidate Lean statements;
- identify ambiguity and hidden assumptions;
- decompose the theorem into subgoals;
- search for proof strategies;
- retrieve applicable libraries;
- generate Lean source;
- repair compiler failures iteratively;
- propose equivalence proofs;
- generate human-readable LaTeX explanations;
- estimate proof-search uncertainty;
- produce compute and generation receipts.

### 8.2 ASTRA cannot certify ASTRA

Every ASTRA output is an untrusted candidate until independent Lean/checker reproduction. Its receipt proves what model request and usage were recorded, not that the mathematics is valid.

### 8.3 ASTRA generation receipt

```text
model and optional snapshot
reasoning setting
request hash
context root
input, cached input, output and tool usage
wall-clock time and retries
actual charge
candidate roots
time and node signature
```

The adapter is configuration-driven so changes to model availability or model names do not alter protocol identities.

---

## 9. Lean and LaTeX bridge

### 9.1 Lean integration

A declaration may be marked:

```lean
@[xlemma]
theorem noArbitrage ... := by
  ...

#xlemma_export noArbitrage
#print axioms noArbitrage
```

The production exporter must extract:

- elaborated theorem type;
- serialized proof object;
- universe parameters;
- direct formal dependencies;
- transitive dependency root;
- observed axiom inventory;
- exact toolchain and lockfile;
- source and build artifact roots.

### 9.2 LaTeX integration

```latex
\begin{lemma}[No-arbitrage condition]
  \lean{Finance.NoArbitrage.noArbitrage}
  \leanok
  \uses{def:market,def:no-free-lunch}
  \xtheoryid{xlt:...}
  \xclaimid{xlc:...}
  \xproofid{xlp:...}
  \xverification{xlemma-gold-v1}
  \xlicense{Apache-2.0}
  ...
\end{lemma}
```

Changing only the exposition creates a new `PresentationID`; changing the elaborated Lean statement creates a new `ClaimID`.

A declaration link does not prove that the prose correctly explains it. The formal type, definitions, assumptions, notation, type classes, dependencies, and axiom policy must remain inspectable.

---

## 10. Verification ladder

| Level | Meaning |
|---|---|
| Draft | Content-addressed and signed, not checked |
| Local | Researcher or ASTRA runner built it locally |
| Kernel | One pinned Lean environment accepts it |
| Audited | Axiom report, placeholder controls, fresh checker replay |
| Independent | Unrelated operator reproduces it |
| Gold | Exact trusted challenge, sandboxed build, official kernel and independent checker families |
| Research Certified | Gold plus novelty/interpretation review |
| Economically Final | Challenge period closed and revenue route activated |
| Mature | Reused, periodically revalidated, and uncontested |
| Succinct | Gold plus a cryptographic proof of checker execution |
| Quarantined | Contradictory evidence or checker compromise exists |

A single `verified: true` flag is forbidden because it erases trust assumptions.

---

## 11. Node consensus: Proof of Independent Reproduction

### 11.1 Core rule

\[
\boxed{
\text{research consensus}
=
\text{independent reproduction}
+
\text{evidence aggregation}
+
\text{economic finality}
}
\]

Not:

\[
\text{valid theorem} = \text{majority token vote}.
\]

### 11.2 Observation receipt

Each selected node commits and later reveals a receipt binding:

```text
ClaimID
ProofID
TheoryID
ArtifactRoot
DependencyRoot
AxiomSetRoot
Checker implementation and binary digest
Environment digest
Verdict
Execution trace root
NodeID
OperatorClusterID
Timestamp
Signature
```

### 11.3 Five consensus planes

| Plane | Question | Rule |
|---|---|---|
| Formal validity | Did this exact proof establish this exact claim? | Exact deterministic reproduction |
| Provenance | Who committed and contributed what? | Signatures and chronological ordering |
| Novelty | Is it already known or materially different? | Evidence-weighted calibrated review |
| Economic state | Did payment and routing settle? | Base-chain or BFT state-machine finality |
| Availability | Can artifacts be retrieved? | Replicated custody attestations |

Significance is tracked as a sixth observational plane, transitioning from predictions to measured reuse.

### 11.4 Generalized quorum

A Gold policy can require:

```text
2 official-kernel executions
AND 1 independently implemented checker
AND 3 operator clusters
AND 3 OperatorIDs
AND 3 verified participants or organizations
AND valid, fresh, non-revoked V2-or-higher credential chains
AND 2 infrastructure providers
AND 2 geographic regions
AND identical artifact/environment/dependency/axiom roots
AND no unresolved challenge
```

A hundred public keys operated by one participant count as one independence domain, even if presented through multiple OperatorIDs or operator clusters.

### 11.5 Divergence is not a vote

```text
Lean kernel A: PASS
Lean kernel B: PASS
independent checker C: FAIL
```

Result:

```text
DIVERGENT → QUARANTINED → expanded reproduction → implementation investigation
```

Never `2–1 PASS`.

---

## 12. First-class node network and separation of duties

### 12.1 Researcher nodes

Create claims, choose policies and budgets, commit artifacts, pay for services, receive revenue, and run local checks. They cannot independently finalize their own proof.

### 12.2 Research-prover nodes

Perform model-assisted formalization, search, repair, and explanation. ASTRA is the reference implementation, not a protocol dependency. A prover's accepted-proof bonus is separate from formal verification.

### 12.3 Lean build nodes

Reconstruct the pinned project, compile candidates, export proof objects, enumerate axioms, and create deterministic build artifacts.

### 12.4 Checker nodes

Replay the exact proof with official Lean and independent checker implementations. They are paid for valid execution whether the verdict is pass or fail.

### 12.5 Novelty and interpretation nodes

Search prior art, compare claims, assess material difference, inspect whether the exposition overstates the formal theorem, and publish evidence-backed probabilities.

### 12.6 Watchers and challengers

Continuously reproduce certificates, monitor checker vulnerabilities, challenge false receipts, detect collusion or hidden common control, and trigger quarantine or revalidation.

### 12.7 Storage and index nodes

Replicate bundles, answer proof-of-custody challenges, index theorem and dependency graphs, and preserve historical versions.

### 12.8 Payment facilitators

Verify and settle payment payloads. They do not participate in proof consensus.

### 12.9 Incompatible same-job roles

```text
ASTRA prover ≠ final verifier
researcher ≠ independent verifier
payment facilitator ≠ proof validator
originator ≠ sole novelty reviewer
challenged node ≠ dispute adjudicator
```

### 12.10 Discovery, order book, and immutable matches

A node publishes a signed, expiring `NodeServiceAdvertisement` that binds its
NodeID, OperatorID, conservative OperatorClusterID, credential IDs and chain root, roles, XLMP endpoints,
implementations/checker families, theories and domains, hardware, capacity,
latency, price, reputation snapshot, eligibility bond, terms root, and
validity interval. Price or capacity changes create a new content-derived
AdvertisementID and monotonically increasing sequence; prior advertisements
remain available as historical evidence.

`NodeDiscoveryRequest` states service, role, checker, theory, domain, capacity,
latency, price, reputation, and independence constraints. A deterministic
result commits to the ordered AdvertisementID set. `ServiceOrder` expresses
demand and `ServiceMatch` reserves one exact advertisement sequence. Discovery,
matching, payment settlement, execution, and certification are different
protocol actions with different receipts.

### 12.11 Multidimensional reputation and bonds

Reputation has eight separately evidenced dimensions: formal accuracy,
availability, latency, novelty calibration, challenge quality, and operator
independence, storage quality, and integrity. Policies gate each relevant dimension by minimum score and sample
size. XLMP defines no universal weighted score, so strength in one dimension
cannot conceal failure in another.

A `NodeBond` binds independently valued collateral to a node, OperatorID, operator cluster,
eligible roles, slashing policy, escrow evidence, and lock period. Bond size
above the minimum does not increase sortition weight or mathematical authority.

---

## 13. Committee selection and independence

### 13.1 Eligibility and sortition

An issuer-verified, non-revoked participant/operator/node credential chain,
active collateral, role capability, checker family, software identity, conflict
disclosures, and every required reputation dimension determine eligibility.
The eligible-set root and a future randomness source are committed before the
seed is revealed; domain-separated public hash ranking then selects a committee.

Stake is an eligibility bond, not unlimited voting weight:

\[
E_n = \mathbf 1[b_n \ge b_{min}]\mathbf 1[q_n=1]
      \prod_{d\in D_{policy}}\mathbf 1[r_{n,d}\ge r_{min,d}].
\]

Among eligible nodes, the reference algorithm gives no additional rank weight
for more bond or a higher reputation score. It deterministically searches the
publicly ranked candidates for an assignment with unique VerifiedUserIDs,
OperatorIDs, and operator clusters and
the required provider and region diversity, failing closed when none exists.

### 13.2 Randomness

Selection derives from SortitionID, JobID, policy ID, epoch, role and slot,
NodeID, OperatorID, VerifiedUserID, the committed eligible-set root, and future manipulation-resistant
randomness. Each member exposes its rank hash and the committee exposes a
selection root so any observer can reproduce it exactly. A production
deployment must authenticate a VRF or equivalent beacon proof rather than use
raw same-block data or trust a coordinator-supplied seed.

### 13.3 Operator clustering

Conservative clustering considers declared ownership, payout relationships, infrastructure, deployment keys, signing keys, network patterns, correlated failures, build identity, and historical coordination. Clustering is imperfect, so it is combined with randomized assignment, collateral, checker diversity, open receipts, and challenges.

### 13.4 Commit-reveal

A checker first commits:

\[
c_n=H(\text{JobID}\parallel\text{Verdict}\parallel\text{ObservationRoot}\parallel\text{Salt}).
\]

Only after enough commitments exist does it reveal the receipt and salt. This makes blind copying more difficult.

---

## 14. Aggregation rules by epistemic type

### 14.1 Formal validity

Exact root equality plus role-qualified checker-family requirements. No confidence averaging.

### 14.2 Provenance

Signed contribution manifests, timestamps, ordering references, and dispute windows. Conflicting claims remain visible.

### 14.3 Novelty

For hypothesis `H`, a weighted logarithmic opinion pool can be used:

\[
P(H\mid E) \propto P_0(H)\prod_n P_n(H)^{w_n},
\]

where:

\[
w_n=\operatorname{cap}(\text{calibration}_n\cdot\text{domain}_n\cdot\text{independence}_n\cdot\text{evidence quality}_n).
\]

Reviewers are rewarded for evidence and long-run calibration, not agreement with the majority. Minority reports remain attached.

### 14.4 Significance

Initial significance opinions are provisional. Mature rewards rely on downstream proof dependencies, verified citations, paid API calls, independent implementations, compute saved, and successful applications.

### 14.5 Availability

Thresholds require distinct replicas, operator clusters, providers, and regions, followed by periodic custody challenges.

---

## 15. Node incentives and slashing

### 15.1 Pay for evidence, not agreement

A verifier receives its base execution fee for a complete reproducible receipt regardless of pass or fail. A pass-only payment model would incentivize rubber-stamping.

```text
Node reward = execution cost
            + availability fee
            + timely reveal bonus
            + delayed reliability/calibration reward
```

ASTRA prover nodes may additionally receive an accepted-proof bonus, but that bonus cannot affect checker compensation.

### 15.2 Slash objectively provable misconduct

Slashable acts include:

- equivocation on the same execution;
- fabricated execution evidence;
- signing a different artifact root;
- copying without required trace or custody evidence;
- concealing a material operator relationship;
- failing a custody challenge after attesting availability;
- withholding after a binding reveal commitment.

Do not automatically slash honest checker divergence, good-faith novelty disagreement, or minority detection of a shared implementation bug.

### 15.3 Challenger rewards

A successful objective challenge can receive a capped fraction of slashed collateral. Failed or abusive challenges lose a bond.

---

## 16. Verification job state machine

```text
DRAFT
  → CLAIM_COMMITTED
  → QUOTED
  → FUNDED
  → ASSIGNED
  → FORMALIZING
  → CANDIDATE_READY
  → BUILDING
  → CHECKERS_COMMITTED
  → CHECKERS_REVEALED
      ├── PASSED
      ├── FAILED
      └── DIVERGENT
              → QUARANTINED
  → CHALLENGED
  → FINALIZED / REJECTED / QUARANTINED
  → PUBLISHED
  → REVALIDATED / SUPERSEDED / QUARANTINED
```

Revenue activation occurs only after payment settlement, required formal evidence, policy checks, and challenge completion.

---

## 17. Payment and transport adapters

XLMP messages may travel over HTTP, libp2p, WebSocket, chain event streams, or x402-protected HTTP. Payment may use x402, stablecoins, native-chain assets, backed research credits, grants, bounty escrow, or institutional invoices. These adapters do not replace the research protocol. x402 is the reference paid-HTTP option.

### 17.1 Scheme mapping

| Endpoint type | Scheme |
|---|---|
| Fixed basic Lean verification | `exact` |
| Variable ASTRA proof search | `upto` |
| Metered repair | `upto` |
| Repeated proof-state calls | `batch-settlement` |
| Fixed artifact download | `exact` |
| Continuous research session | `batch-settlement` |

### 17.2 Typical interaction

```text
client → GET/POST protected research resource
server → 402 + PAYMENT-REQUIRED + xlmp extension
client → retry + PAYMENT-SIGNATURE
server/facilitator → verify authorization
server → perform job
facilitator → settle actual amount
server → result + PAYMENT-RESPONSE + separate research receipts
```

### 17.3 XLMP extension

The extension binds:

```text
protocol version
XLMP MessageID
JobID
ResearcherID
ClaimID and optional ProofID
artifact commitment
compute quote
verification policy
model policy
rights manifest hash
revenue route hash
delivery mode
expiration
```

### 17.4 Payment idempotency

Every attempt has a stable payment identifier so retries do not create duplicate charges. Unused `upto` authorization is returned or remains unredeemed.

### 17.5 Reverse-direction bounties

A research bounty pays the solver, unlike ordinary client-pays-server x402. It therefore uses a separate escrow contract with commit-reveal submissions, formal certificate acceptance, challenge period, and payout.

---

## 18. Compute curve

Compute is perishable service capacity; an unused GPU-hour today cannot be economically stored for tomorrow. The curve therefore represents expectations, reservations, and risk transfer—not ordinary inventory carry.

### 18.1 Service curves

\[
F^{ASTRA}_{d,T},\quad F^{Lean}_{T},\quad F^{review}_{d,T},\quad F^{storage}_{T}.
\]

`d` is the research domain/difficulty class and `T` the delivery horizon.

### 18.2 Job quote

For expected quantities `q`:

\[
Q_j(T)=q^A_jF^{ASTRA}_{d,T}+q^L_jF^{Lean}_T+q^R_jF^{review}_{d,T}+q^S_jF^{storage}_T+\pi^{risk}_j.
\]

### 18.3 Quality-Adjusted Certification Cost

For prover model `m`, compute class `g`, probability of Gold verification `P_G`, and novelty clearance `P_N`:

\[
\mathrm{QAC}_{d,T}=
\min_{m,g}
\frac{C_{m,g,T}+C^{verify}_{g,T}+H_{d,T}}
{P_G(m,d,T)P_N(d,T)}.
\]

This compares routes on cost per accepted, novelty-cleared result—not token
price alone. Concrete service units include reasoning tokens, proof-search
attempts, build seconds, checker executions, reviewer hours, and byte-months.
The probabilities come from signed, expiring protocol calibration records,
not provider-advertised success claims.

### 18.4 Quality-adjusted migration spread

\[
\mathrm{MigrationSpread}_{Astra/small,d,T}
=
\frac{\mathrm{QAC}_{Astra,d,T}}
{\mathrm{QAC}_{small,d,T}}.
\]

A higher-priced model can still be economically superior if it materially improves proof yield.

### 18.5 Research lead signal

\[
\mathrm{ResearchLeadSignal}_t=
\frac{\mathrm{ContractedProofAndAPIValue}_t}
{\mathrm{ReservedResearchComputeCost}_t}
\cdot\mathrm{ContractSurvival}_t
\cdot P_G(t)\cdot P_N(t).
\]

The flywheel is confirmed sequentially:

```text
funding → reserved compute → consumed compute → Lean-valid output
→ Gold verification → novelty clearance → downstream use
→ recognized revenue → renewal
```

### 18.6 Scheduling choices

Researchers may choose spot, economy queue, deadline, reserved capacity, or
competitive multi-prover routing. A forward contract is a claim on a defined
future service profile, not stored compute. Tradeable compute futures remain a
deferred layer until service profiles and settlement history are standardized.

---

## 19. Compute-savings impact allocation

A reusable lemma can lower future context size, branching, retries, wall time,
model tier, and checker work. This unobserved counterfactual is one impact
signal, not a precise invoice.

For upstream lemma `k` and downstream proof `j`:

\[
\Delta C_{k\rightarrow j}
=
\operatorname{LCB}_{\alpha}
\left[\widehat C_j^{(-k)}-C_j^{(+k)}\right].
\]

The dividend is:

\[
I_{k\rightarrow j}
=
\min\left[
\rho\max(0,\Delta C_{k\rightarrow j}),
\kappa N_j,
U_j
\right].
\]

`U_j` is the remaining budget of an explicit, settled impact pool. The
evidence graph records formal dependencies; a separate economic graph records
agreed payment obligations. `FORMALLY_DEPENDS_ON` never means
`OWES_PAYMENT_TO`.

Requirements:

- the upstream lemma appears in the final proof dependency graph;
- equivalent duplicates are clustered;
- the estimate uses conservative lower confidence bounds;
- an active economic policy and eligible economic edge authorize allocation;
- payments are capped by realized downstream net revenue and the pool budget;
- no unlimited transitive royalty recursion;
- counterfactual estimation is audited and periodically re-sampled.

Measurement methods include holdout re-proving, matched tasks, proof-state branch reduction, failed Lean iterations avoided, context-token savings, wall-clock savings, model substitution, and downstream verified reuse.

---

## 20. How researchers and supporters earn or participate

### 20.1 Revenue channels

- defined proof bounties;
- x402 proof search, formalization, verification, explanation, and artifact services;
- commercial licenses to code, datasets, implementations, or other enforceable assets;
- capped research-impact allocations;
- retrospective public-goods funding;
- verifier and expert-review work;
- high-quality proof-state trajectory licensing for model training;
- certified builds and formal-verification support;
- early access or confidential research delivery;
- grants and sponsorships.

### 20.2 Support modes

| Mode | Supporter receives | Researcher receives |
|---|---|---|
| Grant | Support certificate and attribution | Unrestricted or purpose-bound funding |
| Bounty | Defined acceptance criteria and certified deliverable | Milestone reward |
| Compute pre-purchase | Future research services or credits | Immediate compute financing |
| Commercial co-development | Contract-defined rights or capped recoupment | Operating capital and compute |

The first three should remain service/grant relationships where possible. A freely transferable instrument promising passive profit from future researcher or protocol effort belongs in a separate legally reviewed wrapper.

### 20.3 Negative results

Counterexamples, failed proof searches, impossibility evidence, and reusable dead ends can receive public-goods or sponsor funding even when they do not generate ordinary usage revenue.

---

## 21. Optional token and NFT projections

### 21.1 Non-transferable origin certificate

Records priority and contributor identity. It cannot be sold to rewrite authorship.

### 21.2 Proof capsule token

A non-transferable or tightly controlled ERC-1155 token may point to one immutable proof capsule. Possession does not own the theorem or alter validity.

### 21.3 License editions

ERC-1155 editions can represent a finite number of commercial licenses, support certificates, access rights, or course/conference editions when backed by a rights manifest.

### 21.4 Public vault token

A token whose value or distributions depend on a researcher's future profits is economically different from a service credit. It should use a separate legal structure and, if technically appropriate, a tokenized-vault interface. It is intentionally outside the core V1 implementation.

---

## 22. Security and anti-gaming controls

| Attack | Primary controls |
|---|---|
| Trivial lemma flood | No reward for compile-only status; novelty/use/bounty gates |
| `sorry` or axiom laundering | Axiom inventory, trust policy, exact challenge, external replay |
| Malicious Lean build code | No-network sandbox, read-only root, resource limits, proof export and replay outside sandbox |
| Dependency substitution | Pinned toolchain, lockfile and dependency root |
| Valid proof of misleading theorem | Trusted challenge, raw formal rendering, definition inspection |
| Solver front-running | Commit-reveal and optional encrypted submission |
| Node copying | Commit-reveal observations and trace requirements |
| Sybil verifier cartel | Verified participant credentials, user/operator/cluster uniqueness, neutral bond, sortition, implementation/provider/region diversity |
| Credential forgery or revoked control | Content-derived credential chain, issuer/delegation signature adapters, fresh registry-root proof, append-only revocation |
| Researcher self-verification | Conflict-of-interest exclusion and independent committee |
| Facilitator capture | Payment/research separation and multiple facilitator paths |
| Compute-cost manipulation | Signed provider offers, realized reconciliation, benchmark sampling |
| Dependency stuffing | Final proof-term dependencies only; duplicate clustering; caps |
| Self-citation rings | Graph-cycle analysis, related-wallet detection, delayed impact rewards |
| False priority | Ordered commitments, prior-art challenges, competing claims retained |
| Rights laundering | Signed clearance and legal-wrapper evidence |
| Signature replay | Domain separation, nonce, expiration, chain and contract binding |
| Contract history rewrite | Immutable core records and append-only adapters |
| Private proof leakage | Client-side encryption, key release after authorization, minimal public roots |
| Model-provider dependence | Provider-neutral adapter and model policy |
| Reviewer herd behavior | Pay for evidence/calibration, preserve minority reports |
| Recursive royalties | Fixed upstream pool and per-result caps |
| Insolvent credit | On-chain/off-chain conservation checks; backing ≥ supply |

---

## 23. Storage, privacy, and availability

Proof bundles form a content-addressed Merkle graph containing source, exact challenge, toolchain, lockfile, dependency roots, exported proof object, axiom report, LaTeX, rendered output, receipts, and manifests.

For private research:

```text
plaintext bundle
  → client-side encryption
  → publish encrypted content-addressed bundle
  → selected payment-adapter authorization/settlement
  → release wrapped decryption key
  → buyer decrypts and independently verifies
```

Public ledgers should contain minimal commitments and economic state. Sensitive unpublished proofs, personal data, export-controlled material, proprietary datasets, and model prompts should not be placed on-chain.

Availability policies require multiple independent storage nodes, providers, and regions, with periodic custody challenges and explicit retention horizons.

---

## 24. Legal and governance boundaries

### 24.1 Mathematical truth versus protectable assets

The protocol never represents that a token exclusively owns a mathematical fact. Potentially protectable or contractible assets may include human-authored manuscripts, source code, datasets, diagrams, explanations, trade secrets, applied inventions, contractual access, and commercial implementations.

### 24.2 Employment, university, sponsor, and grant rights

Contributors must disclose and clear pre-existing obligations. The rights capsule records claims and evidence; it is not a court judgment.

### 24.3 AI-assisted authorship

Human and machine contributions are separately recorded. The protocol does not assume that prompting alone creates human copyright or that model output is free of third-party concerns.

### 24.4 Public profit tokens

Closed-loop backed credits and pay-per-service flows are the recommended launch design. Transferable profit-linked interests require independent securities, commodities, tax, payments, consumer, and jurisdictional analysis.

### 24.5 Governance cannot override proof evidence

Governance may version policies, appoint emergency security roles, fund public goods, and adjudicate objective process violations. It cannot change a failed checker observation into a passing mathematical result.

### 24.6 Upgrades

Protocol upgrades create new policy IDs and versioned adapters. Historical objects retain the policy and software roots under which they were certified. Emergency quarantine is append-only and does not erase attribution.

---

## 25. Recommended launch sequence

The initial product is a sponsor-backed ASTRA-assisted Lean formalization and
independent-certification marketplace in one domain with identifiable buyers,
not a universal market for all research. Cryptographic protocol proofs,
smart-contract properties, verified algorithms, and selected optimization
results are suitable wedges.

### V0.1 — research objects

Canonical IDs, deterministic bundles, Lean annotations/export, local CLI, LaTeX mapping, signed receipts.

### V0.2 — independent reproduction

PoIR nodes, official and independent checkers, commit-reveal, operator diversity, challenge/quarantine, availability attestations.

### V0.3 — paid research services

Backed researcher credits, Research Vaults, pluggable payments with x402 exact/upto/batch as the reference paid-HTTP adapter, provider-neutral prover marketplace with ASTRA as the reference implementation, stable-asset node payouts.

### V0.4 — revenue and impact

Contribution splits, bounties, paid access, bounded impact-pool allocations,
retrospective funding, negative-result support.

### V0.5 — optional rights wrappers

Audited capsule and license tokens, jurisdiction-specific contracts, and separately reviewed public vault interests.

### Longer-term cryptographic finality

1. named verifier federation with public receipts;
2. N-of-M independent signatures and challenges;
3. bonded optimistic verification with reproducible disputes;
4. succinct/zk proof that a minimal checker accepted the exact claim.

---

## 26. Final protocol principle

A decentralized research network should not ask wealthy token holders what is mathematically true. It should let independent operators reproduce exact evidence, let formal checkers determine validity, let reviewers expose uncertainty around novelty and significance, let the ledger settle value, and let researchers continuously recycle real research revenue into future compute.

\[
\boxed{
\text{Nodes are witnesses of reproducible evidence, not governors of truth.}
}
\]
