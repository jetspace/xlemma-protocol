# XLIP-019 — Node network, service marketplace, reputation, and sortition

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Scope

The XLMP node network is the decentralized service plane for research
production, verification, review, storage, indexing, challenge monitoring,
finalization, and revalidation. It defines discovery, orders, matches,
eligibility, bonding, reputation evidence, and committee sortition without
making any provider, checker, payment rail, chain, or storage network part of
the xLemma protocol.

The node network MUST preserve this boundary:

```text
price + capacity + latency       → service routing
bond + role evidence + reputation → eligibility
future public randomness         → committee assignment
independent checker evidence      → PoIR outcome
```

Price, bond size, token ownership, and reputation MUST NOT weight a formal
vote or override a conflicting required checker family.

## 2. Native records

### 2.1 NodeServiceAdvertisement

A signed advertisement binds:

- NodeID, OperatorID, and conservative OperatorClusterID;
- UserCredentialID, OperatorCredentialID, NodeCredentialID, credential-chain
  root, coarse jurisdiction class, and delegation signature;
- monotonically increasing sequence and optional superseded advertisement;
- supported roles and XLMP endpoints;
- service kind, implementation/checker families, theories, and domains;
- hardware and optional trusted-execution attestation;
- maximum parallel jobs, total capacity, and currently available capacity;
- p50 and p95 observed latency;
- pricing model, unit, quantity scale, amount, asset, and decimals;
- ReputationSnapshotID, BondID, terms root, and validity interval.

AdvertisementID is content-derived without its signature or identifier field.
A new price, capacity statement, endpoint, reputation snapshot, bond, or terms
root creates a new advertisement. Supersession MUST NOT delete the historical
record.

Named providers such as ASTRA advertise the generic `research_prover` role.
Their product name appears only in `implementation_id` and adapter receipts.

### 2.2 Discovery and service orders

`NodeDiscoveryRequest` filters by service, roles, checker families, theory,
domain, available units, p95 latency, unit price, reputation dimensions, and
excluded operator clusters. `NodeDiscoveryResult` commits to the ordered
AdvertisementID set and generation time.

`ServiceOrder` is demand: it binds a job, requester, service/role, quantity,
maximum total price, delivery deadline, assurance constraints, terms, and
expiration. `ServiceMatch` is an immutable reservation against one exact
advertisement sequence. A match MUST NOT silently consume a later price or
capacity revision.

The reference matcher orders compatible advertisements by normalized unit
price, then p95 latency, then AdvertisementID. Arithmetic MUST be integer,
checked, and rounded upward for the buyer's total charge. Payment settlement
is a separate `PaymentAdapter` action and never implies successful research or
verification.

## 3. Multidimensional reputation

`NodeReputationVector` contains eight independent metrics:

1. formal accuracy;
2. availability;
3. latency performance;
4. novelty calibration;
5. challenge quality;
6. operator independence.
7. storage quality; and
8. integrity.

Each metric binds basis-point score, sample size, and evidence root.
Requirements apply minimum score and sample size per dimension. XLMP/1 defines
no weighted sum, universal rank, or transferable reputation token. A high
latency score cannot compensate for weak formal accuracy; a high formal score
cannot compensate for evidence of common control.

Metrics that do not apply to a role MAY carry zero samples, but a policy MUST
require the relevant dimensions for the assigned service. Updates create a new
`NodeReputationSnapshot` with explicit supersession. Honest dissent and
reproducible `FAIL` observations MUST NOT reduce formal accuracy merely for
disagreeing with a majority.

## 4. Bonding

`NodeBond` binds a NodeID, OperatorID, OperatorClusterID, independently valued asset amount, eligible
roles, slashing policy, escrow reference, lock deadline, status, and evidence
root. A bond is an eligibility and misconduct-security mechanism. Amount above
the policy threshold MUST NOT increase formal voting or sortition weight.

Slashing is limited to objective evidence such as equivocation, fabricated
execution, false artifact/custody binding, unauthorized key use, or a missed
reveal after commitment. Checker divergence, a valid `FAIL`, or an unpopular
novelty assessment is not itself slashable.

## 5. Committee sortition

### 5.1 Request commitment

Before randomness is known, `XLMP_SORTITION_REQUEST` MUST bind:

