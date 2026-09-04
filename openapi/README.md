# OpenAPI

The API contract describes the intended protocol surface. The Rust reference server currently implements health, verification-job creation/read, observation submission, and formal evaluation. Remaining endpoints are specification targets.

Payment-protected endpoints use x402 headers rather than API keys as their economic authorization mechanism. Authentication, researcher signatures, and privacy controls remain separate from payment.
