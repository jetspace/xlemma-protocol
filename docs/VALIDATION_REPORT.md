# Validation report — XLMP/1 source snapshot

As-of date: 2026-09-04.

## Scope

This report records checks actually executed for the XLMP/1 researcher-
sovereignty, anti-capture, security-hardening, and conservation-law refinement.
It distinguishes local evidence from checks
delegated to CI and from production assurance that the reference
implementation does not claim.

## Executed successfully

The repository validator completed successfully after regeneration of the
deterministic source manifest. It checked:

- required source, specification, schema, example, contract, Lean, LaTeX, and
  documentation files;
- JSON syntax, JSON Schema Draft 2020-12 validity, and example conformance,
  including the 53-message XLMP/1 envelope, node advertisement/reputation/bond vectors,
  pseudonymous user/operator/node credential and revocation vectors, the
  published credential-bound eligible-set/sortition/committee vector, and all
  provider-neutral native-object schemas, including sovereignty bundles,
  portability, bounded residual rights, economic constitutions and compliance,
  all six verification profiles, generalized reproduction, capture dashboards,
  constitutional governance, funding rails, statement alignment, signed
  protocol success calibration, capsule modes, and impact-pool authorization;
- the content-derived trust-policy registry, axiom profile, exact proof-trust
  evidence, canonical registry root, and deterministic artifact-bundle vector;
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

The Rust suite contains 173 unit/async-test declarations. It exercises RFC 8785
canonicalization and safe-integer rejection, strict XLMP typed ingress,
validated Lean environment export ingestion, theory/toolchain/encoding
matching, domain-separated ClaimID/ProofID derivation, and mandatory
verifier-request/export identity, dependency-root, and axiom binding,
content-bound native protocol and receipt identity, Ed25519 signature and signer/NodeID binding,
statement-alignment identity and signature-verifier boundaries, signed
protocol success calibration, exclusion of uncalibrated provider offers,
signed impact-pool authorization and exact revenue-event binding,
evidence/economic graph separation,
content-derived impact evidence, checked fixed-point quote/allocation math and
deterministic equal-cost routing,
Commons and Reciprocal economic constraints, message and advertisement
identity/mutation rejection, append-only HTTP and
node-market records, deterministic native lifecycle projection, canonical
non-HTTP binary framing with exact MessageID preservation, allowlisted HTTPS
transport, durable hash-chained API restart recovery, native read projections,
and tamper rejection,
asset-compatible capacity matching, eight-dimensional
reputation gates, pseudonymous credential integrity, append-only revocation,
reproducible future-randomness committee sortition, duplicate node, operator,
and verified-participant exclusion, lifecycle gates and revalidation, PoIR divergence,
credential-bound generalized commit-reveal and authenticated-certificate ingress,
all six research verification profiles, researcher portability and company-disappearance recovery,
economic-compliance/validity separation, multi-issuer concentration limits,
multi-chamber governance, compute cooperatives and capture-resistance scoring,
work-backed node revenue and bond exposure,
backed credits, revenue conservation, impact-pool caps, provider adapters,
actual-use x402 settlement and replay/mutation rejection, immutable multi-file
storage with exact retrieval, hardened Lean/ASTRA adapter
boundaries, fail-closed trust-policy and axiom-profile evaluation, mutation
rejection for registry roots, byte-for-byte deterministic bundle reproduction,
and x402-to-XLMP binding.

`cargo audit --no-fetch` completed against the local RustSec advisory database
at commit `5a0ebed` dated 2026-09-02: 237 locked dependencies were scanned
against 1,239 advisories with no vulnerable packages reported. This does not
replace an independent audit or a later scan against a newer database.

`scripts/simulate_economics.py` completed successfully for backing,
authorization, settlement, refund, solvency, revenue allocation, compounding,
and conservative compute-impact allocations. Python bytecode compilation also
completed successfully for the repository scripts.

