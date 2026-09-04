# XLIP-016 — Privacy and confidential research

Private bundles SHOULD be encrypted client-side before content-addressed publication. Payment authorization MAY release a wrapped decryption key after settlement.

Public ledgers SHOULD contain only commitments, policy references, minimal status, and economic state. Unpublished proofs, personal data, confidential datasets, export-controlled material, trade secrets, prompts, and private review evidence SHOULD remain off-chain.

Receipts SHOULD support selective disclosure or redaction without breaking the ability to verify disclosed roots. Model-provider transmission requires explicit data classification and policy consent.

Public node identity is pseudonymous and follows `VerifiedUserID → OperatorID →
NodeID(s)`. `XLMP_UserCredential` publishes an issuer, uniqueness commitment,
coarse qualifications, tier, evidence root, and public pseudonym—not raw legal
identity. Legal names, identity documents, addresses, biometric data, and the
issuer's uniqueness evidence MUST remain with the issuer under a disclosed
retention and access policy.

Selective-disclosure and zero-knowledge credentials MAY be used when they
preserve stable accountability, uniqueness, delegation, expiration, and
revocation checks. Privacy MUST NOT be used to multiply committee independence:
multiple NodeIDs or OperatorIDs under one verified participant count as one
domain. See XLIP-020.
