# OpenAPI

The API contract describes the intended protocol surface. The Rust reference
server currently implements health, canonical XLMP/1 message ingress/read,
verification-job creation/read, committed and signed observation submission,
and formal reproduction evaluation. Remaining endpoints are specification targets.
The node-advertisement, discovery, service-order, and committee-sortition paths
describe the first-class XLMP node-network surface; they are currently
specification targets backed by schemas and deterministic Rust reference logic.

Payment-protected endpoints may use x402 headers in addition to service
authentication. x402 is optional and remains separate from XLMP state. The
reference ingress validates content-derived MessageIDs and Ed25519 signatures,
requires a configured signer allowlist, binds one distinct trusted key to each
NodeID, and rejects unknown or non-canonical typed XLMP fields. Verification
jobs retain their server-side policy and exact checker roster; a reveal must
match both its signed commit and that stored job state.
Durable replay state, researcher credential resolution, privacy controls, and
production key custody remain deployment integrations.

Formal status does not imply semantic alignment. XLMP/1 defines the
content-addressed `StatementAlignmentReceipt` separately, and license/capsule
responses distinguish Open Commons, Commercial Research, and Sponsored
Challenge economic modes. Formal dependency fields are evidence only; payment
requires a separate economic-policy edge and settled revenue record.
Compute-quote callers supply a signed protocol calibration record, not their
own success probabilities; deployments authorize estimator keys under the
referenced policy and calculate monetary amounts with checked fixed-point
integer arithmetic.
