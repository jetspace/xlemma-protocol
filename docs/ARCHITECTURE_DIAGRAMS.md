# Architecture diagrams

These diagrams are normative illustrations of the trust boundaries described in the numbered specifications. They are written in Mermaid so they render in GitHub-compatible Markdown viewers.

## 1. System context

```mermaid
flowchart TB
    X[XLMP/1 canonical protocol]
    X --> RG[Research graph<br/>claims, proofs, dependencies,<br/>provenance, rights]
    X --> N[Node network]
    X --> E[Economics<br/>credits, revenue, compute,<br/>bounties, dividends]
    N --> D[Discovery<br/>signed advertisements]
    N --> M[Service markets<br/>orders and immutable matches]
    N --> SO[Committee sortition<br/>committed set + future randomness]
    N --> I[Identity credentials<br/>participant → operator → nodes<br/>delegation + revocation]
    N --> R[Reputation + bonding<br/>multidimensional eligibility]
    N --> C[PoIR consensus<br/>challenge, quarantine, revalidation]
    RG --> AB[Adapter / transport boundary]
    N --> AB
    E --> AB
    AB --> A[ResearchProver<br/>ASTRA / others]
    AB --> L[VerifierAdapter<br/>Lean / Coq / Rocq / others]
    AB --> P[PaymentAdapter<br/>x402 / credits / grants / escrow]
    AB --> F[FinalityAdapter<br/>Ethereum / Base / Solana / others]
    AB --> S[StorageAdapter<br/>IPFS / object stores / archives]
    AB --> T[TransportAdapter<br/>HTTP / libp2p / WebSocket]
```

XLMP owns research state and consensus. Every named external technology is an adapter. A chain may order certificates and economic state, but it does not execute a social vote over theorem validity.

## 1.1 Node-network routing and authority

```mermaid
flowchart LR
    A[Signed service advertisement] --> D[Constraint-filtered discovery]
    D --> O[Service order]
    O --> M[Immutable service match]
    M --> P[Separate payment authorization]
    M --> X[Measured execution receipt]

    B[Active bond] --> EL[Eligibility]
    RV[Eight-dimension operator reputation<br/>with node subrecords] --> EL
    VC[Valid non-revoked V2+ chain<br/>VerifiedUser → Operator → Node] --> EL
    CF[Role + checker family] --> EL
    EL --> ES[Committed eligible-set root]
    FR[Future public randomness] --> S[Deterministic sortition]
    ES --> S
    S --> C[Reproducible committee]
    C --> POIR[Independent PoIR observations]
```

Price routes services. Bond and reputation gate eligibility. Neither creates
formal vote weight; mathematical status still follows exact independent
checker evidence.

One committee slot consumes all three independence identifiers. A collision on
VerifiedUserID, OperatorID, or conservative OperatorClusterID rejects the
candidate; adding NodeIDs cannot multiply one participant's authority.

## 2. Trust planes

```mermaid
flowchart TB
    subgraph Formal[Formal-validity plane]
      L1[Lean kernel A]
      L2[Lean kernel B]
      I1[Independent checker]
      Q[Generalized equality quorum]
      L1 --> Q
      L2 --> Q
      I1 --> Q
    end

    subgraph Provenance[Provenance plane]
      O[Origin commitment]
      CM[Signed contribution manifest]
      OR[Chronological ordering reference]
    end

    subgraph Review[Novelty and interpretation plane]
      NR[Prior-art evidence]
      ER[Expert review]
      NP[Calibrated posterior]
    end

    subgraph Economic[Economic plane]
      X[Payment adapter<br/>x402, credits, grants, escrow]
      BV[Backed vault]
      RR[Revenue route]
      BF[Base-chain finality]
    end

    subgraph Availability[Availability plane]
      SR1[Storage receipt A]
      SR2[Storage receipt B]
      SR3[Storage receipt C]
    end

    Q --> RCC[Research Consensus Certificate]
    O --> RCC
    CM --> RCC
    OR --> RCC
    NP --> RCC
    SR1 --> RCC
    SR2 --> RCC
    SR3 --> RCC
    RCC --> BF
    X --> BF
    BV --> BF
    RR --> BF
```

Each plane has a different aggregation rule. Formal validity requires deterministic equality across required checker families. Novelty uses evidence-weighted review. Economics uses ledger finality. These outputs must remain separately inspectable.

