# XLIP-022 — Researcher sovereignty, portability, and economic constitutions

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative. This
specification defines protocol records and validation rules, not ownership of
mathematical truth or legal advice.

## 1. Constitutional purpose

xLemma converts research contribution into portable, verifiable, economically
participatory knowledge capital. Infrastructure providers supply replaceable
services; they do not acquire origin, formal validity, attribution, artifacts,
or economic rights merely by supplying compute, models, settlement, indexing,
credentials, or storage.

The protocol MUST preserve:

```text
Truth remains open.
Origin remains attributable.
Artifacts remain controllable to the extent of demonstrated rights.
Economic participation remains explicit, bounded, and programmable.
Compute remains contestable.
Verification remains independently reproducible.
Protocol state remains portable.
```

## 2. ResearcherSovereigntyBundle

Every protected research object SHOULD publish a content-derived
`ResearcherSovereigntyBundle` binding exactly seven distinct protections:

1. origin;
2. attribution;
3. artifact control;
4. economic participation;
5. license control;
6. governance consent;
7. portability and exit.

Origin, attribution, and portability/exit MUST be nontransferable and MUST NOT
be revocable by protocol governance. Origin MUST remain challengeable through
append-only evidence; a successful challenge supersedes or qualifies a record
and never silently rewrites history. Artifact and license control MUST be
limited to rights actually supported by the linked rights evidence.

The bundle MUST link the exact claim, origin receipt, contribution manifest,
rights manifest, economic policy, controlled artifact roots, credential
commitments, direct-custody vaults, and a portability manifest. Its identifier
MUST cover these fields and any supersession edge. Signatures are authenticated
under the XLMP signature domain and MUST NOT be treated as valid merely because
their byte strings are nonempty.

## 3. ResearcherResidualRight

A `ResearcherResidualRight` (RRR) is a nonexclusive residual claim on named,
qualifying, settled xLemma economic activity. It is not ownership of a claim,
proof, theorem, fact, or downstream work.

An RRR MUST declare:

- the origin researcher and exact claim;
- current beneficiary and direct-custody payee vault;
- the economic-policy root and qualifying revenue sources;
- a share, per-event cap, lifetime cap, maximum graph depth, depth decay, and
  per-ancestor cap;
- nonexclusivity, no downstream veto, non-recursion, one charge per revenue
  event, and equivalent-claim clustering.

An RRR MUST NOT apply to bare citations, unused dependencies, or independently
created external claims. Assignment is valid only through an append-only record
that identifies both parties, links a signed agreement, and carries both
signatures. Transfer of a token or artifact does not assign an RRR.

## 4. Capsule economic constitutions

Every capsule selects exactly one constitution:

| Mode | Protocol effect |
|---|---|
| `commons` | No mandatory use payment. Grants, donations, sponsorship, and separately authorized impact funding remain possible. |
| `reciprocal` | A named class of monetized xLemma descendants contributes one bounded upstream pool. No veto or recursive charge exists. |
| `commercial_artifact` | Controlled artifacts or services may carry explicit, scoped, finite commercial terms. The public claim is not enclosed. |
| `sponsored_challenge` | Funding, acceptance, contributor allocation, result rights, deadlines, and disputes are committed before work begins. |

All modes MUST reject bare-citation eligibility, independently created claims,
downstream vetoes, recursive charging, repeated charging of one revenue event,
and unclustered equivalent claims. A `commons` constitution MUST have a zero
mandatory upstream pool. A `reciprocal` constitution MUST name qualifying
revenue, use nonzero finite caps and a minimum-payout floor, and bound both
ancestor count and depth. Payouts below the declared asset-minor-unit floor
remain in the explicit unallocated remainder; they do not disappear from the
conservation equation.

`EconomicComplianceCertificate` separately reports `COMPLIANT`,
`NONCOMPLIANT`, or `DISPUTED` for the exact obligation roots and settled
revenue/receipt set. `COMPLIANT` requires every required obligation to be
satisfied; non-Commons settlement obligations require both revenue-event and
settlement-receipt evidence. This status has no effect on research validity.

## 5. Evidence and economic graphs

The evidence graph is descriptive. Its edge kinds include
`FORMALLY_DEPENDS_ON`, `EXTENDS`, `USES_DATASET`, `USES_LIBRARY`, `CITES`, and
`EQUIVALENT_TO`. An evidence edge can establish use or provenance but its
protocol-level `authorizes_payment` result is always false.

The economic graph is prescriptive. Its edges include `PAYS_TO`,
`CONTRIBUTES_TO_UPSTREAM_POOL`, and `ALLOCATES_BOUNTY_TO`. An economic edge is
valid only when it binds an economic policy, qualifying revenue source, finite
cap, effective interval, authorization root, and signatures.

The invariant is:

```text
FORMALLY_DEPENDS_ON != OWES_PAYMENT_TO
```

Allocation and settlement logic MUST require both evidence of qualifying use
and an independent economic authorization. No dependency path may spend a
revenue-event pool recursively.

## 6. Contribution accounting

Contribution manifests distinguish human roles including question originator,
formula or conjecture author, dataset creator, experimental contributor, proof
discoverer, formalizer, tool or tactic developer, statement-alignment reviewer,
independent verifier, application developer, maintainer, reviewer, and
exposition author. Machine records separately disclose provider, model,
snapshot, request, context, output artifacts, and human selection or edits.

Model or compute operation alone does not grant origin. Omitting a supported
contributor MAY make a downstream capsule attribution- or
economic-noncompliant, but it MUST NOT alter the result of exact verification.

## 7. Portability and company-disappearance recovery

