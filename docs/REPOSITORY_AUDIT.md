# Repository implementation and security audit

Date: 2026-09-04. Baseline commit: `0380149`.

## Readiness decision

The repository implements substantial deterministic protocol logic and working
reference adapters, but it is **not fully integrated for production**. Local
conformance does not establish independent reproduction, live settlement,
credential uniqueness, or operational resilience. This audit fixes concrete
implementation defects; it is not an independent cryptographic or financial
systems audit and does not assert that no other vulnerabilities exist.

## Scope and method

Inventoried all 13 Rust crates, 10 Solidity contracts, Lean sources and exporter,
schemas, OpenAPI, examples, deployment configuration, CI, packaging scripts,
roadmap and trust documentation. Ran the available regression/conformance suites
and dependency scan, reviewed the main ingress, persistence, proof-evidence,
payment, storage, economic and adapter boundaries, and added adversarial
regressions for confirmed defects. A tracked-source credential-pattern scan
returned zero matches; this is not a complete Git-history secret scan.

## Confirmed findings and fixes

Severity describes the impact inside the documented reference trust boundary.
Trusted message writers, host operators and external adapters are not assumed
to be equivalent to independently verified evidence.

| ID | Severity | Defect | Implemented fix and evidence |
|---|---|---|---|
| A01 | High | An authenticated formal certificate could cite previously submitted receipts without evaluating their verdicts, policy, roster or roots; an aggregator could omit dissent. | Validate all accepted job receipts, including the job observation endpoint, against the immutable roster and policy. Bind exact roots, receipt/cluster/family sets, proof, job identities, issuance time and challenge duration. Mutation, unauthorized-roster and dissent regressions pass. |
| A02 | High | Formal CertificateIDs were only syntax checked and could remain unchanged after content substitution. | Derive the ID from all certificate fields except its ID and signature; enforce it at XLMP ingress. Schema structure is unchanged; the draft semantic identity requirement is stricter. |
| A03 | High | x402 wrappers were caller-recomputable and consumption was keyed by the wrapper ID; failed settlement calls reopened authorization even after a possible external charge. | Require exact locally issued records, consume the underlying network/payment ID atomically, and retain consumption on uncertain outcomes. Recomputed expiry, repeated issuance and lost-response regressions pass. |
| A04 | High | API relationship checks differed from the native projection and restart replay. Capsule references could refer to unrelated objects; publication timing and license relationships were incomplete. | Use the same projection at API ingress and replay. Bind claim/theory/proof/artifact/rights/contribution relationships, challenge lineage and publication license/finality order. Reject future live finalization/publication timestamps. |
| A05 | Medium | A journal could continue after partial writes/fsync errors; a complete JSON record lacking its newline was accepted on recovery, and multiple writers were not excluded. | Poison failed writers, require complete bounded records, verify envelope signatures on recovery, sync file/directory metadata, use Unix advisory locking and reject symlink/non-regular journal files. Fault-injection and recovery regressions pass. |
| A06 | Medium | Storage retrieval loaded payloads before checking aggregate manifest limits, allowing resource exhaustion from corrupt local metadata. | Check manifest limits and identity before payload I/O, bound each read to its declared size, and reject non-regular descriptors. Pre-read rejection regression passes. |
| A07 | High | Archive and manifest generators included local `.env`, private keys, runtime artifacts and agent configuration. | Share a source inventory excluding these common sensitive paths; test exclusion and symlink rejection; enforce inventory tests in CI. No release archive was published. |
| A08 | Medium | Converting an ASTRA receipt into `ComputeReceipt` copied an ID/signature for a different structure, so the result failed native XLMP integrity validation. | Derive the native receipt ID and require a separate native signing operation. An adapter-to-XLMP regression now passes. |
| A09 | Medium | ASTRA accepted impossible cached-token counts, saturated charge arithmetic, and could panic on non-JCS-safe metering; x402 timeout arithmetic could panic. | Validate metering and timeout representability, use checked charge arithmetic, and propagate errors. Invalid-count, overflow and extreme-timeout regressions pass. |
| A10 | Medium | Solidity compounding reverted on legitimate allocations rounded to zero and accepted vault calls that consumed no funds. | Skip zero allocations; verify exact vault consumption and the final router balance. Dust and no-op-vault regressions pass. |
| A11 | Low | The participant journey harness existed but was not executed in CI. | Add it to the Lean CI job with its Rust/Python prerequisites; preserve the existing independent-checker CI gates. |

## Implementation and integration inventory

