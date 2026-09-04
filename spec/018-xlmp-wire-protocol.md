# XLIP-018 — XLMP/1 canonical protocol and wire specification

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Scope

XLMP is the xLemma Protocol. It defines immutable research objects, message
meaning, lifecycle transitions, evidence requirements, consensus inputs,
economic records, provenance, and rights bindings. It is independent of any
particular model provider, theorem prover, transport, payment rail, chain, or
storage system.

An implementation can conform to XLMP without using ASTRA, Lean, x402,
Ethereum, Base, Solana, IPFS, or any other named integration. An integration
MUST NOT redefine XLMP identity, verification, attribution, rights, or economic
conservation rules.

The protocol boundary is:

```text
XLMP defines research state and consensus; payment adapters transport value.
XLMP defines verification requirements; verifier adapters reproduce evidence.
XLMP defines artifact identity; storage adapters preserve and retrieve bytes.
XLMP defines finalizable state; chains may order or anchor that state.
```

## 2. Native object families

XLMP/1 recognizes the following native object families. Their canonical JSON
Schemas are part of the conformance surface.

| Plane | Native objects |
|---|---|
| Evidence graph | `ResearcherID`, `TheoryID`, `ClaimID`, `ProofID`, `LemmaCapsule`, `ContributionManifest`, `StatementAlignmentReceipt` |
| Identity | `VerifiedUserID`, `OperatorID`, `XLMP_UserCredential`, `XLMP_OperatorCredential`, `XLMP_NodeCredential`, `CredentialRevocation` |
| Node network | `NodeServiceAdvertisement`, `NodeDiscoveryRequest`, `ServiceOrder`, `ServiceMatch`, `NodeReputationSnapshot`, `NodeBond`, `CommitteeSortitionRequest`, `CommitteeSelection` |
| Verification | `VerificationJob`, `VerificationProfile`, `ObservationReceipt`, `ReproductionObservation`, `PoIRCertificate`, `ResearchVerificationCertificate`, `Challenge`, `QuarantineRecord` |
| Economic graph | `ComputeQuote`, `ComputeReceipt`, `ResearchCredit`, `ResearchVault`, `RevenueEvent`, `ImpactPoolAuthorization`, `DependencyDividend` |
| Rights/publication | `RightsManifest`, `License`, `PublicationRecord` |

Object identifiers are typed and domain-separated. A formal `ClaimID` MUST be
derived from the canonical elaborated formal type and its theory binding; it
MUST NOT be derived from source text alone. A correction or amendment creates a
new object with an explicit parent or supersession edge. Historical proof,
contribution, rights, receipt, challenge, quarantine, license, and publication
records MUST NOT be silently mutated.

## 3. Envelope

Every XLMP/1 wire message uses this logical envelope:

```text
protocol        = "XLMP"
version         = 1
message_id      = typed, content-derived MessageID
correlation_id  = optional prior MessageID
sender          = stable signer/principal identifier
sent_at         = RFC 3339 timestamp
message         = exactly one typed XLMP message
signature       = detached or encoded signature reference
```

The JSON media type is `application/x-xlmp+json;version=1`. Implementations MAY
also accept `application/json` when the same schema and canonicalization rules
are enforced.

`message_id` MUST commit to protocol name, major version, correlation ID,
sender, timestamp, message type, and payload using the canonical encoding and
domain separator declared by `MessageID`. The signature MUST cover the same
identity material plus the MessageID under the
`xlmp-envelope-signature-v1` domain through a deployment-approved signature
profile. Parsers MUST reject unsupported major versions, unknown required
fields, malformed typed identifiers, identity mismatches, empty senders, and
empty signatures.

Transport delivery receipts are separate from message signatures, proof
observations, payment receipts, and finality receipts.

## 4. Required message vocabulary

XLMP/1 defines these message discriminators:

