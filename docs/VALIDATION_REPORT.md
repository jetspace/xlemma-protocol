# Validation report — XLMP/1 source snapshot

As-of date: 2026-09-04.

## Scope

This report records checks actually executed for the XLMP/1 security-hardening
and conservation-law refinement. It distinguishes local evidence from checks
delegated to CI and from production assurance that the reference
implementation does not claim.

## Executed successfully

The repository validator completed successfully after regeneration of the
deterministic source manifest. It checked:

- required source, specification, schema, example, contract, Lean, LaTeX, and
  documentation files;
- JSON syntax, JSON Schema Draft 2020-12 validity, and example conformance,
  including the XLMP/1 envelope, node advertisement/reputation/bond vectors,
  pseudonymous user/operator/node credential and revocation vectors, the
  published credential-bound eligible-set/sortition/committee vector, and all
  provider-neutral native-object schemas, including statement alignment,
  signed protocol success calibration, capsule modes, and impact-pool
  authorization;
- TOML and YAML syntax, OpenAPI 3.1 operations, routes, response maps, external
  references, fragments, and local reference resolution;
- revenue and contributor-share conservation, checker-root agreement, and
  verified-user, operator, operator-cluster, provider, region, checker-family diversity, constrained service
  matching, advertisement supersession, and sortition input commitments;
- documented protocol invariants and absence of obvious embedded API keys or
  private-key blocks;
- complete `MANIFEST.sha256` file coverage and source digests.

The Rust 1.82 workspace was resolved with the committed `Cargo.lock`. The
following commands completed successfully:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

The Rust suite contains 106 unit/async-test declarations. It exercises RFC 8785
canonicalization and safe-integer rejection, strict XLMP typed ingress,
content-bound receipt identity, Ed25519 signature and signer/NodeID binding,
statement-alignment identity and signature-verifier boundaries, signed
protocol success calibration, exclusion of uncalibrated provider offers,
signed impact-pool authorization and exact revenue-event binding,
evidence/economic graph separation,
content-derived impact evidence, checked fixed-point quote/allocation math and
deterministic equal-cost routing,
Open Commons economic constraints, message and advertisement
identity/mutation rejection, append-only HTTP and
node-market records, asset-compatible capacity matching, eight-dimensional
reputation gates, pseudonymous credential integrity, append-only revocation,
reproducible future-randomness committee sortition, duplicate node, operator,
and verified-participant exclusion, lifecycle gates and revalidation, PoIR divergence,
backed credits, revenue conservation, impact-pool caps, provider
adapters, bounded and symlink-safe storage, hardened Lean/ASTRA adapter
boundaries, and x402-to-XLMP binding.

`cargo audit` completed against a current local RustSec advisory database with
`--deny warnings`: 237 locked dependencies were scanned against 1,239
advisories with no vulnerable packages reported.

`scripts/simulate_economics.py` completed successfully for backing,
authorization, settlement, refund, solvency, revenue allocation, compounding,
and conservative compute-impact allocations. Python bytecode compilation also
completed successfully for the repository scripts.

Foundry 1.4.0 formatting, build-size, unit, fuzz, and invariant checks completed
successfully against the immutable OpenZeppelin and forge-std revisions listed
in `contracts/README.md`. Eight suites ran 32 tests; the stateful vault invariant
ran 256 sequences of 32,768 calls. The tests include vault-only credit minting,
exact asset transfers, content-derived proof/certificate/bounty identifiers,
artifact-bound bounty release, certificate invalidation, revenue replay
namespacing, and ResearcherID registration isolation.

`docker compose config --quiet` completed successfully with explicit test
secrets. The committed runtime and service image references use immutable
digests, the API binds only to loopback by default, and the container profile is
read-only, non-root, capability-free, and protected by `no-new-privileges`.

The snapshot contains 13 Rust crates, 54 JSON schemas, 22 numbered
specifications, 9 Solidity contracts, and 8 Solidity test suites. These source
counts establish repository coverage; they do not substitute for an audit.

## Not executed locally

This environment did not include Lean/Lake. The following checks are therefore
delegated to the included CI workflow and are not claimed as locally executed:

- Lean package build, official kernel replay, nanoda replay, and axiom audit.

The following production activities were also outside this validation scope:

- live ASTRA/provider calls or calibration;
- live decentralized service discovery, order matching, or node-reputation assessment;
- production beacon/VRF proof authentication and decentralized eligible-set publication;
- production credential issuer, delegation-signature/key-resolution, private uniqueness, and revocation-accumulator integrations;
- live x402, stablecoin, grant, escrow, invoice, IPFS, or chain integration;
- a second independent XLMP implementation and cross-implementation vectors;
- independent security, cryptographic, economic, smart-contract, or legal audit;
- static analysis, symbolic execution, chain-fork testing, or a public security contest.

## Release status

This remains an architectural reference implementation and prototype. The
XLMP/1 provider-neutral boundary, twenty-five canonical messages, lifecycle,
first-class identity/credential and node-network planes, native records,
schemas, and adapters are implemented
and locally checked. The
unchecked gates in `docs/PRODUCTION_CHECKLIST.md`, including independent
checker deployment and external audits, remain mandatory before real-value
production use.

## Reproduction commands

```bash
python3 scripts/validate_repo.py
python3 scripts/simulate_economics.py
python3 -m compileall -q scripts

cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo audit --deny warnings

cd lean && lake build
cd ../contracts && forge fmt --check && forge build --sizes && forge test -vvv
```

Use `docs/PRODUCTION_CHECKLIST.md` as the deployment gate rather than treating
this report as production certification.
