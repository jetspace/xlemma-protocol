# XLIP-000 — Protocol overview and normative invariants

Status: Draft XLMP/1

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## Purpose

XLMP is xLemma's canonical, provider-neutral protocol for proof production, deterministic verification, provenance, rights manifests, researcher funding, paid access, compute markets, and downstream rewards for decentralized researchers. ASTRA, Lean, x402, chains, transports, and storage systems are adapters; none defines XLMP research state or consensus.

## Normative invariants

1. Formal validity MUST NOT be decided by token-weighted or popularity-weighted voting.
2. Every formal certificate MUST bind an exact TheoryID, ClaimID, ProofID, ArtifactID/root, dependency root, axiom policy, checker policy, and observation set.
3. Required checker-family disagreement MUST produce divergence or quarantine.
4. ASTRA or another proof producer MUST NOT certify its own candidate as final.
5. Research credits MUST NOT be issued beyond independently valued backing.
6. Research-credit ownership MUST NOT affect node selection, node reputation, or verification outcome.
7. Verification nodes MUST be compensated for reproducible execution, not for returning a pass.
8. Payment receipts and research-verification receipts MUST remain distinct.
9. Origin attribution MUST NOT be rewritten by token transfer.
10. A rights manifest MUST NOT imply rights beyond those actually held by its signers or legal wrapper.
11. Formal claim changes MUST create a new ClaimID.
12. Corrections, challenges, revocations, and supersession MUST be append-only.
13. Only dependencies used by the final proof object MAY receive automatic dependency rewards.
14. Upstream rewards MUST be capped by realized downstream net revenue.
15. LaTeX or natural-language presentation MUST NOT override the formal Lean declaration.
16. A public “verified” representation MUST disclose the verification policy or assurance level.
17. Payment, transport, chain, model, checker, and storage adapters MUST NOT redefine XLMP object identity or research-state transitions.
18. An XLMP message MUST preserve its content-derived MessageID across transport adapters.
19. No node may contribute to XLMP consensus without a valid, non-revoked OperatorCredential controlled by a verified participant; multiple nodes under one participant MUST count as one independence domain.
