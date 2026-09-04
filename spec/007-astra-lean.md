# XLIP-007 — ASTRA production and Lean verification

ASTRA is a configurable proof-production adapter. It MAY formalize, decompose, search, generate, repair, retrieve, compare, and explain. It MUST label outputs as candidates.

ASTRA receipts MUST bind model/provider, request hash, context root, usage, charge, candidate roots, and time. Model identity SHOULD include a snapshot where available.

High-assurance Lean verification SHOULD:

1. use an exact trusted challenge;
2. build hostile code in a no-network sandbox;
3. export a checker-consumable proof object;
4. inspect axioms and disallowed trust paths;
5. replay with the official Lean kernel/checker;
6. replay with an independently implemented checker family;
7. compare exact theorem statements;
8. sign independent receipts.

A model, builder, or checker MUST NOT silently modify the trusted claim.
