# Funded discovery service

Researchers freely choose what to discover; pooled funding rewards verified
contributions under transparent, budget-conserving rules. Institutions finance
research freedom. Independent verification determines what the evidence supports.

The repository implements the authenticated command service, durable journal,
USDC escrow, certificate publication bridge, operator CLI and local EVM integration.
It does not launch a funded network by itself. Qualified operators, independently
reviewed policies and contracts, real USDC deposits, and qualified laboratories
are deployment requirements. The offline adversarial pilot remains available;
its synthetic declarations are never accepted as live certificate evidence.

## Implemented behavior

| Area | Service behavior |
|---|---|
| Open research | No bounty, buyer or institution is required in a submission. Participation uses registered signing identities and disclosed control clusters. |
| Separate budgets | Foundation, discovery, formalization, proof improvement, replication, tools and negative results have isolated solver/reproduction/review reserves. A protected foundational floor is mandatory. |
| Calibrated difficulty | Domain-specific reference-cost tiers, sample counts, outcomes, uncertainty and prior-art cutoff are committed before opening and approved by two independent assessors. Raw tokens, elapsed time and submission count do not increase an award. |
| Verification | Exact claim, artifact, profile and evidence roots bind authenticated XLMP history. Formal results require PoIR/checker reproduction; generalized certificates cannot stand in for formal checks. All authenticated dissent is considered. |
| Reproduction fees | A separate fixed reproduction fee is split among registered independent verifier payees. Signed assessor and appeal-review work is paid per reserved panel slot, including rejection or disagreement. A checking job/operator pair is charged once across submissions and rounds; settled work invoices cannot be replayed in escrow. |
| Collaboration | Owner-bound salted commitments and signed contributor consents precede assessment. One contribution group has one budget weight. Attribution corrections append a new decision and allocation commitment. |
| Simultaneous discoveries | Commitment time, then SubmissionID, defines priority. Independently assessed, control-disjoint teams committed before the primary disclosure within the fixed window share that group's award equally before each team's contributor split. They do not enlarge its weight. Independent-discovery evidence is required for every additional team. |
| Capacity | Category reserves, per-researcher commitment caps, contributor/verifier limits and assisted slots bound admission and settlement size. Queue timestamps, profiles, fees and deadlines are readable. Queue admission is not a finding of scientific importance. |
| Appeals | Signed evidence, grouping, eligibility, allocation, attribution and process appeals have funded capacity, fixed deadlines and fresh independent reviewers excluding the original reproduction operators. Process appeals cover admitted work awaiting assessment. Dissent is retained and holds allocation; a vote cannot repair a failed proof. |
| Expiry | Unresolved research gets no solver award. Completed work can settle during the published grace period; the contract refunds unused category funds. Two confirmed unpaid-expiry observations release abandoned award reservations for later rounds. |
| Transparency | Read APIs expose signed history, donor/admin restrictions, confirmed deposits, budgets, queues, plans and settlement observations. Multiple future funded rounds can be published; unconfirmed pledges do not become spending capacity. |

Weights and semantic grouping are qualified, signed assessments, not perfect
novelty or difficulty oracles. The existing adversarial report deliberately
demonstrates leakage from an undetected semantic duplicate. Qualification,
prior-art coverage and beneficial-control evidence must be independently reviewed.

## Configuration and identities

Build the API and CLI with `cargo build --locked -p xlemma-api -p xlemma-cli`.
The API requires its existing bearer token, XLMP signer/node mappings and durable
`XLEMMA_EVENT_LOG_PATH`. Set `XLEMMA_DISCOVERY_TRUST_PATH` to a public JSON file
conforming to `discovery-trust.schema.json`. Without it discovery commands fail
closed. The API fsyncs an accepted command before acknowledging it and replays
signatures and state transitions on restart.

The trust file pins a network name, EVM chain ID, escrow address, USDC asset and
principals. Use `eip155:<chain-id>/erc20:<token-address>` for the asset. Each
principal binds an Ed25519 public key, ResearcherID, control cluster, payout
address, credential commitment and roles. Roles are `administrator`, `researcher`,
`funding_observer`, `verifier`, `assessor`, `appeal_reviewer` and
`settlement_observer`. Verifier payees must be unique per verifier cluster.
Donor/control and qualification evidence remains a deployment responsibility.

Trust roots are signed into every command. Changing a trust file does not silently
rewrite historical authority: replay fails. Keep the historical file with the
journal. A trust migration requires a separately reviewed, explicit transition;
stop admission and revoke relevant onchain roles if a deployment key is compromised.

The `examples/discovery/service-*.json` files are reproducible development vectors
with historical timestamps and publicly reproducible test identities. Configure
deployment keys, dates, USDC and policies before opening an actual round.

## Operator workflow

1. Deploy `DiscoveryEvidenceRegistry` with a publication delay of at least one
   hour and `DiscoveryRoundEscrow` with the exact six-decimal USDC contract and
   that registry. Assign independent registry relay/watcher and escrow
   planner/reviewer/watcher/resolver roles and control clusters. The constructor
   does not grant all operating roles to the administrator.
2. Publish a `ServiceRoundPolicy`. Its settlement expiry must leave more than one
   day after the review deadline. Capacity must fit the escrow's 256-item batch.
   Derive the round and policy IDs with the CLI; create the same onchain round
   using the generated transaction. Category caps and dates are checked against
   the signed policy by the chain observer.
3. Submit the signed `create_round` command. Two independent assessors sign
   `approve_calibration` before opening. Fund each category onchain before
   `opens_at`; obtain two independent confirmed `observe_funding` commands for
   each deposit. The observer checks chain, canonical receipt/block, confirmations,
   escrow implementation hash, token, policy, dates, caps and exact funding log.
   Opening fails if confirmed funding does not cover every published reserve.