| Message | Meaning |
|---|---|
| `XLMP_RESEARCHER` | Publish a sovereign researcher manifest without exposing issuer-private identity evidence |
| `XLMP_THEORY` | Publish a pinned theory, dependency, trust, checker, axiom, and canonical-encoding environment |
| `XLMP_CLAIM` | Publish a formal claim manifest and bind contribution and rights roots |
| `XLMP_CONTRIBUTION` | Reveal the content-derived contribution manifest committed by a claim |
| `XLMP_RIGHTS` | Reveal the non-mutating rights manifest committed by a claim |
| `XLMP_COMMIT` | Commit a researcher and job to a claim, policy, and reveal deadline |
| `XLMP_COMPUTE_QUOTE` | Publish a signed, expiring provider-neutral compute quote |
| `XLMP_PROOF_CANDIDATE` | Publish an untrusted proof candidate and immutable artifact binding |
| `XLMP_VERIFY_REQUEST` | Request reproduction under an exact challenge, theory, dependency, axiom, and verifier policy |
| `XLMP_OBSERVATION_COMMIT` | Commit an independent verifier before peer observations are visible |
| `XLMP_OBSERVATION_REVEAL` | Reveal the complete signed observation and commitment salt |
| `XLMP_CERTIFICATE` | Publish a PoIR certificate that references, but does not replace, observations |
| `XLMP_CHALLENGE` | Append bonded counterevidence against a certificate |
| `XLMP_QUARANTINE` | Append a fail-closed quarantine record bound to a certificate and optional challenge |
| `XLMP_FINALIZE` | Record completion of the challenge gate and finalization evidence |
| `XLMP_REVENUE` | Record realized external revenue bound to an independent settlement receipt |
| `XLMP_COMPUTE_RECEIPT` | Record provider-neutral metering, charge, context, implementation, and output evidence |
| `XLMP_RESEARCH_CREDIT` | Record credit issuance only when independently valued backing covers the issuance |
| `XLMP_RESEARCH_VAULT` | Publish a content-bound, solvent vault snapshot without redefining research validity |
| `XLMP_DEPENDENCY_DIVIDEND` | Record one capped, authorized, nonrecursive allocation from realized downstream revenue |
| `XLMP_LICENSE` | Publish a bounded license under the exact committed rights root and economic mode |
| `XLMP_CAPSULE` | Publish an immutable Lemma Capsule that binds research, evidence, rights, and economics |
| `XLMP_PUBLISH` | Publish the finalized claim, proof, certificate, artifact, rights, license, and location binding |
| `XLMP_AVAILABILITY` | Publish a node-signed custody and retention receipt for one content-addressed artifact |
| `XLMP_REVALIDATE` | Request fresh reproduction of a previously certified object |
| `XLMP_NODE_ADVERTISE` | Publish an append-only, expiring node capability/price/capacity advertisement |
| `XLMP_DISCOVERY_REQUEST` | Query advertisements using explicit service, implementation, capacity, latency, reputation, and independence constraints |
| `XLMP_DISCOVERY_RESPONSE` | Return a content-committed ordered set of matching advertisements |
| `XLMP_SERVICE_ORDER` | Publish a bounded service demand with price, delivery, role, and assurance constraints |
| `XLMP_SERVICE_MATCH` | Reserve advertised capacity and bind an order to a specific advertisement sequence |
| `XLMP_SORTITION_REQUEST` | Commit the eligible set, role requirements, diversity policy, and future randomness source |
| `XLMP_COMMITTEE` | Publish an exactly reproducible committee selection with per-member rank proofs |
| `XLMP_REPUTATION` | Publish a superseding multidimensional, evidence-backed node reputation snapshot |
| `XLMP_BOND` | Publish independently settled eligibility collateral and its slashing policy |
| `XLMP_USER_CREDENTIAL` | Publish a pseudonymous verified-participant attestation |
| `XLMP_OPERATOR_CREDENTIAL` | Delegate accountable roles from a verified participant to an operator |
| `XLMP_NODE_CREDENTIAL` | Delegate exact roles and a public key from an operator to a node |
| `XLMP_CREDENTIAL_REVOCATION` | Append an effective revocation for an exact credential |
| `XLMP_SOVEREIGNTY` | Publish a content-derived Researcher Sovereignty Bundle with all seven protections |
| `XLMP_PORTABILITY` | Publish an independently reconstructable researcher export and storage map |
| `XLMP_RESIDUAL_RIGHT` | Publish bounded, nonexclusive economic participation in named settled revenue |
| `XLMP_ECONOMIC_CONSTITUTION` | Select Commons, Reciprocal, Commercial Artifact, or Sponsored Challenge rules |
| `XLMP_ECONOMIC_COMPLIANCE` | Publish a content-derived status for explicit obligations and settlement without changing research validity |
| `XLMP_VERIFICATION_PROFILE` | Publish required evidence and independent-reproduction policy for one verification class |
| `XLMP_REPRODUCTION_OBSERVATION` | Reveal a node-signed, credential-bound non-formal or hybrid reproduction receipt after its exact `XLMP_OBSERVATION_COMMIT` |
| `XLMP_RESEARCH_CERTIFICATE` | Publish the deterministic multi-operator reproduction outcome for any verification profile |
| `XLMP_COMPUTE_COOPERATIVE` | Publish user-owned cooperative membership, nodes, treasury, and control evidence |
| `XLMP_CAPTURE_DASHBOARD` | Publish eight-layer concentration and capture-resistance evidence |
| `XLMP_NODE_WORK` | Publish completed work and externally settled node revenue evidence |
| `XLMP_NODE_EXPOSURE` | Bind maximum certificate exposure to active bond coverage |
| `XLMP_MISCONDUCT` | Publish objectively provable misconduct and a bounded slash against a bond snapshot |
| `XLMP_GOVERNANCE_PROPOSAL` | Publish an approved three-chamber proposal with immutable constitution, simulation, timelock, and fork/exit plan |
| `XLMP_CREDENTIAL_EVIDENCE` | Publish a selectively disclosed, non-revoked, multi-issuer eligibility evidence set |

