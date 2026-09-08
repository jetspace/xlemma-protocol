# Local development

Run commands from the repository root unless a block changes directory. The
[README](../README.md) introduces the project; this guide collects setup,
validation, reference-vector and API commands.

## Prerequisites

| Tool | Used for |
|---|---|
| Python 3.11+ | Structural validation and simulation runners; CI uses Python 3.12. |
| Rust through `rustup` | The workspace pins Rust 1.82.0 with rustfmt and Clippy in `rust-toolchain.toml`. |
| Lean through `elan` | Lean-specific checks use the version pinned in `lean/lean-toolchain`. |
| Foundry | Solidity and local EVM checks; dependency installation and the CI-pinned version are documented below. |

Set up Python dependencies using the versions in
[CI](../.github/workflows/ci.yml):

```sh
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install jsonschema==4.25.1 pyyaml==6.0.2
```

`Cargo.lock` is committed; use `--locked` for reproducible Rust dependency
resolution. Contract dependencies are installed separately at the revisions
specified in [contracts/README.md](../contracts/README.md). Production dependency
qualification remains an open gate.

## Validate the repository

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/validate_repo.py
python3 scripts/simulate_economics.py
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 scripts/simulate_discovery.py --check
```

The structural validator checks schemas, examples, OpenAPI references, invariants,
source inventory and `MANIFEST.sha256`. After intentional source edits, regenerate
the manifest with `python3 scripts/generate_manifest.py`, then validate again.
To independently check digests, use `shasum -a 256 -c MANIFEST.sha256` on macOS or
`sha256sum -c MANIFEST.sha256` where available.

The [Makefile](../Makefile) provides equivalent targets such as `make validate`,
`make rust-test`, `make lean-test` and `make contracts-test`.

### Lean and participant journeys

```sh
(cd lean && ./self-test.sh)
python3 scripts/simulate_use_cases.py \
  --markdown /tmp/xlemma-journeys.md \
  --json /tmp/xlemma-journeys.json
```

The Lean script is an author-operated package, export, axiom and bundled-checker
self-test. It does not establish independent checker qualification or clean-room
reproduction. The journey runner also invokes the Lean self-test; it writes its
reports to the supplied temporary paths so a local run does not overwrite the
committed historical reports. See [Lean documentation](../lean/README.md) and the
[testing strategy](TESTING_STRATEGY.md).

### Solidity and discovery settlement

Install Foundry and the pinned contract dependencies using the
[contract instructions](../contracts/README.md#dependency-installation-and-tests).
CI uses Foundry v1.4.0. Then run:

```sh
(cd contracts && forge fmt --check && forge build --sizes && forge test -vvv)
cargo build --locked -p xlemma-cli
python3 scripts/test_discovery_evm.py
```

The discovery integration starts an isolated local Anvil with a mock six-decimal
token and an explicit research-checker test double. It exercises funding, signed
allocation, certificate publication, settlement and refunds. It does not launch
a funded network or establish scientific validity. The signed service workflow,
observer requirements and deployment gates are in [DISCOVERY_SERVICE.md](DISCOVERY_SERVICE.md).

## Run the API

The API reads its **process environment**. Copying `.env.example` alone does not
load settings into `cargo run`. Use [.env.example](../.env.example) as a template
for your environment manager, or the configured environment-file support in the
[deployment templates](../deploy/). Preserve JSON values when injecting them.

Configure these settings before starting:

| Variable | Required configuration |
|---|---|
| `XLEMMA_API_AUTH_TOKEN` | A secret bearer token generated from at least 32 random bytes. |
| `XLEMMA_TRUSTED_SIGNERS` | Comma-separated authorized `ed25519:<base64url-public-key>` identities. |
| `XLEMMA_TRUSTED_NODE_SIGNERS` | A nonempty JSON object mapping valid NodeIDs to trusted signers; every node must use a distinct key and every signer must be in the trusted set. |
| `XLEMMA_EVENT_LOG_PATH` | A persistent path for the single-writer event journal. |
| `XLEMMA_DISCOVERY_TRUST_PATH` | Optional qualified public trust file to enable discovery commands. See the discovery runbook. |
| `XLEMMA_BIND` | Optional listener address; defaults to `127.0.0.1:8080`. |

Once the settings are present in the process environment:

```sh
cargo run --locked -p xlemma-api
```

Only `/health` is unauthenticated. XLMP ingress requires canonical signed
envelopes; observation acceptance additionally binds node identity and the exact
job roster. The server fsyncs accepted events before acknowledgement and fails
closed on invalid journal recovery. Run one writer per journal and preserve its
history. Distributed replication and recovery remain production work.

Model-provider and payment settings in `.env.example` apply to their respective
adapters; configure them when exercising those external services. The basic
validation and CLI examples below do not require API credentials or a funded
wallet. Consult the [operator runbook](OPERATOR_RUNBOOK.md) before running services.

## Explore the CLI and deterministic vectors

```sh
cargo run --locked -p xlemma-cli -- --help
```

The commands below reproduce checked-in protocol examples. They do not certify
real participants or establish novelty, ownership, live settlement or production
trust. Paths are relative to the repository root.

```sh
cargo run --locked -p xlemma-cli -- derive-id user-credential examples/node-network/user-credential.json
cargo run --locked -p xlemma-cli -- lean-export-ids \
  examples/lean-export/expected-add-zero.json \
  examples/no-arbitrage/theory.json
cargo run --locked -p xlemma-cli -- credential-chain-root examples/node-network/credential-chain.json
cargo run --locked -p xlemma-cli -- evaluate-reproduction \
  examples/no-arbitrage/computational-verification-profile.json \
  examples/no-arbitrage/computational-verification-job.json \
  examples/no-arbitrage/computational-observations.json
cargo run --locked -p xlemma-cli -- verify-portability \
  examples/no-arbitrage/portability-manifest.json
cargo run --locked -p xlemma-cli -- verify-economic-compliance \
  examples/no-arbitrage/economic-constitution.json \
  examples/no-arbitrage/economic-compliance-certificate.json
cargo run --locked -p xlemma-cli -- verify-trust \
  examples/no-arbitrage/trust-policy-registry.json \
  examples/no-arbitrage/theory.json \
  examples/no-arbitrage/proof.json \
  examples/no-arbitrage/proof-trust-evidence.json
cargo run --locked -p xlemma-cli -- pack \
  examples/deterministic-bundle \
  examples/deterministic-bundle/inputs.json \
  --lean-toolchain leanprover/lean4:v4.33.1 \
  --dependency-lock-hash blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --source-commit vector-1 \
  --build-image-digest sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  --created-at 2026-09-04T12:00:00Z
```

Discovery-specific preparation, signing and EVM observer commands are documented
in [DISCOVERY_SERVICE.md](DISCOVERY_SERVICE.md#operator-workflow).

## Implement another transport

`xlemma-xlmp::encode_xlmp_frame` and `decode_xlmp_frame` provide canonical binary
framing for one XLMP envelope: a four-byte big-endian length followed by RFC 8785
JSON. Decoding preserves the HTTP envelope's MessageID and rejects trailing,
truncated, oversized or non-canonical payloads. See
[XLMP/1](../spec/018-xlmp-wire-protocol.md) for the protocol boundary and message rules.
