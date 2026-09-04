# XLIP-008 — x402 payment adapter

x402 is an optional `PaymentAdapter` and paid-HTTP transport for XLMP/1. xLemma adds an `xlmp` extension that binds the payment obligation to a canonical XLMP MessageID. Chain-specific settlement SHOULD be delegated to an audited SDK/facilitator or self-facilitated implementation.

An XLMP implementation MUST remain usable with another payment adapter or with no payment. x402 fields MUST NOT define research state, proof validity, attribution, rights, or consensus.

## Schemes

- `exact`: fixed Lean check or artifact download;
- `upto`: one-request variable ASTRA/repair usage;
- `batch-settlement`: repeated proof-state calls or long-running agent sessions.

## Extension fields

The extension MUST bind `XLMP/1`, MessageID, JobID, ResearcherID, ClaimID, optional ProofID, artifact commitment, quote ID, verification policy, model policy, rights manifest, revenue route, delivery mode, and expiration.

## Separation

The payment facilitator verifies and settles payment. It MUST NOT issue a formal research certificate unless it is independently operating a separately qualified verification role under the applicable conflict policy.

## Idempotency

Every retryable payment MUST include a stable payment identifier. Settlement MUST be at most once for an authorization.
