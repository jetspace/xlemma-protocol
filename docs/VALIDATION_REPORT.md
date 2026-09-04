# Validation report — v0.2.0 source snapshot

As-of date: 2026-09-03.

## Scope

This report distinguishes validations executed in the packaging environment from validations delegated to CI or still required before production.

## Executed successfully

The repository's `scripts/validate_repo.py` completed successfully and checked:

- required source, specification, schema, example, contract, Lean, LaTeX, and documentation files;
- JSON syntax for all protocol schemas and examples;
- JSON Schema Draft 2020-12 validity;
- example instances against their schemas;
- observation and compute-offer arrays;
- TOML syntax and default policy invariants;
- YAML syntax for OpenAPI, Docker Compose, and GitHub Actions;
- OpenAPI 3.1 operation IDs, response maps, required routes, repository-contained external schemas, fragments, and local reference resolution;
- revenue-waterfall conservation;
- contributor-share conservation;
- example checker root agreement;
- operator, provider, and regional diversity in the example quorum;
- required two official-kernel plus one independent-checker policy;
- explicit documentation of core protocol invariants;
- absence of obvious embedded API keys or private-key blocks;
- deterministic `MANIFEST.sha256` file-set coverage and all source digests.

`scripts/simulate_economics.py` completed successfully and exercised:

- 1:1 research-credit backing;
- maximum authorization, actual settlement, and refund;
- post-settlement solvency;
- gross-to-net research revenue;
- creator-pool allocation;
- cash/auto-compound split;
- conservative revenue-capped compute-savings dividend.

Python source compilation completed successfully for the scripts. The source snapshot contains 12 Rust crates, 20 JSON schemas, 18 numbered specifications, 9 Solidity contracts, 7 Solidity test suites, and 40 Rust unit/async-test declarations. These counts establish repository coverage, not native compilation.

The source review also corrected:

- formal quorum enforcement of independent operators, infrastructure providers, and regions;
- timestamp-independent artifact identities;
- transactional in-memory credit mutations;
- duplicate contributor rejection;
- authorization payee binding;
- bounty dependence on a final matching PoIR certificate;
- revenue-event replay protection;
- atomic external-revenue compounding;
- storage traversal and symlink containment.

## Not executed in this packaging environment

The environment did not include Rust/Cargo, Lean/Lake, Foundry/Solidity, or network access to install those toolchains and dependencies. Therefore the following were not falsely claimed as executed:

- `cargo fmt`, `cargo clippy`, `cargo test`, and native Rust compilation;
- Lean package build and checker execution;
- Foundry formatting, build, tests, fuzzing, or invariant suites;
- OpenZeppelin dependency resolution;
- live OpenAI ASTRA calls;
- live x402 facilitator/network settlement;
- live IPFS or chain integration;
- independent security, cryptographic, economic, or legal audit;
- generation of `Cargo.lock` and deployment-specific Solidity dependency locks, because dependency resolution was unavailable in the packaging environment.

The included CI workflow runs native Rust, Lean, Lean kernel rechecking, nanoda, axiom auditing, and Foundry jobs in an environment with dependency access. The first dependency-enabled release build must generate, review, and commit `Cargo.lock` and deployment-specific contract dependency locks before production tagging.

## Release status

This repository is an architectural reference implementation and prototype. Structural/schema/economic checks pass. Native compilation, cross-checker proof validation, adversarial distributed tests, contract audits, and production operations remain mandatory gates.

## Reproduction commands

```bash
python3 scripts/validate_repo.py
python3 scripts/simulate_economics.py
python3 -m compileall -q scripts
sha256sum -c MANIFEST.sha256

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd lean && lake build
cd ../contracts && forge test -vvv
```

Use `docs/PRODUCTION_CHECKLIST.md` as the deployment gate rather than treating this report as production certification.