## 3. Proof lifecycle

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> ClaimCommitted
    ClaimCommitted --> Quoted
    Quoted --> Funded
    Funded --> Assigned
    Assigned --> Formalizing
    Formalizing --> CandidateReady
    CandidateReady --> Building
    Building --> CheckersCommitted
    CheckersCommitted --> CheckersRevealed
    CheckersRevealed --> Passed: all required families agree
    CheckersRevealed --> Failed: deterministic rejection
    CheckersRevealed --> Divergent: required observations disagree
    Divergent --> Quarantined
    Passed --> Challenged
    Passed --> Finalized: challenge window expires
    Challenged --> Quarantined: credible unresolved evidence
    Challenged --> Passed: challenge dismissed + new window
    Finalized --> Published
    Published --> Revalidated
    Published --> Quarantined: later checker compromise
    Revalidated --> Published
    Published --> Superseded
    Failed --> [*]
    Quarantined --> [*]
    Superseded --> [*]
```

## 4. Independent reproduction sequence

```mermaid
sequenceDiagram
    participant Researcher
    participant Router
    participant Payment as Payment adapter
    participant Astra as ResearchProver (ASTRA)
    participant Builder as Verifier adapter (Lean)
    participant K1 as Kernel node A
    participant K2 as Kernel node B
    participant IC as Independent checker
    participant Agg as PoIR aggregator
    participant Chain

    Researcher->>Router: Commit ClaimID + ArtifactRoot + budget
    Router-->>Researcher: XLMP compute quote
    Researcher->>Payment: authorize via selected payment rail
    Payment-->>Router: separate payment receipt
    Router->>Astra: Formalize/search/repair task
    Astra-->>Router: Candidate + generation receipt
    Router->>Builder: Reproducible build request
    Builder-->>Router: Pinned artifact bundle
    par Independent execution
      Router->>K1: Verify artifact
      Router->>K2: Verify artifact
      Router->>IC: Verify artifact
    end
    K1-->>Agg: Commitment
    K2-->>Agg: Commitment
    IC-->>Agg: Commitment
    Agg-->>K1: Reveal phase open
    Agg-->>K2: Reveal phase open
    Agg-->>IC: Reveal phase open
    K1-->>Agg: Observation receipt + salt
    K2-->>Agg: Observation receipt + salt
    IC-->>Agg: Observation receipt + salt
    alt required roots and verdicts agree
      Agg->>Chain: Submit PoIR certificate
      Chain-->>Researcher: Final after challenge window
    else required family diverges
      Agg->>Chain: Submit divergence evidence
      Chain-->>Researcher: Quarantined; no revenue activation
    end
```

## 5. Research-credit conservation

```mermaid
flowchart LR
    D[External stable asset deposit] --> Vault[Research Vault]
    Vault -->|1:1 mint| RC[Restricted Research Credit R_i]
    RC -->|authorize maximum| Lock[Per-job credit lock]
    Lock -->|actual use| Burn[Burn actual credits]
    Burn -->|release matching backing| Nodes[Independent service nodes]
    Lock -->|unused amount| Refund[Return credits]

    ExtRev[Settled external research revenue] --> Router[Revenue Router]
    Router --> Cash[Researcher cash payout]
    Router --> Pools[Explicit economic-policy / security / public-goods pools]
    Router -->|chosen auto-compound share| Vault
    Vault -->|new backing received first| NewCredits[New backed R_i]
```

The required conservation invariant is:

\[
\operatorname{backing}(V_i) \geq \operatorname{totalSupply}(R_i)
\]

No claim, token price, expected royalty, or unverified future profit counts as backing.

## 6. Job-specific service-cost loop

```mermaid
flowchart TD
    O[Signed provider price/capacity offers] --> Curves[ASTRA / Lean / Review / Storage curves]
    T[Proof-task features and audited outcomes] --> Estimator[Independent protocol success estimator]
    Curves --> Quote[Quality-Adjusted Certification Cost]
    Estimator --> Quote
    Quote --> Scheduler[Spot / economy / deadline / reserved scheduler]
    Scheduler --> Execution[Measured execution]
    Execution --> Receipts[Compute and verification receipts]
    Receipts --> Calibration[Point-in-time calibration]
    Calibration --> Estimator
    Receipts --> Savings[Counterfactual compute-impact estimator]
    Policy[Settled economic policy and bounded pool] --> Allocation[Capped impact allocation]
    Savings --> Allocation
