# xLemma use-case simulation report

Generated: 2026-09-04T20:00:00Z

> This is deterministic reference-implementation evidence, not a security audit,
> independent mathematical verification, live settlement attestation, or production certification.

## Executive result

**11/11 documented journeys passed** across **13/13 executable gates** and **162 Rust tests**.

| ID | Documented journey | Result | Executable gates |
|---|---|---:|---|
| UC-01 | Researcher onboarding | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `trust_policy`, `portable_exit` |
| UC-02 | Create and verify a lemma | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `trust_policy`, `deterministic_bundle`, `formal_poir` |
| UC-03 | Pay with researcher credits | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `economic_conservation` |
| UC-04 | Earn from a verified result | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `economic_conservation`, `economic_compliance`, `compute_impact` |
| UC-05 | Support a decentralized researcher | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `economic_compliance` |
| UC-06 | Operate a prover node | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `compute_quote` |
| UC-07 | Operate a checker node | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `formal_poir` |
| UC-08 | Challenge a certificate | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity` |
| UC-09 | Reuse an upstream lemma | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `compute_impact` |
| UC-10 | Publish a negative result | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `generalized_verification` |
| UC-11 | Correct or supersede work | **PASS** | `repository_structure`, `rust_workspace`, `durable_protocol_history`, `binary_transport_identity`, `portable_exit` |

## Executable gate results

### repository_structure — Repository and schema integrity

- Result: **PASS**
- Evidence: schemas, examples, source tree, invariants, and manifest passed
- Command: `python3 scripts/validate_repo.py --skip-simulation-report`

### rust_workspace — Rust workspace regression suite

- Result: **PASS**
- Evidence: all discovered Rust tests passed
- Command: `cargo test --locked --workspace --quiet`

### durable_protocol_history — Durable XLMP history and restart recovery

- Result: **PASS**
- Evidence: signed XLMP ingress was fsynced, reopened, hash-chain verified, and retrieved after restart
- Command: `cargo test --locked -p xlemma-api tests::accepted_xlmp_message_survives_api_restart -- --exact`

### binary_transport_identity — Canonical non-HTTP XLMP framing

- Result: **PASS**
- Evidence: the published HTTP envelope survived canonical binary framing with the identical MessageID
- Command: `cargo test --locked -p xlemma-xlmp framing::tests::binary_transport_round_trip_preserves_message_identity -- --exact`

### trust_policy — Trust-policy and axiom gate

- Result: **PASS**
- Evidence: content-derived policy and axiom profile accepted exact evidence
- Command: `cargo run --quiet -p xlemma-cli -- verify-trust examples/no-arbitrage/trust-policy-registry.json examples/no-arbitrage/theory.json examples/no-arbitrage/proof.json examples/no-arbitrage/proof-trust-evidence.json`

### formal_poir — Formal PoIR reproduction

- Result: **PASS**
- Evidence: 3 independent observations: 2 Lean kernel + 1 nanoda
- Command: `cargo run --quiet -p xlemma-cli -- evaluate-consensus examples/no-arbitrage/policy.json examples/no-arbitrage/observations.json`

### generalized_verification — Computational-profile reproduction

- Result: **PASS**
- Evidence: computational profile reached its independent reproduction threshold
- Command: `cargo run --quiet -p xlemma-cli -- evaluate-reproduction examples/no-arbitrage/computational-verification-profile.json examples/no-arbitrage/computational-verification-job.json examples/no-arbitrage/computational-observations.json`

### portable_exit — Researcher portability and exit

- Result: **PASS**
- Evidence: portable exit manifest reconstructed across independent locations
- Command: `cargo run --quiet -p xlemma-cli -- verify-portability examples/no-arbitrage/portability-manifest.json`

### economic_compliance — Economic-constitution compliance

- Result: **PASS**
- Evidence: economic compliance validated without changing research validity
- Command: `cargo run --quiet -p xlemma-cli -- verify-economic-compliance examples/no-arbitrage/economic-constitution.json examples/no-arbitrage/economic-compliance-certificate.json`

### economic_conservation — Credit and revenue conservation

- Result: **PASS**
- Evidence: backing, settlement, refund, revenue, compounding, and impact caps conserved
- Command: `python3 scripts/simulate_economics.py`

### compute_quote — Calibrated compute-market quote

- Result: **PASS**
- Evidence: six service offers routed using independently signed success calibration
- Command: `cargo run --quiet -p xlemma-cli -- quote examples/no-arbitrage/compute-offers.json examples/no-arbitrage/expected-work.json examples/no-arbitrage/protocol-success-estimates.json --trusted-estimator ed25519:6kpsY-KcUgq-9VB7Ey7F-ZVHdq6-vnuSQh7qaRRG0iw --deadline 2027-09-04T20:00:00Z --quoted-at 2026-09-04T20:00:00Z`

### compute_impact — Bounded compute-impact allocation

- Result: **PASS**
- Evidence: 671000 minor units remained below revenue and authorized-pool caps
- Command: `cargo run --quiet -p xlemma-cli -- compute-impact examples/no-arbitrage/compute-savings-evidence.json examples/no-arbitrage/compute-savings-policy.json examples/no-arbitrage/downstream-net-revenue.json examples/no-arbitrage/impact-pool-authorization.json --trusted-authorizer ed25519:_RckOFqgx1tk-3jNYC-h2ZH96_drE8WO1wLqyDXp9hg`

### deterministic_bundle — Deterministic artifact bundle

- Result: **PASS**
- Evidence: complete bundle object matched the published deterministic vector
- Command: `cargo run --quiet -p xlemma-cli -- pack examples/deterministic-bundle examples/deterministic-bundle/inputs.json --lean-toolchain leanprover/lean4:v4.33.1 --dependency-lock-hash blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --source-commit vector-1 --build-image-digest sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb --created-at 2026-09-04T12:00:00Z`

## End-to-end journey traces

Each ordered trace is accepted only when every linked executable gate and named regression test passes. The trace demonstrates reference-implementation conformance; external services listed in the limitations remain simulated adapters.

### UC-01 — Researcher onboarding

- Protocol path: identity → sovereignty bundle → backed vault → policy selection
- Ordered trace: 1. identity [PASS] → 2. sovereignty bundle [PASS] → 3. backed vault [PASS] → 4. policy selection [PASS]
- Simulated outcome: The researcher obtains portable protocol state and mints credits only against backing.
- Fail-closed invariant: Missing sovereignty protections or insufficient backing is rejected.
- Regression evidence: `sovereignty::tests::sovereignty_bundle_requires_every_durable_right`, `credit::tests::credits_remain_fully_backed_through_usage_settlement`

### UC-02 — Create and verify a lemma

- Protocol path: CLAIM → COMMIT → FORMALIZE → PROVE → REPRODUCE → CERTIFY
- Ordered trace: 1. CLAIM [PASS] → 2. COMMIT [PASS] → 3. FORMALIZE [PASS] → 4. PROVE [PASS] → 5. REPRODUCE [PASS] → 6. CERTIFY [PASS]
- Simulated outcome: Three independent observations reproduce one exact artifact under the selected trust policy.
- Fail-closed invariant: A producer cannot self-certify, and missing or divergent evidence cannot advance.
- Regression evidence: `trust::tests::registered_policy_accepts_exact_fail_closed_evidence`, `formal::tests::gold_quorum_certifies_only_unanimous_exact_reproduction`

### UC-03 — Pay with researcher credits

- Protocol path: deposit backing → mint Rᵢ → authorize maximum → settle actual → unlock remainder
- Ordered trace: 1. deposit backing [PASS] → 2. mint Rᵢ [PASS] → 3. authorize maximum [PASS] → 4. settle actual [PASS] → 5. unlock remainder [PASS]
- Simulated outcome: Only consumed credits burn and release an equal amount of neutral backing.
- Fail-closed invariant: Over-authorization, replay, or under-collateralization fails without partial mutation.
- Regression evidence: `credit::tests::credits_remain_fully_backed_through_usage_settlement`, `credit::tests::forged_or_cloned_authorization_cannot_unlock_another_reservation`

### UC-04 — Earn from a verified result

- Protocol path: settled external revenue → costs/refunds/reserves → bounded waterfall → cash + compounding
- Ordered trace: 1. settled external revenue [PASS] → 2. costs/refunds/reserves [PASS] → 3. bounded waterfall [PASS] → 4. cash + compounding [PASS]
- Simulated outcome: Net revenue is conserved and auto-compounded credits receive matching vault backing.
- Fail-closed invariant: Token appreciation, unsettled revenue, or an unauthorized impact signal cannot fund payouts.
- Regression evidence: `revenue::tests::revenue_is_conserved_across_waterfall_and_creator_rounding`, `dividend::tests::compute_signal_without_economic_authorization_cannot_pay`

### UC-05 — Support a decentralized researcher

- Protocol path: grant / bounty / compute pre-purchase / public-goods support → typed funding receipt
- Ordered trace: 1. grant / bounty / compute pre-purchase / public-goods support [PASS] → 2. typed funding receipt [PASS]
- Simulated outcome: Funding has an explicit rail, backing source, restrictions, and settlement evidence.
- Fail-closed invariant: Self-issued inflation and vague passive-profit promises do not qualify as funding.
- Regression evidence: `funding::tests::protocol_fees_conserve_value_and_fund_all_infrastructure`, `funding::tests::inflation_without_settlement_cannot_be_funding`

### UC-06 — Operate a prover node

- Protocol path: advertise capacity → calibrated quote → candidate generation → compute receipt
- Ordered trace: 1. advertise capacity [PASS] → 2. calibrated quote [PASS] → 3. candidate generation [PASS] → 4. compute receipt [PASS]
- Simulated outcome: A provider-neutral prover can earn for bounded work and return a candidate artifact.
- Fail-closed invariant: The proof producer is excluded from independent reproduction of its own candidate.
- Regression evidence: `protocol::tests::producer_cannot_count_as_an_independent_reproducer`, `tests::endpoint_must_be_https_and_explicitly_allowlisted`

### UC-07 — Operate a checker node

- Protocol path: credential chain → advertisement → sortition → exact execution → commit/reveal → work payment
- Ordered trace: 1. credential chain [PASS] → 2. advertisement [PASS] → 3. sortition [PASS] → 4. exact execution [PASS] → 5. commit/reveal [PASS] → 6. work payment [PASS]
- Simulated outcome: Distinct verified users, operators, clusters, providers, regions, and checker families reproduce the job.
- Fail-closed invariant: More machines under common control do not create more independence or truth weight.
- Regression evidence: `committee::tests::selection_is_reproducible_and_operator_independent`, `formal::tests::multiple_nodes_under_one_verified_user_are_not_independent_observations`

### UC-08 — Challenge a certificate

- Protocol path: challenge → counterevidence → expanded reproduction → dismiss / quarantine / reject
- Ordered trace: 1. challenge [PASS] → 2. counterevidence [PASS] → 3. expanded reproduction [PASS] → 4. dismiss / quarantine / reject [PASS]
- Simulated outcome: A valid challenge can move the object into fail-closed quarantine and later revalidation.
- Fail-closed invariant: Checker divergence is never resolved by a 2-to-1 majority or by slashing honest dissent.
- Regression evidence: `formal::tests::checker_disagreement_is_divergent_not_majority_vote`, `tests::divergent_reproduction_can_fail_closed`, `capture::tests::honest_divergence_is_not_a_slashable_offense`

### UC-09 — Reuse an upstream lemma

- Protocol path: final proof dependency → separate economic edge → settled revenue → bounded nonrecursive pool
- Ordered trace: 1. final proof dependency [PASS] → 2. separate economic edge [PASS] → 3. settled revenue [PASS] → 4. bounded nonrecursive pool [PASS]
- Simulated outcome: Eligible upstream contributors can receive a capped allocation from an authorized pool.
- Fail-closed invariant: Formal dependency alone creates no debt; stuffing, cycles, dust, and recursive charging are blocked.
- Regression evidence: `upstream::tests::one_pool_is_bounded_clustered_and_conserved`, `protocol::tests::formal_dependency_without_an_economic_edge_never_creates_payment`

### UC-10 — Publish a negative result

- Protocol path: failed or inconclusive work → attributable evidence artifact → commons/public-goods funding
- Ordered trace: 1. failed or inconclusive work [PASS] → 2. attributable evidence artifact [PASS] → 3. commons/public-goods funding [PASS]
- Simulated outcome: Useful negative evidence remains publishable and fundable without a false validity badge.
- Fail-closed invariant: Negative results cannot be relabeled as certified proofs or used to mint unbacked credits.
- Regression evidence: `funding::tests::negative_results_are_commons_not_market_funding`

### UC-11 — Correct or supersede work

- Protocol path: new immutable object → AMENDS / CORRECTS / SUPERSEDES edge → revalidation
- Ordered trace: 1. new immutable object [PASS] → 2. AMENDS / CORRECTS / SUPERSEDES edge [PASS] → 3. revalidation [PASS]
- Simulated outcome: The old artifact, attribution, receipts, and correction lineage remain reconstructable.
- Fail-closed invariant: A superseded record cannot silently return to published state or overwrite history.
- Regression evidence: `state::tests::supersession_is_append_only_and_cannot_restore_old_publication_state`, `marketplace::tests::order_book_preserves_superseded_advertisements`

## Integration defects found and corrected

1. The formal-policy schema and example allowed a zero requirement for an optional checker family, while the Rust validator rejected it. The validator now requires at least one positive family and permits explicit optional zero entries.
2. The published formal observation vector still contained illustrative receipt IDs, evidence roots, and commitments. The CLI now prepares arrays of content-derived observations, and the vector was regenerated so direct PoIR evaluation succeeds.

## Limitations and production blockers

- Lean/Lake, official kernel replay, nanoda replay, and the hostile proof corpus were not executed by this harness.
- Formal consensus simulation consumes structurally valid content-derived observations; production ingress must additionally authenticate node signatures, credentials, committee assignments, and non-revocation proofs.
- ASTRA/model calls, x402 or stablecoin settlement, chains, storage providers, credential issuers, and randomness beacons are represented by deterministic adapters and fixtures rather than live external services.
- No independent implementation, clean-room bundle reproduction, cryptographic audit, smart-contract audit, sandbox audit, economic audit, or legal review is claimed.
- Passing scenarios demonstrate internal consistency of the checked snapshot, not theorem novelty, informal-statement alignment, commercial value, or production safety.

## Reproduce

```bash
python3 scripts/simulate_use_cases.py \
  --generated-at 2026-09-04T20:00:00Z \
  --markdown docs/USE_CASE_SIMULATION_REPORT.md \
  --json reports/use-case-simulation.json
```
