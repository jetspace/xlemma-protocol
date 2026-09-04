# XLIP-008 — x402 transport

xLemma uses x402 V2 payment semantics and adds an `xlemma` extension. Chain-specific settlement SHOULD be delegated to an audited SDK/facilitator or self-facilitated implementation.

## Schemes

- `exact`: fixed Lean check or artifact download;
- `upto`: one-request variable ASTRA/repair usage;
- `batch-settlement`: repeated proof-state calls or long-running agent sessions.

## Extension fields

The extension MUST bind protocol version, JobID, ResearcherID, ClaimID, optional ProofID, artifact commitment, quote ID, verification policy, model policy, rights manifest, revenue route, delivery mode, and expiration.

## Separation

The payment facilitator verifies and settles payment. It MUST NOT issue a formal research certificate unless it is independently operating a separately qualified verification role under the applicable conflict policy.

## Idempotency

Every retryable payment MUST include a stable payment identifier. Settlement MUST be at most once for an authorization.
