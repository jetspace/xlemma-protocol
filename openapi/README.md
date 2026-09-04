# OpenAPI

The API contract describes the intended protocol surface. The Rust reference
server currently implements health, canonical XLMP/1 message ingress/read,
verification-job creation/read, observation submission, formal evaluation, and
x402 payment-offer construction. Remaining endpoints are specification targets.

Payment-protected endpoints may use x402 headers rather than API keys as their
economic authorization mechanism. x402 is optional and remains separate from
XLMP state. The prototype message ingress validates content-derived MessageIDs
but does not authenticate envelope signatures; authentication, researcher
signatures, and privacy controls require deployment integrations.