4. Researchers commit, reveal and obtain all contributor consents. Run the existing
   XLMP verification workflow and attach its accepted certificate MessageID.
   Two independent assessors sign the same assessment, including grouping,
   additional contribution, calibrated tier and reason commitments. Divergent
   assessments remain recorded and cannot allocate a reward.
5. File and resolve appeals within the published windows. Two matching independent
   reviewer decisions are needed; conflicting reviews remain unresolved. A
   revalidation remedy preserves old evidence and requires a new verified
   submission in a later funded round. It does not overwrite a certificate.
6. Finalize after the appeal window. The service rechecks every awarded
   certificate, including finality and quarantine, and calculates the whole batch.
   Alternatively expire unresolved work after its review deadline to create a
   completed-work-only plan during the settlement grace period.
7. Independent relays retrieve the evidence publication payload and reproduce
   its full authenticated evidence before publishing to the registry. They must
   monitor new dissent/quarantine and publish holds before settlement. Propose the
   exact API plan to escrow, obtain two independent plan approvals, then execute
   after the delay. The contract rechecks registry finality at execution.
8. Two settlement observers verify the exact `Settled` event and item commitment.
   Donors reclaim unused funds pro rata by category. If no payment executes,
   anyone can expire the onchain round after its deadline; two `observe_expiry`
   commands confirm that it expired unpaid and release the API's award reservation.

Example preparation and signing (commands print JSON; they do not broadcast):

```sh
target/debug/xlemma-cli discovery-evidence-inputs certificate-envelope.json
target/debug/xlemma-cli discovery-prepare trust.json policy.json \
  --submission submission.json --salt "$DISCOVERY_COMMIT_SALT"
target/debug/xlemma-cli discovery-sign trust.json command.json signing-seed.key \
  --nonce "unique-command-nonce" > signed-command.json
python3 scripts/discovery_chain.py create-round --trust trust.json --policy policy.json
python3 scripts/discovery_chain.py observe-funding --trust trust.json --policy policy.json \
  --transaction "$FUNDING_TX" --log-index "$FUNDING_LOG_INDEX" \
  --code-hash "$QUALIFIED_ESCROW_CODE_HASH" --donor-cluster "$DONOR_CLUSTER" \
  --administrator-cluster "$ADMINISTRATOR_CLUSTER"
python3 scripts/discovery_chain.py publish-evidence --trust trust.json --policy policy.json \
  --evidence publication.json --registry "$EVIDENCE_REGISTRY"
python3 scripts/discovery_chain.py propose-plan --trust trust.json --policy policy.json --plan plan.json
```

`XLEMMA_EVM_RPC_URL` is used for observations; HTTPS is required except on
loopback, redirects are disabled, and responses and execution time are bounded.
Each observer uses its own RPC and finality policy. `cast` performs ABI encoding.
Signing seeds are exactly 32 bytes encoded as hex in a private regular file;
Unix group/other access and symlinks are rejected. Wallet transaction signing
and broadcasting remain with the deployment's wallet or multisig.

## HTTP and evidence bindings

All routes use the existing API bearer authentication. Requests reject unknown
nested fields. Discovery commands form a separate signed namespace; the canonical
XLMP vocabulary is unchanged.

| Route | Purpose |
|---|---|
| `POST /v1/discovery/commands` | Accept a signed command after durable validation |
| `GET /v1/discovery/rounds` | Funding, policies, budgets, queues and settlement status |
| `GET /v1/discovery/rounds/{round_id}/queue` | Commitment-ordered admitted submissions |
| `GET /v1/discovery/rounds/{round_id}/history` | Append-only signed command history and reasons |
| `GET /v1/discovery/rounds/{round_id}/settlement` | Immutable settlement proposal |
| `GET /v1/discovery/rounds/{round_id}/submissions/{submission_id}/evidence` | Revalidated certificate publication payload |

For formal research, `formal_policy_bindings` maps the round's formal profile
to an exact approved PoIR consensus PolicyID. Its four evidence commitments use
`ArtifactId::derive`: formal statement from structural ClaimID; proof object
from `(ProofID, artifact_root)`; pinned toolchain from
`(environment_root, dependency_root)`; axiom inventory from `axiom_set_root`.
The settlement and publication policy digest is the actual PoIR policy.
Source text is never the final ClaimID.

For other research classes, the certificate profile must equal the submission
profile. Every authenticated reproduction observation must be present. Job
evidence strings already containing an ArtifactID are used directly; other root
strings are committed with `ArtifactId::derive`. Empirical profiles require
provenance, methods, calibration/instruments, uncertainty and independent
replication. A computational or formal certificate cannot satisfy that class.

## Tests and remaining operational gates

```sh
cargo test --locked --workspace
python3 scripts/simulate_discovery.py --check
cd contracts && forge test
# From the repository root, after building the CLI and contracts:
python3 scripts/test_discovery_evm.py
```

The EVM test creates a fresh local Anvil and mock six-decimal token, observes real
deposits, runs signed Rust commands, publishes certificates through independent
relay accounts, executes USDC-unit transfers and checks refunds. Its checker is
an explicit test double. API tests separately cover exact formal evidence and
authenticated XLMP ingress. The tests are not independent scientific validation.

Before public activation, obtain independent economic/security review, qualify
actual checker/operator and physics/laboratory profiles, establish outcome-neutral
registered-method service agreements and capacity, fund the deployed contracts,
and staff independent appeal/monitoring operations. Capacity rejection before
admission, operator outages beyond the settlement grace period, recovery of
already-spent USDC and trust migrations require published operating procedures;
the service does not invent scientific rejection or unlimited funding in those cases.
