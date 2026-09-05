# Solidity reference contracts

These contracts implement the economic and registry projection of xLemma. They are deliberately unable to decide mathematical truth. Off-chain PoIR nodes reproduce evidence; contracts escrow neutral value, record immutable roots, enforce challenge windows, and route realized revenue.

## Contracts

| Contract | Purpose | Critical constraints |
|---|---|---|
| `ResearcherCredit.sol` | Researcher-specific service credit | Restricted transfer; vault-only mint/burn; no validity or profit vote |
| `ResearchVault.sol` | Holds neutral 1:1 backing | Payee-bound idempotent authorization; actual-only settlement; refund; solvency assertion |
| `ResearchVaultFactory.sol` | Creates researcher vaults | Self-registration; account-namespaced ResearcherIDs prevent cross-account slot squatting |
| `DiscoveryRoundEscrow.sol` | Funded open discovery rounds | Seven isolated categories, protected foundation floor, bounded atomic plans, independent approval/holds, exact transfers and refunds |
| `DiscoveryEvidenceRegistry.sol` | Publish protocol certificate IDs for discovery | Independent qualified relays, exact claim/artifact/policy binding, producer exclusions, delayed finality, irreversible quarantine |
| `PoIRCertificateRegistry.sol` | Economic finality for off-chain evidence certificates | Content-derived certificate ID; minimum challenge period; challenge/quarantine/reject; exact claim/artifact/policy query |
| `ProofRegistry.sol` | Append-only proof/certificate roots | Content-derived record ID; challenge-gated finalization; correction by pre-linked child/supersession |
| `ResearchCommitmentRegistry.sol` | Generic on-chain research-object projection | Commits researcher, claim, artifact, policy, committee, rights, contribution-split, and supersession roots without deciding truth |
| `NodeBondRegistry.sol` | Neutral node collateral | Immutable operator cluster per NodeID; delayed unbond; stake is eligibility, not vote weight |
| `BountyEscrow.sol` | Reverse-direction proof bounty | Content-derived bounty ID; commit-reveal; exact artifact certificate; delayed release/refund; finality recheck |
| `RevenueRouter.sol` | Realized external revenue distribution | Payer-namespaced replay IDs; 10,000-bps conservation; exact transfers; atomic vault compounding |
| `LemmaCapsule1155.sol` | Optional capsule/license handles | Originator immutable; proof/support tokens nontransferable; license/access editions may transfer |

## Dependency installation and tests

The archive does not vendor OpenZeppelin or forge-std:

```bash
cd contracts
forge install OpenZeppelin/openzeppelin-contracts@fcbae5394ae8ad52d8e580a3477db99814b9d565 --no-git
forge install foundry-rs/forge-std@8e40513d678f392f398620b3ef2b418648b33e89 --no-git
forge fmt --check
forge test -vvv
```

Included tests cover:

- deposit and 1:1 minting;
- x402-style maximum authorization, actual settlement, and exact refund;
- no double settlement or over-settlement;
- expired authorization cancellation;
- restricted peer-to-peer research-credit transfer;
- rejection of administrator attempts to mint unbacked credits;
- external-revenue compounding and redemption;
- fuzzed settlement solvency;
- stateful vault backing invariant;
- revenue conservation, compounding, and replay resistance;
- PoIR content identity, challenge windows, and quarantine;
- bounty content identity, commit-reveal, exact-artifact certificate dependence, and invalidation refund;
- append-only proof records and supersession;
- content-bound research, committee, rights, and contribution roots with explicit supersession;
- factory registration authorization and ResearcherID namespace isolation;
- immutable node operator clusters, unbond delay, and slashing;
- nontransferable proof capsules and transferable license editions.

## Deployment role graph

A production deployment should transfer administration from an EOA to reviewed threshold governance. At minimum:

```text
Vault admin
  ├── SETTLER_ROLE          narrowly scoped x402 reconciliation service
  └── REVENUE_ROUTER_ROLE   audited RevenueRouter only

PoIR registry admin
  ├── AGGREGATOR_ROLE       threshold receipt aggregator
  ├── CHALLENGER_ROLE       open/bonded challenger adapter or broad watcher set
  └── RESOLVER_ROLE         independent security/dispute threshold

Proof registry admin
  ├── CERTIFIER_ROLE        PoIR finality adapter
  └── QUARANTINE_ROLE       security threshold

Research commitment admin
  └── COMMITTER_ROLE        threshold adapter for authenticated XLMP commitments

Node bond admin
  └── SLASHER_ROLE          evidence-bound dispute outcome adapter
```

No role should be able to mint unbacked credits or turn a divergent proof into a valid one.

## Explicit limitations

The contracts are unaudited reference implementations. Production work must address:

- callback, blacklist, and depeg behavior of settlement assets; fee-on-transfer and rebasing assets are deliberately rejected by exact-balance checks;
- open role administration, timelocks, threshold keys, escape hatches, and governance capture;
- VRF/sortition and off-chain certificate signature verification;
- chain reorganization and cross-chain settlement assumptions;
- typed-data signatures, replay domains, oracle dependencies, and account abstraction;
- sanctions, tax, privacy, identity, consumer-credit, money-transmission, securities, and jurisdictional rules;
- upgrade/migration and user redemption paths;
- formal verification, fuzzing, static analysis, public contest, and independent audits.

Do not deploy with real funds until `docs/PRODUCTION_CHECKLIST.md` is satisfied and independent reviews are complete.

The [discovery service runbook](../docs/DISCOVERY_SERVICE.md) connects these
contracts to signed commands and EVM receipt observers. Run
`python3 scripts/test_discovery_evm.py` from the repository root after building
the CLI and contracts to exercise the funded settlement path on a fresh Anvil.
