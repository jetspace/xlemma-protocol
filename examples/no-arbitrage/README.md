# End-to-end example

This package is intentionally simple: `Market.noFreeLunch` is defined as absence of arbitrage, so the Lean proof is direct. It demonstrates protocol wiring, not a novel theorem of finance.

Files include theory, claim, proof, artifact, contributions, rights, researcher, lemma capsule, generalized quorum policy, three independent observations, compute offers, work estimates, x402 extension, and compute-savings inputs.

Reference CLI flows:

```bash
cargo run -p xlemma-cli -- evaluate-consensus policy.json observations.json

cargo run -p xlemma-cli -- quote compute-offers.json expected-work.json \
  --deadline 2026-09-04T20:00:00Z \
  --gold-probability 0.75 \
  --novelty-probability 0.60

cargo run -p xlemma-cli -- compute-dividend \
  compute-savings-evidence.json \
  compute-savings-policy.json \
  downstream-net-revenue.json

cargo run -p xlemma-cli -- pack . bundle-inputs.json \
  --lean-toolchain leanprover/lean4:v4.33.1 \
  --dependency-lock-hash blake3:example
```

The illustrative IDs demonstrate wire format and are not claimed to be hashes of the included content; a production exporter must regenerate every identifier from canonical inputs.