| Area | Implemented/reference evidence | Remaining integration or assurance |
|---|---|---|
| Core / XLMP / crypto | Typed IDs, canonicalization, structural schemas, Ed25519 envelopes, framing, append-only native projection and stricter certificate identity. | Clean-room encoding reproduction; governance-authenticated policy/key roots; additional cross-language vectors. |
| API / persistence | Authenticated HTTP ingress, native reads, formal job/observation evaluation, local durable journal. | Global bearer/signer allowlists are administrative trust, not per-object ownership or issuer authorization. PostgreSQL/NATS in Compose are not connected. No replicated log, outbox, automatic quarantine worker or externally anchored rollback protection. |
| Consensus / node network | Deterministic committee selection, credential-chain structure, commit/reveal, dissent-aware formal evaluation, market matching and reputation policies. | Live issuer/delegation verification, freshness/revocation services, Sybil-resistant operator clustering and authenticated beacon integration. Generalized research messages still rely on trusted writers for job/profile/credential authority. |
| Lean | Pinned 4.33.1 exporter, byte-stable vector, axiom inventory and local fresh-checker self-test. | `LocalCommandRunner` deliberately refuses hostile execution. No production sandbox runner, clean-room exporter, independently operated checker network or hostile-corpus qualification. Adapter command lines and comparator output require qualification against the selected external tools. |
| ASTRA | Configurable HTTPS prover boundary, metering and correctly converted native receipts. | No live provider call was made. Provider-neutral artifact results still expose references rather than an integrated persisted payload workflow. External signers must implement the new native receipt signing method. |
| Economics / compute curve | Checked backing ledger, revenue/dividend caps, independent estimate verification and deterministic allocation tests. | Live external revenue reconciliation, durable cross-service allocation consumption, independent success/impact estimation and solvency review. |
| x402 | Reference authorization and settlement adapter, actual-use accounting, local issuance and replay enforcement. | Production facilitator/SDK integration, persistent reconciliation, restart/failover idempotency, qualified batch settlement and alternative payment adapters. An uncertain payment remains blocked pending external reconciliation. |
| Storage / portability | Immutable local bundles, integrity checking, bounded retrieval and portability structures. | Remote replicas, encryption/key release, independent custody challenges, cross-process transactional storage and restoration exercises. |
| Solidity | Vaults/credits, bounty escrow, bonds, proof/certificate/rights projections, cash routing and optional capsule handles. | Contract roles trust off-chain operators; they do not verify mathematical truth or credential ownership. NodeID registration needs an authenticated ownership/delegation adapter. Independent audits, production role custody, chain finality/reorg integration and deployment qualification remain open. |
| Tooling / deployment | CLI, schemas, reference journeys, dependency audit job, hardened container configuration and filtered source packaging. | Container images were not built/scanned or deployed in this pass. No complete Git-history secret scan, symbolic execution, live network exercise or disaster-recovery drill. |

## Verification

- Rust 1.82: 184 tests passed, including existing deterministic/property cases and new regressions.
- Solidity 0.8.26 / Foundry 1.4.0: 38 tests passed; includes 1,000-run settlement fuzzing and the 256-run, depth-128 vault invariant.
- Python release-inventory tests: 2 passed.
- Lean 4.33.1: package build, exact exporter vector, unsafe/valueless rejection, axiom inventory and `leanchecker --fresh` passed. Author-operated evidence; independent checker reproduction is not claimed.
- Cargo audit 0.22.2: 237 locked dependencies checked against 1,239 RustSec advisories; no advisories or denied warnings.
- Rust formatting and Clippy with warnings denied passed. Solidity formatting passed.
- Repository/schema/manifest validation and the economic sanity simulation passed.
- Slither 0.11.6 ran 102 detectors over 38 compiled contracts/interfaces/dependencies, with dependency/test findings filtered. It initially reported 36 warnings; explicit zero initialization removed 3 medium warnings. The remaining 33 were individually reviewed: 3 guarded reentrancy/balance warnings, 14 external-call-in-loop warnings, 14 timestamp warnings, and 2 maintainability notes. The added callback attack regression confirms the router rejects nested routing. These are not represented as an empty scanner result. Machine-readable triage and scanned source hashes are in [`reports/security-scan.json`](../reports/security-scan.json).
- Participant journey conformance: all 11 scenarios and 20 executable gates passed; the refreshed report inventories all 184 Rust tests.

Solidity tests used an isolated copy of the current contract source and cached
OpenZeppelin/forge-std dependencies. Foundry needed execution outside the macOS
sandbox because its system-configuration library crashed in the sandbox.
Dependency source commits must still be re-established from the pinned public
commits when reproducing the build in a clean environment.

## Compatibility and operational follow-up

Do not edit old journals or historical protocol objects to force acceptance.
Keep the original evidence and explicitly rebuild/reissue invalid objects under
the tightened draft rules. Existing histories containing orphan claims or
label-derived formal CertificateIDs can now fail recovery.

`AstraReceiptSigner` implementers must separately sign native `ComputeReceipt`
objects. x402 authorizations only work in the adapter instance that issued them;
this is deliberate until durable issuance and reconciliation are integrated.
Unix file locking is local/cooperative and does not protect against a malicious
host, lock-ignoring writer, network-filesystem semantics or a rewritten log.

Do not mark the production gates in `ROADMAP.md` or `PRODUCTION_CHECKLIST.md`
complete on the strength of this repository pass.

## Advisory sources

The dependency scan uses the [RustSec advisory database](https://rustsec.org/advisories/).
The Solidity dependency review consulted the upstream
[OpenZeppelin security advisories](https://github.com/OpenZeppelin/openzeppelin-contracts/security/advisories).
The additional Solidity static scanner is [Slither](https://pypi.org/project/slither-analyzer/).


## Discovery service follow-up — 2026-09-05

The [funded discovery service](DISCOVERY_SERVICE.md) adds authenticated,
journaled round/assessment/appeal operations, exact verification-policy and
evidence bindings, contributor consent, independent simultaneous-discovery
sharing, protected category funding, and verdict-neutral reproduction/review
payments. The USDC escrow and protocol-certificate publication registry are
connected by executable receipt observers and a local EVM integration.

The updated [security scan](../reports/discovery-security-scan.json) records
237 Rust dependencies with no known advisories and the complete Slither triage:
43 remaining warnings across the contract workspace, including the three
previously reviewed guarded router reentrancy/balance findings. New discovery
findings are bounded external calls, deadline timestamps and interface
maintainability; three implicit-zero initialization warnings were removed.
This is an author-operated review, not an independent audit or production launch.
Validation counts and reproducible commands are recorded in
[the service validation report](../reports/discovery-service-validation.json).
