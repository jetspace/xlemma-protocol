# Contributing

xLemma is designed for adversarially robust research infrastructure. Contributions should preserve the protocol invariants in `spec/000-overview.md` and add tests for state transitions, hashing, economic conservation, and consensus divergence.

1. Open an issue describing the invariant affected.
2. Keep protocol objects append-only and content-addressed.
3. Do not add token-weighted mathematical voting.
4. Do not make verifier compensation contingent on a passing verdict.
5. Add a threat-model update for new externally reachable components.
6. Run `make validate`, `make fmt`, `make lint`, and `make test` before submission.

Security issues should follow `SECURITY.md` rather than public disclosure.