Proof candidates MUST be labeled as candidates. `XLMP_CERTIFICATE` MUST NOT be
emitted solely from a prover response. `XLMP_OBSERVATION_REVEAL` MUST bind the
same job, receipt, node, operator cluster, commitment, and commit time as its
corresponding `XLMP_OBSERVATION_COMMIT`.

Node-network messages are specified further in XLIP-019 and identity messages
in XLIP-020. Sovereignty, residual rights, portability, and verification
profiles are specified in XLIP-022. Price, bond size, credential tier, and
reputation MUST remain service-routing and eligibility inputs; none is formal
vote weight and none can override checker evidence.

## 5. Canonical lifecycle

The protocol-level happy path is:

```text
CLAIM
  → COMMIT
  → FORMALIZE
  → PROVE
  → REPRODUCE
  → CERTIFY
  → CHALLENGE
  → FINALIZE
  → PUBLISH
  → REUSE
  → REWARD
  → REVALIDATE
```

`CHALLENGE` represents the mandatory challenge gate, including the case where
the window expires without a submitted challenge. Implementations MAY expose
more granular local job states, but MUST NOT use them to bypass a protocol
gate.

Negative and recovery transitions are fail-closed:

- divergent or insufficient reproduction MAY transition to `QUARANTINED` or
  `REJECTED`, never directly to `CERTIFY`;
- an upheld or unresolved challenge MAY transition to `QUARANTINED` or
  `REJECTED`;
- a published object MAY transition to `QUARANTINED`, `SUPERSEDED`, or
  `REVALIDATE` through a new append-only record;
- revalidation MUST return through `REPRODUCE` (or `FORMALIZE` followed by the
  proof path when repair is required) and fresh evidence; it MUST NOT return
  directly to `PUBLISH` or overwrite the prior certificate or receipts.

Formal checker-family divergence MUST NOT be resolved by token vote, stake
weight, popularity, or majority count.

## 6. Adapter contracts

### 6.1 Research prover

A `ResearchProver` exposes provider-neutral `formalize`, `propose`, `prove`,
`repair`, and `explain` operations. ASTRA is one implementation. Prover output
is untrusted until independently reproduced and MUST carry a provider-neutral
`ComputeReceipt` plus immutable artifact roots.

### 6.2 Verifier

A `VerifierAdapter` reproduces an exact formal `VerificationJob` and produces
an `ObservationReceipt`. A `ReproductionAdapter` performs the equivalent
profile-bound operation for computational, statistical, simulation, empirical,
and hybrid evidence and returns a `ReproductionObservation`. Lean is the
default XLMP/1 formal backend. Coq, Rocq,
Isabelle, HOL, Agda, future proof systems, and independent checker families MAY
implement the interface under explicit theory and policy identifiers.

Supporting another formal system MUST NOT weaken the policy requiring exact
reproduction or allow a proof producer to certify itself. Cross-system
equivalence requires explicit formal evidence under a declared relation; it is
not inferred from text similarity.

### 6.3 Payment

A `PaymentAdapter` authorizes and settles an XLMP compute or access obligation.
x402, stablecoin transfers, native-chain payments, fully backed research
credits, grants, bounty escrow, and institutional invoicing are peer adapter
options.

Payment success MUST NOT imply proof validity. Payment failure MUST NOT rewrite
an existing research observation. Verifier compensation MUST be based on
reproducible execution under the service agreement and MUST NOT depend solely
on returning `PASS`.

### 6.4 Transport