- SortitionID, JobID, PolicyID, and epoch;
- canonical eligible-set root;
- future randomness source, round, seed commitment, and proof reference;
- role counts, minimum bonds, per-dimension reputation requirements, and
  required checker families, minimum credential tier, maximum status-proof age,
  and qualifications;
- minimum distinct infrastructure providers and regions;
- excluded operator clusters.

The wire payload carries the exact canonical eligible-node records alongside
the request. Their derived root MUST equal `eligible_set_root`; this makes the
selection input independently available instead of trusting a coordinator's
private database.

The eligible set contains the exact credential chain and fresh non-revocation
status proof alongside AdvertisementID, BondID, ReputationSnapshotID, roles,
checker families, provider, region, and active state. Mutating one input changes
the eligible-set root.

### 5.2 Eligibility and ranking

A node is eligible for a slot only when all credential-chain, non-revocation,
role, active-bond, reputation, checker-family, exclusion, provider, and region
requirements pass. The
reference algorithm hash-ranks each eligible node using domain-separated
committed randomness plus SortitionID, JobID, PolicyID, epoch, role, slot,
NodeID, OperatorID, VerifiedUserID, and eligible-set root.

Each VerifiedUserID, OperatorID, and OperatorClusterID MAY fill at most one slot
in the committee. The selector MUST enforce the declared provider and region
diversity. Deterministic search
MUST find a valid independent assignment when a greedy first role would block
one; it MUST fail closed when no assignment exists.

Implementations MUST reject duplicate NodeIDs in an eligible set and MUST
publish deterministic resource bounds. The reference profile accepts at most
1,024 eligible nodes and 32 slots, explores at most 1,000,000 assignment
states, and returns a distinct fail-closed resource-limit outcome rather than
silently choosing a partial or lower-assurance committee.

### 5.3 Selection proof

`XLMP_COMMITTEE` publishes every member's role, slot, NodeID, VerifiedUserID,
OperatorID, OperatorClusterID, credential IDs, credential tier, credential-chain
root, AdvertisementID, BondID, ReputationSnapshotID, provider, region, and rank
hash plus a selection root. Any implementation can reproduce
the selection from the request, eligible records, randomness reveal, and
selection time.

Randomness proof validation is adapter-specific but MUST authenticate the
declared beacon/VRF round. A correct seed commitment alone does not prove that
the seed source was manipulation-resistant.

## 6. Required XLMP messages

The node-network profile adds:

```text
XLMP_NODE_ADVERTISE
XLMP_DISCOVERY_REQUEST
XLMP_DISCOVERY_RESPONSE
XLMP_SERVICE_ORDER
XLMP_SERVICE_MATCH
XLMP_SORTITION_REQUEST
XLMP_COMMITTEE
XLMP_REPUTATION
XLMP_BOND
```

These use the ordinary XLMP/1 envelope and MessageID/signature rules. Unknown
node-network messages or fields MUST fail closed under XLMP/1.

## 7. Privacy and market integrity

Advertisements SHOULD reveal only routing information needed before an order.
Confidential prompts, unpublished proof contents, private datasets, and secret
checker traces MUST remain out of public advertisements and logs. Orders MAY
use encrypted artifact references while still committing to exact roots.

Nodes MUST NOT advertise unavailable capacity, false hardware attestations,
fake checker families, or undisclosed common control. Discovery operators MUST
return reproducible results for the same snapshot and constraints. Censorship
resistance requires multiple discovery/index providers; no discovery server is
a consensus authority.

## 8. Conformance

A conforming node-network implementation MUST:

1. validate all node-network schemas and typed IDs;
2. preserve append-only advertisement, order, match, reputation, bond, and
   committee history;
3. reproduce the published AdvertisementID and XLMP MessageID vectors;
4. enforce price, capacity, latency, role, checker, reputation, and exclusion
   constraints without floating-point money arithmetic;
5. use bond/reputation only for eligibility, never formal vote weight;
6. enforce unique VerifiedUserIDs, OperatorIDs, operator clusters, and required provider/region diversity;
7. reproduce committee member ranks and the selection root;
8. keep service matching, payment settlement, and PoIR certification as
   separate state transitions and receipts.
9. validate credential chains under XLIP-020 and ensure one verified
   participant cannot gain independence by operating additional machines.