`scripts/simulate_use_cases.py` completed all eleven documented researcher and
participant journeys across 20 executable gates and schema-validated
ordered end-to-end traces. The run exercised the full Rust suite, durable API
restart recovery, native lifecycle and API projections, canonical binary and
allowlisted HTTPS transport, immutable filesystem storage, x402 actual-use
settlement, structural/schema validation, deterministic Lean export-to-ID
derivation and pseudo self-test, content-derived trust policy, formal PoIR,
generalized computational reproduction, portability, economic compliance and
conservation, calibrated compute routing, bounded impact, and deterministic
artifact packing. It produced the schema-validated structured
report at `reports/use-case-simulation.json` and the human-readable report at
`docs/USE_CASE_SIMULATION_REPORT.md`.

After the main snapshot run, the pinned `leanprover/lean4:v4.33.1` toolchain was
installed locally. The print-only marker was then replaced by a checked-
environment serializer for closed elaborated types, proof terms, universes,
direct constants, and transitive axioms. `lake build`, warning-as-error
elaboration of `XLemma/Example.lean`, exact reproduction of the checked-in
machine-readable export vector, an empty target-theorem axiom inventory, and
`lake env leanchecker --fresh XLemma.Example` all completed successfully. This
is an author-operated pseudo/self-test, documented in
`docs/LEAN_SELF_TEST_REPORT.md`; it is not independent reproduction.

Foundry 1.4.0 formatting, build-size, unit, fuzz, and invariant checks completed
successfully against the immutable OpenZeppelin and forge-std revisions listed
in `contracts/README.md`. Nine suites ran 35 tests; the stateful vault invariant
ran 256 sequences of 32,768 calls. The tests include vault-only credit minting,
exact asset transfers, content-derived proof/certificate/bounty identifiers,
artifact-bound bounty release, certificate invalidation, revenue replay
namespacing, ResearcherID registration isolation, and content-bound on-chain
research, policy, committee, rights, contribution, and supersession roots.

`docker compose config --quiet` completed successfully with explicit test
secrets. The committed runtime and service image references use immutable
digests, the API binds only to loopback by default, and the container profile is
read-only, non-root, capability-free, and protected by `no-new-privileges`.

The snapshot contains 13 Rust crates, 85 JSON schemas, 24 numbered
specifications, 10 Solidity contracts, and 9 Solidity test suites. These source
counts establish repository coverage; they do not substitute for an audit.

## Not executed locally

Lean/Lake and the bundled fresh checker are now installed and were exercised as
described above. The following Lean assurance checks remain unexecuted locally
and are not claimed by the pseudo/self-test:

- nanoda or another independently implemented checker-family replay;
- the comparator challenge workflow, hostile proof corpus, and hardened
  sandbox execution;
- independent clean-room reproduction of the exporter encoding and proof bundle.

The following production activities were also outside this validation scope:

- live ASTRA/provider calls or calibration;
- live decentralized service discovery, order matching, or node-reputation assessment;
- production beacon/VRF proof authentication and decentralized eligible-set publication;
- production credential issuer, delegation-signature/key-resolution, private uniqueness, and revocation-accumulator integrations;
- live x402, stablecoin, grant, escrow, invoice, IPFS, or chain integration;
- an independent clean-room execution of the published bundle and trust-policy
  vectors, plus a second XLMP implementation for the remaining vectors;
- independent security, cryptographic, economic, smart-contract, or legal audit;
- static analysis, symbolic execution, chain-fork testing, or a public security contest.

## Release status

This remains an architectural reference implementation and prototype. The
XLMP/1 provider-neutral boundary, 53 canonical messages, lifecycle,
first-class identity/credential and node-network planes, native sovereignty,
portability, economic-compliance, generalized-verification, capture,
node-economics, and governance records, schemas, and adapters are implemented
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

cd lean && ./self-test.sh
cd ../contracts && forge fmt --check && forge build --sizes && forge test -vvv
```

Use `docs/PRODUCTION_CHECKLIST.md` as the deployment gate rather than treating
this report as production certification.