HTTP, libp2p, WebSocket, x402-protected HTTP, and chain event streams MAY carry
the same XLMP envelope. A transport MUST preserve the exact canonical bytes or
provide a reversible encoding whose decoded envelope has the same MessageID.

The XLMP/1 reference binary frame is `uint32_be(payload_length) || payload`,
where `payload` is one canonical RFC 8785 envelope and `payload_length` is from
1 through 1,048,576 bytes. A decoder MUST reject truncated headers, declared
length mismatches, trailing bytes, oversized frames, non-canonical JSON, and an
envelope whose content-derived MessageID does not match. The length prefix is a
transport concern and is not part of MessageID derivation.

The reference outbound HTTP adapter accepts only allowlisted HTTPS endpoints,
does not follow redirects, bounds response time and size, and requires the
recipient to echo an intact canonical envelope with the same MessageID before
issuing a signed, content-derived transport receipt. Transport authentication
material MUST NOT be copied into that receipt.

### 6.5 Finality and storage

Chains MAY anchor state roots, ordering, settlement, and challenge deadlines.
They MUST NOT claim that transaction inclusion alone establishes mathematical
validity, novelty, attribution, or legal rights.

Storage adapters MAY use IPFS, content-addressed object stores, local archival
systems, or future networks. Availability evidence remains distinct from proof
validity and rights evidence.

An indexer MAY replay canonical messages into the reference
`ProtocolProjection`. The projection MUST reject duplicate native objects and
orphaned research, certificate, publication, economic, availability, or
supersession references. Its state root commits to accepted MessageIDs; it is
an auditable view and MUST NOT replace the signed source messages.

## 7. Economic conservation

An XLMP `ResearchCredit` issuance MUST bind independently valued backing, a
valuation policy, and a backing reference. The conservative backing value MUST
not be lower than issued credit units. Expected revenue, an unverified lemma,
or unrealized token appreciation is not backing.

An `XLMP_REVENUE` event MUST reference finalized external settlement evidence.
Its content-derived identity binds the related-party disclosure, settlement
receipt, amounts, deductions, realization time, and evidence root. The
reference upstream allocator rejects related-party events; they remain visible
for audit rather than being misrepresented as arm's-length demand.
A `DependencyDividend` MUST reference a dependency used by the final proof,
conservative compute-savings evidence, realized downstream net revenue, a
prescriptive economic-policy root, an eligible economic edge, a fixed pool,
and an explicit cap. A formal dependency alone MUST NOT create payment.
Recursive or uncapped royalties are non-conforming. Commons capsules MUST
default mandatory upstream payments to zero; compute savings are an impact
signal rather than a precise invoice.

## 8. Rights and attribution

Token transfers and payment settlement MUST NOT rewrite origin attribution,
formal validity, contribution history, or a rights manifest. A `License` grants
only the scope supported by its referenced rights manifest and legal context.
A capsule MUST distinguish its nontransferable origin, rights in actual
artifacts/contracts, and any explicitly bounded economic participation. It MUST
select one of `commons`, `reciprocal`, `commercial_artifact`, or
`sponsored_challenge`.
Economic compliance is a separate certificate plane. A compliant payment
record MUST NOT change a failed, divergent, unchecked, or quarantined research
status; a mathematically valid claim MUST NOT erase an unsatisfied economic
obligation.
A `StatementAlignmentReceipt` is independent of formal status and MUST bind the
formal claim to the reviewed informal claim and presentation.
A `PublicationRecord` binds a specific claim, proof, certificate, artifact,
rights root, and publication location; later corrections use supersession.

## 9. Versioning and conformance

Unknown XLMP major versions MUST be rejected unless explicitly negotiated.
Minor, backwards-compatible application profiles MAY add optional fields only
through a versioned extension mechanism defined by a later XLIP; receivers MUST
NOT silently ignore unknown required fields.

An XLMP/1 implementation is conforming only if it:

1. validates the canonical schemas and typed identifiers;
2. reproduces MessageID derivation test vectors;
3. enforces lifecycle transitions and fail-closed negative paths;
4. preserves append-only historical evidence;
5. keeps proof, payment, availability, transport, rights, and finality receipts
   distinct;
6. preserves the consensus and economic invariants in XLIP-000;
7. discloses which adapter implementations and policy roots were used.
8. preserves append-only node advertisements, orders, matches, reputation
   snapshots, bonds, and committee evidence.

The reference Rust crate is `xlemma-xlmp`. The JSON Schema authority is
`schemas/xlmp-envelope.schema.json`; neither artifact is privileged over the
normative rules in this specification.
