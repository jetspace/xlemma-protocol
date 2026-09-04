# Security Policy

The code and contracts are unaudited prototypes. Do not deploy with real funds or treat a generated verification receipt as production-grade certification without independent review.

## Report privately

Report vulnerabilities involving signature forgery, payment replay, proof-bundle substitution, committee manipulation, verifier equivocation, sandbox escape, contract authorization, credit over-minting, revenue conservation, or privacy leakage to the repository maintainers through a private channel.

## Fail-closed requirements

- Any checker-family disagreement quarantines the proof.
- Missing artifact, environment, dependency, axiom, or policy bindings block finalization.
- Research-credit issuance must never exceed independently valued backing.
- Settlement and verification receipts remain separate.
- ASTRA output is untrusted until Lean and independent checkers reproduce it.
- Proof-build jobs run without network access and with explicit CPU, memory, filesystem, and time limits.
