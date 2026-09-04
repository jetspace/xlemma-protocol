# Solidity reference contracts

These contracts implement the economic and registry projection of xLemma. They are deliberately unable to decide mathematical truth. Off-chain PoIR nodes reproduce evidence; contracts escrow neutral value, record immutable roots, enforce challenge windows, and route realized revenue.

## Contracts

| Contract | Purpose | Critical constraints |
|---|---|---|
| `ResearcherCredit.sol` | Researcher-specific service credit | Restricted transfer; vault-only mint/burn; no validity or profit vote |
| `ResearchVault.sol` | Holds neutral 1:1 backing | Payee-bound idempotent authorization; actual-only settlement; refund; solvency assertion |
| `ResearchVaultFactory.sol` | Creates one vault per ResearcherID | Registry convenience; production deployment must add reviewed governance and allowlists |
| `PoIRCertificateRegistry.sol` | Economic finality for off-chain evidence certificates | Minimum challenge period; challenge/quarantine/reject; matching claim/policy query |
| `ProofRegistry.sol` | Append-only proof/certificate roots | One first certificate per record; correction by child/supersession; quarantine/revoke preserved |
| `NodeBondRegistry.sol` | Neutral node collateral | Immutable operator cluster per NodeID; delayed unbond; stake is eligibility, not vote weight |
| `BountyEscrow.sol` | Reverse-direction proof bounty | Commit-reveal; final matching PoIR certificate; release delay; finality recheck |
| `RevenueRouter.sol` | Realized external revenue distribution | Replay-safe event IDs; 10,000-bps conservation; atomic vault compounding |
| `LemmaCapsule1155.sol` | Optional capsule/license handles | Originator immutable; proof/support tokens nontransferable; license/access editions may transfer |

## Dependency installation and tests

The archive does not vendor OpenZeppelin or forge-std:

```bash
cd contracts
forge install OpenZeppelin/openzeppelin-contracts --no-commit
forge install foundry-rs/forge-std --no-commit
forge fmt --check
forge test -vvv
```

Included tests cover:

- deposit and 1:1 minting;
- x402-style maximum authorization, actual settlement, and exact refund;
- no double settlement or over-settlement;
- expired authorization cancellation;
- restricted peer-to-peer research-credit transfer;
- external-revenue compounding and redemption;
- fuzzed settlement solvency;
- stateful vault backing invariant;
- revenue conservation, compounding, and replay resistance;
- PoIR challenge windows and quarantine;
- bounty commit-reveal and final-certificate dependence;
- append-only proof records and supersession;
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

Node bond admin
  └── SLASHER_ROLE          evidence-bound dispute outcome adapter
```

No role should be able to mint unbacked credits or turn a divergent proof into a valid one.

## Explicit limitations

The contracts are unaudited reference implementations. Production work must address:

- transfer-fee, rebasing, callback, blacklist, and depeg behavior of settlement assets;
- open role administration, timelocks, threshold keys, escape hatches, and governance capture;
- VRF/sortition and off-chain certificate signature verification;
- chain reorganization and cross-chain settlement assumptions;
- typed-data signatures, replay domains, oracle dependencies, and account abstraction;
- sanctions, tax, privacy, identity, consumer-credit, money-transmission, securities, and jurisdictional rules;
- upgrade/migration and user redemption paths;
- formal verification, fuzzing, static analysis, public contest, and independent audits.

Do not deploy with real funds until `docs/PRODUCTION_CHECKLIST.md` is satisfied and independent reviews are complete.
