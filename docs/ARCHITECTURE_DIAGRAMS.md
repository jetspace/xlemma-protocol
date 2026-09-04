# Architecture diagrams

These diagrams are normative illustrations of the trust boundaries described in the numbered specifications. They are written in Mermaid so they render in GitHub-compatible Markdown viewers.

## 1. System context

```mermaid
flowchart LR
    R[Decentralized Researcher Node] -->|claim, budget, rights manifest| G[x402 Research Gateway]
    G -->|metered proof task| A[ASTRA Prover Nodes]
    A -->|candidate Lean artifacts| B[Lean Build Nodes]
    B -->|immutable bundle| C[PoIR Checker Committee]
    C -->|signed observation receipts| E[Evidence Aggregator]
    E -->|PoIR certificate| L[Base Chain / App Rollup]
    L -->|economic finality| P[Proof Registry]
    P --> V[Research Vault + Revenue Router]
    V -->|cash payout + backed credits| R
    P --> S[Storage and Index Nodes]
    W[Watchers / Challengers] -->|counterevidence| E
    W -->|challenge transaction| L
    N[Novelty / Significance Review Nodes] --> E
```

The base chain orders certificates and economic state. It does not execute a social vote over theorem validity.

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
      X[x402 payment]
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
    participant Astra as ASTRA prover
    participant Builder as Lean builder
    participant K1 as Kernel node A
    participant K2 as Kernel node B
    participant IC as Independent checker
    participant Agg as PoIR aggregator
    participant Chain

    Researcher->>Router: Commit ClaimID + ArtifactRoot + budget
    Router-->>Researcher: x402 quote / authorization requirements
    Researcher->>Router: Backed R_i authorization
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
    Router --> Pools[Dependency / security / public-goods pools]
    Router -->|chosen auto-compound share| Vault
    Vault -->|new backing received first| NewCredits[New backed R_i]
```

The required conservation invariant is:

\[
\operatorname{backing}(V_i) \geq \operatorname{totalSupply}(R_i)
\]

No claim, token price, expected royalty, or unverified future profit counts as backing.

## 6. Compute-curve loop

```mermaid
flowchart TD
    O[Signed provider offers] --> Curves[ASTRA / Lean / Review / Storage curves]
    T[Proof-task features] --> Estimator[Expected work and success estimator]
    Curves --> Quote[Verified Proof Cost quote]
    Estimator --> Quote
    Quote --> Scheduler[Spot / economy / deadline / reserved scheduler]
    Scheduler --> Execution[Measured execution]
    Execution --> Receipts[Compute and verification receipts]
    Receipts --> Calibration[Point-in-time calibration]
    Calibration --> Estimator
    Receipts --> Savings[Counterfactual compute-savings estimator]
    Savings --> Dividend[Capped dividend from realized downstream revenue]
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
    CID --> CM[Contribution manifest]
    CID --> RM[Rights manifest]
    CID --> PR1[PresentationID: LaTeX v1]
    CID --> PR2[PresentationID: LaTeX v2]
    PID1 --> DEP[Direct proof dependencies]
    VR1 --> CERT[PoIR certificate]
    CERT --> REV[Revenue route]
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
