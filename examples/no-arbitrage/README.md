# End-to-end example

This package is intentionally simple: `Market.noFreeLunch` is defined as absence of arbitrage, so the Lean proof is direct. It demonstrates protocol wiring, not a novel theorem of finance.

Files include theory, claim, proof, artifact, contributions, rights, researcher,
lemma capsule, statement-alignment review, generalized quorum policy, three
independent observations, compute offers, independent protocol success
estimates, work estimates, x402 extension, and authorized compute-impact inputs.

Reference CLI flows:

```bash
cargo run -p xlemma-cli -- evaluate-consensus policy.json observations.json

cargo run -p xlemma-cli -- quote \
  compute-offers.json expected-work.json protocol-success-estimates.json \
  --trusted-estimator ed25519:6kpsY-KcUgq-9VB7Ey7F-ZVHdq6-vnuSQh7qaRRG0iw \
  --deadline 2027-09-04T20:00:00Z

cargo run -p xlemma-cli -- compute-impact \
  compute-savings-evidence.json \
  compute-savings-policy.json \
  downstream-net-revenue.json \
  impact-pool-authorization.json \
  --trusted-authorizer ed25519:_RckOFqgx1tk-3jNYC-h2ZH96_drE8WO1wLqyDXp9hg

cargo run -p xlemma-cli -- pack . bundle-inputs.json \
  --lean-toolchain leanprover/lean4:v4.33.1 \
  --dependency-lock-hash blake3:example
```

The illustrative IDs demonstrate wire format and are not claimed to be hashes of the included content; a production exporter must regenerate every identifier from canonical inputs.