```

## 7. Research object graph

```mermaid
graph TD
    RID[ResearcherID] -->|created| LID[LemmaID]
    LID --> TID[TheoryID]
    LID --> CID[ClaimID]
    CID --> PID1[ProofID v1]
    CID --> PID2[ProofID v2]
    PID1 --> AID1[ArtifactID]
    PID2 --> AID2[ArtifactID]
    AID1 --> VR1[Verification receipts]
    AID2 --> VR2[Verification receipts]
    CID --> ORI[Origin certificate]
    CID --> ALIGN[Statement alignment receipts]
    CID --> CM[Contribution manifest]
    CID --> RM[Rights manifest]
    CID --> PR1[PresentationID: LaTeX v1]
    CID --> PR2[PresentationID: LaTeX v2]
    PID1 --> DEP[Direct proof dependencies]
    VR1 --> CERT[PoIR certificate]
    ECON[Economic policy graph] --> REV[Revenue route]
    SETTLE[Settled external revenue] --> REV
    DEP -. evidence only .-> ECON
    LID -->|supersedes| OLD[Prior LemmaID]
```

Formal and presentation identities are separate. Editing exposition does not silently change the formal claim; changing the formal claim creates a new `ClaimID`.

## 8. Deployment topology

```mermaid
flowchart TB
    Internet --> Edge[API gateway / rate limits / DDoS controls]
    Edge --> API[xLemma API]
    API --> Jobs[Durable job queue]
    API --> Pay[x402 adapter]
    API --> Store[Metadata database]
    Jobs --> Provers[ASTRA prover pool]
    Jobs --> Builders[Ephemeral Lean builders]
    Jobs --> Checkers[Independent checker pools]
    Jobs --> Reviewers[Review node network]
    Jobs --> Aggregator[PoIR aggregator]
    Aggregator --> Chain[Settlement chain / rollup]
    Store --> Index[Indexing and graph service]
    Builders --> CAS[Content-addressed artifact storage]
    Checkers --> CAS
    Watchers[External watcher nodes] --> CAS
    Watchers --> Chain
    Metrics[Metrics / logs / traces] --- API
    Metrics --- Jobs
    Metrics --- Provers
    Metrics --- Builders
    Metrics --- Checkers
```

Production checker jobs must run in no-network, resource-constrained, ephemeral sandboxes. A proof artifact may be public, encrypted, or access-controlled, but its certificate must bind the exact content commitment.

## 9. Failure containment

```mermaid
flowchart LR
    D[Checker divergence] --> Q[Quarantine]
    C[Checker CVE or compromised image] --> Q
    A[Artifact unavailable] --> R[Availability degraded]
    V[Vault insolvency signal] --> F[Freeze new authorization]
    P[Payment facilitator outage] --> PS[Payment degraded only]
    M[ASTRA outage] --> PG[Proof generation degraded only]
    Q --> Reproduce[Expanded reproduction committee]
    Reproduce --> Restore[Revalidated certificate]
    Reproduce --> Reject[Reject / revoke]
```

Failures are compartmentalized. A payment outage cannot change formal status, and a model outage cannot invalidate an already verified proof.

## 10. Researcher-sovereignty and anti-capture planes

```mermaid
flowchart TB
    Researcher[Verified researcher / cooperative]
    Bundle[Sovereignty bundle<br/>origin · attribution · custody · participation · license · consent · exit]
    Evidence[Evidence graph<br/>dependencies · provenance · reproduction]
    Economy[Economic graph<br/>explicit policies · settled obligations]
    Network[Node network<br/>discovery · sortition · reputation · bonding]
    Funding[Market · Commons · Assurance]
    Portable[Portable export<br/>open client · redundant storage · funds exit]
    Chain[Settlement projection<br/>roots · ordering · bonds · challenges · revenue]
    Adapters[Replaceable adapters<br/>provers · verifiers · payments · chains · storage]

    Researcher --> Bundle
    Bundle --> Evidence
    Bundle --> Economy
    Network --> Evidence
    Network --> Economy
    Funding --> Economy
    Bundle --> Portable
    Evidence --> Portable
    Economy --> Portable
    Evidence --> Chain
    Economy --> Chain
    Adapters --> Network
    Adapters --> Portable
```

Evidence edges never authorize payment by themselves. Economic edges never
establish scientific validity. The capture dashboard measures identity,
compute, models, verification, storage, settlement, discovery, and governance,
and publishes the weakest layer as the effective independence score.
