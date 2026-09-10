# USDC wallet boundary and offline preflight

Status: first implementation slice, 2026-09-10. `wallet-context` and
`wallet-preflight` are read-only CLI commands backed by deterministic Rust
validation, schemas and reference vectors. No smart account is deployed, no
UserOperation is created or signed, and no payment or sponsorship is reserved.

## Pinned reference profile

[wallet-base-sepolia.json](../config/wallet-base-sepolia.json) selects one explicit
contract set. Addresses use canonical lowercase spelling in protocol inputs.

| Parameter | Pinned value |
|---|---|
| Chain | Base Sepolia, `84532` |
| USDC | `0x036cbd53842c5426634e7929541ec2318f3dcf7e` |
| EntryPoint | v0.7, `0x0000000071727de22e5e9d8baf0edac6f37da032` |
| Circle Paymaster | v0.7 testnet, `0x31be08d380a21fc740883c0bc434fcfc88740b58` |
| Signing and fees | Owner only; self-funded USDC fees |
| Example ceilings | 100 USDC funding, 1 USDC total fee, 300-second intent lifetime |

These ceilings are configurable local policy limits, not fee estimates or actual
funding. Amounts use six-decimal USDC minor units and stay within the protocol's
safe integer range, including the combined maximum debit. This profile rejects
mainnet, substituted tokens/contracts, delegation and sponsored fee modes.

Sources checked on 2026-09-10: [Base network identifiers](https://docs.base.org/base-chain/api-reference/rpc-overview),
[Circle USDC addresses](https://developers.circle.com/stablecoins/usdc-contract-addresses),
[Circle Paymaster addresses](https://developers.circle.com/paymaster/addresses-and-events),
and [viem's pinned EntryPoint constants](https://github.com/wevm/viem/blob/86adf420cfb20721ff99aa5bf6e6c1017588d473/src/account-abstraction/constants/address.ts).
Address-source checks are not RPC bytecode checks, a deployment audit, wallet SDK
compatibility tests or evidence that a bundler is available. Those remain required
before signing. This profile does not automatically upgrade to another version.

## Identity and permitted actions

The discovery trust file pins the escrow, chain, USDC asset and participant
identities. The wallet policy and entire trust file receive separate PolicyIDs;
an intent binds both. The selected registered principal's payout address must
match the wallet. This is a configuration binding, not proof of wallet ownership
or authentication of the principal. A qualified account adapter must establish
ownership and obtain the owner's signature before any execution.

The initial intent vocabulary is limited to `fund_round` and `refund`. The draft
contains the exact escrow ABI signature and typed arguments; arbitrary targets,
raw calldata, additional recipients and delegate calls are not accepted inputs.
Funding authorizes a proposed exact deposit ceiling and a separate total fee
ceiling. Refund preparation budgets its fee from the wallet's existing balance;
an expected refund cannot supply that upfront balance. Preflight does not query
balances, round state, allowances or refund entitlement, so it does not promise
that the operation is currently executable.

Research submissions, evidence and appeals already use signed XLMP/discovery
messages offchain. Their service fees retain the existing economic rules.
Receiving a USDC award does not itself require a recipient transaction. The wallet
layer must not add a mandatory onchain transaction to every research message.

## Budgets, signing and the next adapter

The first profile supports owner-only preparation. Delegation remains disabled
until an account adapter can enforce target/method restrictions, decoded call
arguments, amount and cumulative budgets, expiry, revocation and replay protection
at the signing/execution boundary. Approvals must be bounded by the consented
escrow and Paymaster ceilings; neither an unlimited allowance nor opaque batch
execution is authorized by an offline draft.

Sponsorship requires a separate, funded budget for gas. It must record total and
per-participant caps, request limits, eligibility, commitments and settled fees.
Before acknowledging a reservation, durably record it and subtract existing
commitments from available sponsorship. A failed execution can still incur gas;
an expired intent cannot release an uncertain, already submitted operation's
reservation without reconciliation. Fees and reserves remain separate from solver,
verification and appeal budgets. Circle Paymaster by itself does not sponsor an
empty wallet; the current profile rejects sponsorship requests.

The execution adapter must validate a fresh trusted clock, nonce and policy state,
chain/code bindings, account ownership, available balance and round eligibility;
build and simulate the exact UserOperation; constrain the total USDC debit; obtain
consent; and durably record its identity before broadcasting. Retries reconcile
the same operation and actual costs instead of duplicating funding or payment.
Any changed operation or increased fee ceiling requires a new bound intent and
consent. The CLI's caller-supplied `--at` is for reproducible offline evaluation.

## Record mapping

| Record | Meaning and existing protocol connection |
|---|---|
| Wallet policy | New adapter configuration schema; uses the existing PolicyID type |
| Wallet intent | New unsigned adapter input; binds discovery trust, registered principal, round, category, nonce, expiry, action and fee ceiling |
| Wallet preflight | New draft schema with `prepared_unsigned`, `funds_reserved: false` and `owner_signature_required: true`; content commitments use ReceiptID with explicit wallet domains |
| Future signed UserOperation | Wallet/EVM authorization; must retain its actual chain hash separately from xLemma intent commitments |
| Future fee receipt | Authenticated actual USDC charge and chain evidence; map to the existing `PaymentReceipt` accounting after defining provider evidence and reconciliation vectors |
| Discovery funding/settlement observation | Existing authenticated observer commands; only canonical confirmed escrow events become funding or settlement evidence |

A preflight is repeatable and consumes no nonce. Its nonce is an application
intent identifier, not the ERC-4337 account nonce. Its ReceiptIDs are draft content
commitments, not payment receipts, research certificates or formal ClaimIDs.
No new research message kind or discovery admission authority is introduced.

## Reproduce and extend

```sh
cargo build --locked -p xlemma-cli
target/debug/xlemma-cli wallet-context \
  config/wallet-base-sepolia.json examples/wallet/trust.json
target/debug/xlemma-cli wallet-preflight \
  config/wallet-base-sepolia.json examples/wallet/trust.json \
  examples/wallet/fund-intent.json --at 1788998460
cargo test --locked -p xlemma-economics --test wallet -p xlemma-cli --test wallet
make validate
```

The [funding](../examples/wallet/fund-expected.json) and
[refund](../examples/wallet/refund-expected.json) vectors contain historical test
times, synthetic round/mandate digests and a fixture escrow and participant set.
They must not be submitted to a chain. They reuse public reference identities
from the discovery tests; no deployed wallet, reviewed escrow or real funds are
claimed. Changing trust configuration creates a new root and requires new intents.

Tests cover configuration/authority substitution, policy drift, expiry, excessive
fees, unknown calls, zero commitments, deterministic commitments and 90 amount
boundary combinations. CLI integration compares both complete outputs and checks
expired requests return failure with no draft on stdout. Schema validation covers
the configuration, both intents, both outputs and discovery trust fixture.

The [validation record](../reports/wallet-preflight-validation.json) records this
slice's source digests, focused tests and full Rust/discovery regression results.
Unchanged Solidity and Lean implementations were not rerun for this adapter-only
change; no new chain execution or research verification claim is made.

The next implementation is the account/SDK adapter, durable authorization and
operation outbox, bounded gas sponsorship, and authenticated receipt observation.
Then exercise the [full testnet journey](USDC_RESEARCH_INFRASTRUCTURE.md#delivery-milestones-and-acceptance)
before any independently reviewed real-USDC pilot.