A `ResearcherPortabilityManifest` MUST be sufficient for an independent client
to reconstruct identity links, artifacts, contributions, verification receipts,
economic policies, settlement commitments, and event-log history without a
proprietary xLemma database. Every declared artifact MUST have at least two
independent reconstructable storage locations whose records name distinct
provider identities and retrieval-evidence roots. Event-log history MUST also
have at least two provider-independent locations. The manifest binds the open
reconstruction-client source and funds-exit instructions. The manifest itself
is content-derived, signed, append-only, and supersedable.

Deployments MUST expose open event logs and export operations. Governance,
frontends, indexers, credential issuers, and xLemma-affiliated companies MUST
NOT be able to block export of public records or researcher-controlled
encrypted objects. Settlement adapters MUST provide a documented path for the
researcher to exit with settled funds under direct custody.

## 8. Verification profiles

XLMP defines six verifier-neutral classes:

- `FORMAL`: formal statement, proof object, pinned toolchain, and axiom inventory;
- `COMPUTATIONAL`: source, execution environment, dependency lock, and deterministic rerun;
- `STATISTICAL`: data provenance, analysis plan, source, uncertainty, and robustness checks;
- `SIMULATION`: model, seeds, parameter ranges, convergence, and sensitivity;
- `EMPIRICAL`: preregistration, instruments, data lineage, and independent replication;
- `HYBRID`: formal model plus computational and empirical evidence.

Every profile MUST name verifier implementations, require at least two
reproductions under at least two independent operator domains, and define a
nonzero challenge window. Lean is the default formal adapter, not the protocol
authority. Other proof systems and non-formal reproduction services implement
the same verifier boundary without changing XLMP research state semantics.

For non-formal and hybrid profiles, each `ReproductionObservation` MUST bind
the exact job, claim, input artifact, profile class, required evidence roots,
verifier implementation, node, verified participant, operator, operator
cluster, credential IDs and chain root, provider, region, verdict, execution
trace, and time. A node first publishes `XLMP_OBSERVATION_COMMIT`, then reveals
the salted evidence commitment in `XLMP_REPRODUCTION_OBSERVATION`; the node's
registered key signs the observation itself as well as the XLMP envelope.
Certificates may reference only observations already authenticated through
that ingress sequence. `ResearchVerificationCertificate` evaluates the exact
receipt sequence. Any mixture of `PASS` and `FAIL` is `DIVERGENT`, regardless
of majority; errors, abstentions, or missing threshold evidence are
`INCONCLUSIVE`. Duplicate node, participant, operator, or operator-cluster
domains fail closed, and the producing cluster cannot count as an independent
reproducer.

## 9. Market, commons, and assurance funding

XLMP funding has three parallel rails. Market funding purchases bounded work or
services. Commons funding supports foundational work, negative results, open
infrastructure, maintenance, and retrospective impact. Assurance funding
backs verification, challenges, revalidation, certificate warranties, and
reliance insurance.

A `FundingReceipt` MUST bind one rail-compatible purpose, finalized external
settlement, an external-value evidence root, destination vault, and policy.
Research-credit inflation, an unrealized token-price increase, or a circular
related-party transfer MUST NOT be represented as independent funding. Fee
policies MUST conserve the exact settled amount and assign a nonzero share to
commons and assurance infrastructure.

## 10. Compute contestability

Canonical compute services MUST be provider-neutral. Proof-generation demand
uses `research_prover_generation`; ASTRA and other model systems appear only as
implementations or adapters. Procurement proceeds from spot quotes through
maximum-cost authorizations, reserved capacity, domain service forwards, and
nontransferable capacity options. Transferable compute derivatives are outside
XLMP/1.

Routes MUST be bounded by spend and SHOULD enforce independent provider,
model-family, and region diversity, confidential delivery, and fallback
capacity. Researcher-owned compute cooperatives are native participants but
each cooperative counts as one operator-control cluster for one job, with
independence reduced for ownership overlap.

No protocol economics may require compute to remain scarce. In a cheap-compute
regime, funding and reputation can shift toward question selection, novelty,
curation, empirical grounding, application, and maintenance without changing
the verification or conservation rules.

## 11. On-chain and off-chain boundary

Settlement or finality layers SHOULD contain commitments and economic state:
claim and artifact roots, policy identifiers, bond and committee commitments,
certificate roots, challenge state, settled revenue, contributor splits,
rights hashes, and supersession edges.

Large or confidential material remains off-chain but content-addressed: source,
proof objects, dependency graphs, model-generation artifacts, manuscripts,
review evidence, datasets, execution traces, full credentials, and legal
agreements. At least two independent storage locations retain each portable
artifact. Chain inclusion orders or settles a commitment; it does not prove
validity, attribution, novelty, or legal rights.

The reference `ResearchCommitmentRegistry` is the minimal generic projection:
it commits researcher, claim, artifact, policy, committee-assignment, rights,
contributor-split, and parent roots and permits correction only through an
explicit supersession link. PoIR challenge state, bonds, and realized revenue
remain in their dedicated registries. No contract stores or interprets the
underlying evidence.

## 12. Conformance and safety

A conforming implementation MUST:

- derive sovereignty, residual-right, and portability identifiers from
  canonical content;
- fail closed on missing evidence, identity mismatches, self-supersession,
  insufficient storage replication, unsigned assignment, unbounded economic
  terms, or incomplete verification profiles;
- preserve historical versions rather than mutate them;
- display formal status, rights compliance, attribution compliance, and
  economic compliance as separate states;
- keep ASTRA, models, checkers, payment transports, chains, storage providers,
  identity issuers, and indexers replaceable.

Token balances, bonds, revenue, reputation, and governance votes MUST NOT
establish research validity.
