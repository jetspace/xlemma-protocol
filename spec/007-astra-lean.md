# XLIP-007 — Research-prover and verifier adapters

XLMP defines provider-neutral `ResearchProver` and `VerifierAdapter` boundaries. ASTRA is a configurable proof-production implementation. It MAY formalize, decompose, search, generate, repair, retrieve, compare, and explain. It MUST label outputs as candidates.

ASTRA receipts MUST bind model/provider, request hash, context root, usage, charge, candidate roots, and time. Model identity SHOULD include a snapshot where available.

Lean is the default XLMP/1 formal-verification implementation, not the protocol itself. High-assurance Lean verification SHOULD:

1. use an exact trusted challenge;
2. build hostile code in a no-network sandbox;
3. export a checker-consumable proof object;
4. inspect axioms and disallowed trust paths;
5. replay with the official Lean kernel/checker;
6. replay with an independently implemented checker family;
7. compare exact theorem statements;
8. sign independent receipts.

A model, builder, or checker MUST NOT silently modify the trusted claim.

The deployment MUST resolve the theory's content-derived trust policy through
an authenticated registry snapshot and compare the proof manifest with the
checker-produced trust evidence. Unlisted axioms, `sorry`/`admit`, unsafe
declarations, compiler-trusted `native_decide`, an unpinned toolchain, an
unverified dependency lock, or insufficient checker-family evidence fail
closed under the reference Gold policy. See XLIP-023.

Formal checking establishes that the exact theorem follows under declared
definitions, imports, and axioms. It does not establish that the theorem
faithfully represents an informal claim or empirical interpretation. ASTRA and
Lean MUST NOT issue their own final `StatementAlignmentReceipt`; that record is
produced through the independent human/domain-review boundary in XLIP-021.

Other formal systems MAY implement `VerifierAdapter` under explicit theory, canonicalization, trust, and checker-policy identifiers. Supporting another system MUST NOT permit proof-producer self-certification, majority resolution of checker divergence, or text-derived formal ClaimIDs.
