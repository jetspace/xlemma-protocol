# xLemma Protocol

**Discover the principles that connect physical phenomena. Make the assumptions
explicit, derive the consequences, and test them against nature.**

xLemma is an open research protocol for producing, independently verifying,
connecting, and funding mathematical and physical knowledge. Researchers freely
choose their questions; pooled USDC funding supports verified contributions under
transparent, budget-conserving rules.

> **Status:** reference implementation and prototype. Local protocol, verification,
> economic, and settlement tests are available. The physics research pilot and
> independent research-compounding evaluation remain planned work. Components
> have not been independently audited; complete the [production gates](ROADMAP.md#production-blockers)
> before deploying with real funds.

[Purpose](#why-xlemma) · [Current status](#what-exists-today) ·
[Get started](#start-here) · [Research workflow](#how-research-moves-through-xlemma) ·
[Funding](#open-research-mining) · [Architecture](#protocol-architecture) ·
[Documentation](#documentation) · [License](#license)

<p align="center">
  <img
    src="./xlemma-graph.PNG"
    alt="xLemma connects people, compute, research, verification, and reusable knowledge"
    width="100%"
  />
</p>

## Why xLemma

The long-term purpose is to expand humanity's capacity to understand nature and
turn discovery into new possibilities:

> Turn computation into cumulative understanding, understanding into shared
> capability, and shared capability into greater freedom to live, discover, and create.

Research advances when others can inspect a result, reproduce it, challenge it,
and build upon it. xLemma coordinates those steps with immutable artifacts,
explicit assumptions, independent evidence, attribution, and funded work.

The research web has many entry points: a lemma, calculation, measurement,
counterexample, or connection between theories. Our Landau-inspired perspective
looks for variational structure, symmetry, conservation, equivalences, and
controlled limits. *Mechanics* and *The Classical Theory of Fields* provide seed
questions; research across mathematics and physics remains open.

Foundational work needs no immediate commercial application. Institutions can
finance research freedom through disclosed funding arrangements; evidence
establishes what a result supports. A mathematical derivation establishes a
consequence of assumptions, while empirical claims need appropriate measurements
and independent replication.

The next priority is a small connected physics corpus and a second generation of
unseen tasks. We will compare ordinary research, reuse through Git, and xLemma's
workflow, counting full costs and evaluating access separately from scientific
gains. See the [project direction](docs/PROJECT_DIRECTION.md),
[physics program](docs/FOUNDATIONAL_PHYSICS.md), and
[research compounding pilot](docs/RESEARCH_COMPOUNDING_PILOT.md).

## What exists today

| Area | Reference implementation and evidence | Still required |
|---|---|---|
| Research protocol | Typed IDs, canonical XLMP/1 messages, append-only projections, authenticated APIs, and a durable event journal. | Clean-room encoding reproduction, distributed operation, and production key management. |
| Verification | PoIR logic, credential and trust-policy boundaries, Lean environment exporter, exact artifact bindings, and local tests. | Independent checker qualification, hardened execution, issuer integration, and operated verification teams. |
| Discovery funding | Signed rounds, calibration, contributor consent, assessment, appeals, USDC escrow, publication bridge, and local EVM integration. | Independent review, actual funding, qualified operators, monitoring, and deployed settlement. |
| Researcher economics | Fully backed credit/vault logic, bounded revenue routing, rights/portability records, and reference payment adapters. | External reconciliation, independently exercised exit, and deployment review. |
| Research program | Landau-inspired seed questions and a defined evaluation plan. | Original physics artifacts, qualified empirical profiles, and independent evidence of research acceleration. |

The [offline discovery simulator](docs/DISCOVERY_PILOT.md) uses synthetic inputs.
The [funded discovery service](docs/DISCOVERY_SERVICE.md) has separate authenticated
service and local EVM tests. Neither establishes a live mining network or a
measured rate of scientific progress.

For recorded checks and limitations, read the [repository audit](docs/REPOSITORY_AUDIT.md),
[service validation report](reports/discovery-service-validation.json), and
[roadmap](ROADMAP.md).

## Start here

### Try the reference implementation

From the repository root, use Python 3.11+ and Rust through `rustup` (the repository
pins Rust 1.82.0 in [rust-toolchain.toml](rust-toolchain.toml)). Python dependency
versions below match CI. These checks require no API credentials or funded wallet.

```sh
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install jsonschema==4.25.1 pyyaml==6.0.2
python3 scripts/validate_repo.py
cargo test --locked --workspace
cargo run --locked -p xlemma-cli -- --help
python3 scripts/simulate_discovery.py --check
```

Lean and Foundry are additional requirements for their respective checks. The
[local-development guide](docs/LOCAL_DEVELOPMENT.md) covers full validation,
reference-vector commands, API configuration, and local settlement testing.

### Choose your path

| Your interest | Start with |
|---|---|
| Research direction and scientific contributions | [Project direction](docs/PROJECT_DIRECTION.md) and [physics program](docs/FOUNDATIONAL_PHYSICS.md) |
| Researchers and collectives | [Participant journeys](docs/RESEARCHER_USER_JOURNEYS.md) and [sovereignty specification](spec/022-researcher-sovereignty.md) |
| Funders and discovery-round operators | [Discovery service](docs/DISCOVERY_SERVICE.md) and [economic model](docs/ECONOMICS.md) |
| Node operators and verifiers | [Operator runbook](docs/OPERATOR_RUNBOOK.md) and [trust-policy registry](spec/023-trust-policy-registry.md) |
| Developers | [Local development](docs/LOCAL_DEVELOPMENT.md), [XLMP/1](spec/018-xlmp-wire-protocol.md), and [OpenAPI](openapi/openapi.yaml) |
| Security and governance reviewers | [Threat model](docs/THREAT_MODEL.md), [constitution](docs/GOVERNANCE_CONSTITUTION.md), and [production checklist](docs/PRODUCTION_CHECKLIST.md) |

## How research moves through xLemma

| Step | What happens | Evidence retained |
|---|---|---|
| Choose and describe | A researcher states a question, assumptions, prior work, and intended contribution. | Claim, theory, contribution, and rights records. |
| Produce | People and tools create proofs, computations, data, or other supported artifacts. | Exact artifacts, dependencies, provenance, and compute receipts. |
| Reproduce | Qualified independent operators perform the declared checks. | Signed observations, verification receipts, and certificate status. |
| Assess and challenge | Evidence validity and reward eligibility receive separate decisions; participants can appeal. | Reasons, prior-art/grouping evidence, dissent, and append-only remedies. |
| Publish, reuse, and settle | Final artifacts become reusable under their declared terms; eligible funded work settles after applicable holds. | Publication, attribution, availability, settlement, and later correction records. |

A result can be verified without earning a discovery award. Rejected or
inconclusive work can still support an informative finding and an agreed fee for
honest completed checking. Reuse does not create an automatic royalty obligation.

## Verification and appeals

### Proof of Independent Reproduction

A verifier signs what it executed: checker implementation, exact artifact,
environment, policy, and observed result. Formal verification binds the structural
`ClaimID`, `ProofID`, artifact/dependency roots, and permitted axiom inventory.
Source text alone is never the final formal ClaimID.

PoIR policies require qualified, independent operator domains and checker families.
Required checker-family disagreement produces `DIVERGENT`/`QUARANTINED` status;
token-weighted voting, reputation, or a numerical majority cannot repair it.
ASTRA and other proof producers cannot certify their own output.

Formal validity and statement alignment remain separate. A correct proof can
formalize a vacuous, weakened, or misleading version of the intended claim.
Domain review must make that distinction visible. Computational, statistical,
simulation, empirical, and hybrid profiles carry their own evidence requirements;
a proof or simulation cannot stand in for experimental confirmation.

### Independent review and correction

Evidence, prior art, grouping, eligibility, attribution, and allocation decisions
have reasoned appeal paths with independent reviewers, funded access, deadlines,
and explicit remedies. Timely unresolved appeals hold affected allocations.
Corrections append records; they never silently replace historical evidence.
Honest completed verification and review are paid under agreed terms regardless
of a positive verdict.

See [PoIR](spec/003-poir-consensus.md), [trust policies](spec/023-trust-policy-registry.md),
[verification and appeals](spec/024-open-research-mining.md#verification-and-appeals),
and the [service runbook](docs/DISCOVERY_SERVICE.md) for exact rules and remaining
operational requirements.

## Open research mining

Researchers choose their own questions. Open pools do not require a posted bounty,
institutional affiliation, or immediate commercial buyer. Institutions may fund
broad fields or specific programs with disclosed restrictions, fees, and conflicts.
Only settled funds become spending capacity.

| Decision | What it establishes |
|---|---|
| Verified knowledge | The artifact meets a declared evidence profile, with explicit assumptions and limitations. |
| Rewardable contribution | It adds something under published rules: new knowledge, first formalization, proof improvement, useful tools, replication, or informative negative results. |

Separate foundational, discovery, formalization, proof-improvement, replication,
tools, and negative-result budgets protect different kinds of work. Known results
can qualify for formalization support without being presented as discoveries.

Published round rules determine assessment, category budgets, capacity, contributor
splits, and appeals. Difficulty and reference cost are contestable inputs; raw
tokens, wall time, proof length, and submission count do not directly determine
awards. Renaming, duplication, artificial splitting, and identity rotation cannot
enlarge the same contribution's reward. Useful intermediate results require
evidence of additional contribution.

Completed verification and review have separate reserves. Awards remain subject to
review, appeals, and confirmed funding; total spending cannot exceed available
funds after commitments and reserves. The reference service also prevents repeated
checking-job/operator charges and replayed completed-work invoices. Semantic
novelty and beneficial-control assessment still need qualified independent review.

See [XLIP-024](spec/024-open-research-mining.md), the
[discovery service](docs/DISCOVERY_SERVICE.md), and the
[activation gates](ROADMAP.md#open-research-mining--activation-gates).

### Research-credit economics

Research credits are an optional, fully backed way to prepay services. A deposit
of 1,000 USDC can back at most 1,000 corresponding service credits; consuming
credits releases only the backing for actual authorized usage. Credits cannot be
minted against an unverified lemma, expected profit, or their own token price.
Direct discovery rewards do not require researchers to hold credits.

A researcher may reinvest a chosen share of settled revenue into backed credits.
That financial reinvestment is distinct from the research pilot's proposed
compounding of knowledge. Neither estimated compute savings nor a dependency edge
creates new money. See [Economics](docs/ECONOMICS.md).

## Researcher sovereignty and portable exit

Origin, attribution, artifact control, economic participation, license control,
governance consent, and portability are represented explicitly. A capsule links
its research artifacts, evidence, contribution history, and rights; ownership of
a token does not rewrite any of them.

Commons is the default economic mode, with no mandatory per-use protocol fee.
Reciprocal, Commercial Artifact, and Sponsored Challenge modes can carry explicit,
bounded terms. Residual rights do not confer ownership of mathematical truth or
an unlimited claim on downstream work.

Portable manifests describe how independent clients can reconstruct research
state and preserve attribution and evidence. Independently exercised exit remains
a production requirement. See the [sovereignty specification](spec/022-researcher-sovereignty.md)
and [governance constitution](docs/GOVERNANCE_CONSTITUTION.md).

## Protocol architecture

### XLMP/1 is the protocol boundary

XLMP/1 defines canonical signed messages, research objects, evidence and economic
state transitions. The evidence graph records assumptions, dependencies and
verification. The economic graph records separately authorized obligations:
`FORMALLY_DEPENDS_ON != OWES_PAYMENT_TO`.

| Layer | Responsibility | Reference modules |
|---|---|---|
| Research objects | IDs, claims, proofs, rights, contributions, and append-only lineage | `xlemma-core`, `xlemma-xlmp`, `xlemma-crypto` |
| Node network | Service advertisement, matching, sortition, credentials, and independent reproduction | `xlemma-node`, `xlemma-consensus` |
| Economics | Funded discovery, backed credits, service quotes, settlement, and bounded revenue routing | `xlemma-economics`, `xlemma-compute-curve`, `contracts/` |
| External adapters | Models, checkers, payment rails, storage, and transport | `xlemma-astra`, `xlemma-lean`, `xlemma-x402`, `xlemma-storage` |
| Interfaces | Authenticated API, durable journal, CLI, schemas, and presentation tools | `xlemma-api`, `xlemma-cli`, `openapi/`, `schemas/`, `lean/`, `latex/` |

ASTRA, Lean, x402, chains, and storage systems connect through provider-neutral
boundaries. Each has a defined role; payment receipts remain separate from
verification receipts. The API's durable journal is a single-writer reference
implementation; distributed reconciliation and recovery remain production work.

### First-class node network

Nodes advertise roles, supported profiles, capacity, prices, and implementations.
Eligibility uses credentials, bonds, and multidimensional reputation. Authority
to certify comes from policy-qualified independent reproduction.

A public pseudonymous credential chain binds a verified participant to an operator
and its nodes: `VerifiedUserID → OperatorID → NodeID`. Multiple machines under
common control count as one independence domain. Fresh non-revocation evidence is
required; higher credential tiers do not make a proof more valid. Issuer trust,
private uniqueness evidence, and beneficial-control assessment remain explicit
external dependencies.

See [XLMP/1](spec/018-xlmp-wire-protocol.md), the [node network](spec/019-node-network.md),
and [identity and credentials](spec/020-identity-credentials.md).

## Documentation

| Topic | References |
|---|---|
| Direction and next work | [Roadmap](ROADMAP.md), [project direction](docs/PROJECT_DIRECTION.md), [research compounding pilot](docs/RESEARCH_COMPOUNDING_PILOT.md) |
| Full architecture | [Design](docs/FULL_DESIGN.md), [diagrams](docs/ARCHITECTURE_DIAGRAMS.md), [traceability](docs/TRACEABILITY_MATRIX.md) |
| Formalization and presentation | [Lean package](lean/README.md), [Lean/LaTeX guide](docs/LEAN_LATEX_GUIDE.md), [ASTRA prompts](docs/ASTRA_PROMPTS.md) |
| Service and economic design | [Discovery service](docs/DISCOVERY_SERVICE.md), [economics](docs/ECONOMICS.md), [compute curve](spec/005-compute-curve.md), [x402](spec/008-x402-transport.md) |
| Development and deployment | [Local development](docs/LOCAL_DEVELOPMENT.md), [contracts](contracts/README.md), [deployment architecture](docs/DEPLOYMENT_ARCHITECTURE.md), [operator runbook](docs/OPERATOR_RUNBOOK.md) |
| Validation and review | [Testing strategy](docs/TESTING_STRATEGY.md), [repository audit](docs/REPOSITORY_AUDIT.md), [participant conformance](docs/USE_CASE_SIMULATION_REPORT.md), [production checklist](docs/PRODUCTION_CHECKLIST.md) |
| Sources and scope | [Source register](docs/SOURCES.md), [prior art](docs/PRIOR_ART_AND_DIFFERENTIATION.md), [legal boundaries](docs/LEGAL_BOUNDARIES.md) |

### Repository map

```text
crates/                   Rust protocol, services, and adapter implementations
contracts/                Solidity registries, escrow, credits, and vaults
lean/                     Lean exporter, metadata, and reference example
latex/                    Research presentation package and example
spec/                     Normative protocol specifications
schemas/                  JSON Schema definitions
openapi/                  HTTP API contract
config/                   Reference policy configuration
examples/                 Protocol, discovery, and deterministic reference vectors
docs/                     Design, research plans, operations, and review records
reports/                  Machine-readable validation and simulation evidence
deploy/                   Local container deployment templates
scripts/                  Validation, simulation, and source-packaging tools
```

## Non-negotiable invariants

<details>
<summary>Protocol constraints preserved by every implementation</summary>

1. Researchers cannot pay nodes with unbacked self-minted value.
2. ASTRA may produce proofs but cannot certify its own output.
3. Verifiers are paid for reproducible execution, not agreement or a passing verdict.
4. Required checker disagreement causes divergence and quarantine.
5. Researchers cannot independently finalize their own claims.
6. Realized token appreciation is not research revenue.
7. Profit distributions originate only from settled external revenue after costs and reserves.
8. Only final proof dependencies qualify for dependency rewards.
9. Formal claim changes create new `ClaimID`s.
10. Corrections, revocations, dissent, and supersession remain append-only.
11. Payments, formal validity, novelty, rights, and availability have separate receipts.
12. Token ownership cannot rewrite authorship or Lean validity.
13. A verified Lean theorem may still be trivial, previously known, misleadingly described, or commercially valueless.
14. Rights manifests cannot create intellectual-property rights the contributor never owned.
15. Quorum requirements measure distinct verified participants, operators, conservative control clusters, and implementations—not public-key count.
16. A node cannot enter consensus without a valid V2-or-higher credential chain and a fresh non-revocation proof.
17. Credentials qualify accountable participants; they never certify proofs or override exact checker evidence.
18. Public protocol identity remains pseudonymous; private legal and uniqueness evidence remains outside public XLMP objects.
19. Verified truth alone never creates a reward entitlement; novelty, formalization, proof improvement, and replication have separate, published eligibility rules.
20. Renaming, duplicating, or artificially splitting the same contribution cannot increase its aggregate reward; raw compute expenditure and submission speed cannot bypass this rule.
21. Open-pool eligibility does not require a posted bounty, institutional affiliation, or immediate commercial demand.
22. Discovery allocations and reserved liabilities cannot exceed settled funding; reward policies are fixed before a round opens and affected payouts wait for timely appeals.
23. Evidence and reward decisions have independent, reasoned, append-only appeal paths; neither funders nor appeal panels can vote a failed proof into validity.

The open-mining invariants are requirements for activation; the implementation
gaps are explicitly tracked in XLIP-024 and the roadmap.

</details>

## License

xLemma's original code and documentation are licensed under [Apache-2.0](LICENSE),
unless a file explicitly states otherwise. Third-party dependencies and separately
licensed research artifacts retain their own terms.
